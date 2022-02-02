#[cfg(target_arch = "wasm32")]
extern "C" {
    /// Synchronously read data from a "user file", a handle in JS, e.g. from dragging in a file.
    fn readUserFileRange(user_file_id: u32, buf_ptr: u64, buf_len: u64, file_offset: u64) -> u64;
    /// Synchronously read data from a URL, returning a new buffer. Return value is 0 or 1 depending
    /// on whether the data was successfully read.
    fn readUrlSync(url_ptr: usize, url_len: usize, buf_ptr_out: *mut u32, buf_len_out: *mut u32) -> u32;
}

enum UniversalFileInner {
    /// Actually resolved data; contains the entire file.
    FullyLoaded { data: std::sync::Arc<Vec<u8>>, pos: u64 },

    /// Local file. Only available on native targets.
    ///
    /// The [`std::fs::File`] handle itself gets set lazily when cloning, so that
    /// [`std::clone::Clone::clone`] always succeeds.
    #[cfg(any(doc, not(target_arch = "wasm32")))]
    LocalFile { path: String, file: Option<std::fs::File> },

    /// An actual file handle in JavaScript, e.g. from dragging in a file.
    #[cfg(any(doc, target_arch = "wasm32"))]
    WasmFile { id: usize, size: u64, pos: u64 },
}

/// A file handle that abstracts over the different ways we have to deal with different kinds of
/// files (local files, file URLs, dragged in files).
///
/// It tries to somewhat follow the [`std::fs::File`] API, but there are some major differences:
/// * For the WebAssembly target, opening a file will read the entire file in memory synchronously,
///   over HTTP(S). This is quite different from the native behavior. This is required if we never
///   want [`std::io::Seek::seek`] to fail.
///   TODO(JP): We should consider different behaviors here -- potentially configurable by the user --
///   such as completely disallowing seeking, or seeking into a buffer with a certain size, or even using
///   HTTP Range Requests when available to fetch the data starting at a seek point (when not buffered
///   already). This can somewhat mirror [`std::fs::OpenOptions`]. We can take some inspiration from
///   <https://github.com/cruise-automation/webviz/blob/4dcd47d/packages/webviz-core/src/util/CachedFilelike.js>
///   and <https://github.com/cruise-automation/webviz/blob/4dcd47d/packages/webviz-core/src/dataProviders/BrowserHttpReader.js>.
///   Even just an option to defer loading until the first read would be useful, so you can open a file
///   on the main thread and pass it to another thread for processing (without having to create multiple
///   functions for processing e.g. handles from [`crate::AppOpenFilesEvent`] differently).
/// * [`UniversalFile::open_url`] exists, which is not available in the regular [`std::fs::File`] API. This matches
///   the behavior of the WebAssembly URL loading described above, but works on both WebAssembly and native
///   targets.
/// * You can use [`std::clone::Clone::clone`] to get a truly new handle, e.g. with its own [`std::io::Seek`]
///   state. Also note that it's not a `try_clone` -- it will always succeed. This means that if you
///   clone a handle to a file that doesn't exist anymore, then you'll get that error on the next
///   read, not while cloning.
/// * Currently it only supports reading, not writing. This might change in the future, but requires
///   some thinking about how that should work in WebAssembly.
///
/// Note that you typically want to load files in a thread. Even on native targets the file system
/// can be slow, e.g. when the user has mounted a remote file system, so you want to avoid blocking
/// the UI thread when possible.
///
/// TODO(JP): File handles in WebAssembly ([`UniversalFileInner::WasmFile`]) can't be moved used in threads
/// that were spawned before the file handle became available. It would be nice to figure out some way
/// around this, or to prevent (at compile time) from using these file handles in older threads.
pub struct UniversalFile(UniversalFileInner);

/// Hacky function for determining what is a URL and what isn't.
fn is_absolute_url(path: &str) -> bool {
    path.starts_with("http://") || path.starts_with("https://")
}

/// Actually set [`UniversalFileInner::LocalFile::file`] if it hasn't been set yet.
#[cfg(not(target_arch = "wasm32"))]
fn get_local_file<'a>(path: &'a str, file: &'a mut Option<std::fs::File>) -> std::io::Result<&'a std::fs::File> {
    if file.is_none() {
        *file = Some(std::fs::File::open(path)?);
    }
    Ok(file.as_ref().unwrap())
}

impl UniversalFile {
    /// Open a local/relative file. On the web target this will block until the entire file is loaded.
    ///
    /// Will return an error if the file does not exist.
    ///
    /// This is mostly intended for reading application files. User files should typically be obtained through
    /// an [`crate::AppOpenFilesEvent`].
    ///
    /// On the web target, this will load files relative to the base path, which you can override using the
    /// [<base> tag](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/base).
    pub fn open(path: &str) -> std::io::Result<Self> {
        if is_absolute_url(path) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("'path' is an absolute URL, use 'open_url' instead: {}", path),
            ));
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            Ok(Self(UniversalFileInner::LocalFile { path: path.to_string(), file: Some(std::fs::File::open(path)?) }))
        }
        #[cfg(target_arch = "wasm32")]
        {
            Self::open_url_sync_wasm(path)
        }
    }

    /// Open an absolute URL. This will always block until the entire file is loaded.
    ///
    /// Will return an error if the file does not exist.
    pub fn open_url(url: &str) -> std::io::Result<Self> {
        if !is_absolute_url(url) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("'url' is not an absolute URL, use 'open' instead: {}", url),
            ));
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            Self::open_url_sync_native(url)
        }
        #[cfg(target_arch = "wasm32")]
        {
            Self::open_url_sync_wasm(url)
        }
    }

    /// Create a new [`UniversalFile`] from a JS file handle.
    #[cfg(target_arch = "wasm32")]
    pub(crate) fn from_wasm_file(id: usize, size: u64) -> Self {
        Self(UniversalFileInner::WasmFile { id, size, pos: 0 })
    }

    /// Synchronously load a URL on Wasm targets.
    #[cfg(target_arch = "wasm32")]
    fn open_url_sync_wasm(url: &str) -> std::io::Result<Self> {
        let chars = url.chars().collect::<Vec<char>>();
        unsafe {
            let mut buf_ptr_out: u32 = 0;
            let mut buf_len_out: u32 = 0;
            if readUrlSync(chars.as_ptr() as usize, chars.len() as usize, &mut buf_ptr_out, &mut buf_len_out) == 1 {
                let data = Vec::<u8>::from_raw_parts(buf_ptr_out as *mut u8, buf_len_out as usize, buf_len_out as usize);
                Ok(Self(UniversalFileInner::FullyLoaded { data: std::sync::Arc::new(data), pos: 0 }))
            } else {
                Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Error while loading {}; check the browser console for details", url),
                ))
            }
        }
    }

    /// Synchronously load a URL on native targets.
    #[cfg(not(target_arch = "wasm32"))]
    fn open_url_sync_native(url: &str) -> std::io::Result<Self> {
        if let Ok(resp) = ureq::get(url).call() {
            let mut buffer: Vec<u8> = Vec::new();
            if std::io::Read::read_to_end(&mut resp.into_reader(), &mut buffer).is_ok() {
                Ok(Self(UniversalFileInner::FullyLoaded { data: std::sync::Arc::new(buffer), pos: 0 }))
            } else {
                Err(std::io::Error::new(std::io::ErrorKind::Other, format!("Error while reading {}", url)))
            }
        } else {
            Err(std::io::Error::new(std::io::ErrorKind::Other, format!("Error while loading {}", url)))
        }
    }
}

/// Convenience function to load a local file path into a [`String`].
///
/// Might be faster than manually using [`std::io::Read::read_to_string`] if we can preallocate
/// the size of the [`String`].
pub fn read_to_string(path: &str) -> std::io::Result<String> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::fs::read_to_string(path)
    }
    #[cfg(target_arch = "wasm32")]
    {
        let mut file = UniversalFile::open(path)?;
        // TODO(JP): Use the fact that we should always know the size at this point.
        let mut buffer = String::new();
        std::io::Read::read_to_string(&mut file, &mut buffer)?;
        Ok(buffer)
    }
}

impl Clone for UniversalFile {
    /// Resets the cursor position, as opposed to [`std::fs::File`].
    ///
    /// If the underlying file doesn't exist anymore, you will get an error during the next
    /// read or seek call.
    fn clone(&self) -> Self {
        match &self.0 {
            UniversalFileInner::FullyLoaded { data, pos: _ } => {
                Self(UniversalFileInner::FullyLoaded { data: std::sync::Arc::clone(data), pos: 0 })
            }
            #[cfg(not(target_arch = "wasm32"))]
            UniversalFileInner::LocalFile { path, file: _ } => {
                Self(UniversalFileInner::LocalFile { path: path.clone(), file: None })
            }
            #[cfg(target_arch = "wasm32")]
            UniversalFileInner::WasmFile { id, size, pos: _ } => {
                Self(UniversalFileInner::WasmFile { id: *id, size: *size, pos: 0 })
            }
        }
    }
}

impl std::io::Read for UniversalFile {
    /// Adapted from [`std::io::Cursor`].
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match &mut self.0 {
            UniversalFileInner::FullyLoaded { data, pos } => {
                let amt = std::cmp::min(*pos, data.len() as u64);
                let mut read_buf = &data[(amt as usize)..];
                let bytes_read = std::io::Read::read(&mut read_buf, buf)?;
                *pos += bytes_read as u64;
                Ok(bytes_read)
            }
            #[cfg(not(target_arch = "wasm32"))]
            UniversalFileInner::LocalFile { path, file } => get_local_file(path, file)?.read(buf),
            #[cfg(target_arch = "wasm32")]
            UniversalFileInner::WasmFile { id, size: _, pos } => unsafe {
                let bytes_read: u64 = readUserFileRange(*id as u32, buf.as_ptr() as u64, buf.len() as u64, *pos);
                *pos += bytes_read;
                Ok(bytes_read as usize)
            },
        }
    }
}

/// Adapted from [`std::io::Cursor`].
fn update_pos(pos: &mut u64, size: u64, style: std::io::SeekFrom) -> std::io::Result<u64> {
    let (base_pos, offset) = match style {
        std::io::SeekFrom::Start(n) => {
            *pos = n;
            return Ok(n);
        }
        std::io::SeekFrom::End(n) => (size, n),
        std::io::SeekFrom::Current(n) => (*pos, n),
    };
    let new_pos =
        if offset >= 0 { base_pos.checked_add(offset as u64) } else { base_pos.checked_sub((offset.wrapping_neg()) as u64) };
    match new_pos {
        Some(n) => {
            *pos = n;
            Ok(*pos)
        }
        None => Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid seek to a negative or overflowing position")),
    }
}
impl std::io::Seek for UniversalFile {
    fn seek(&mut self, style: std::io::SeekFrom) -> std::io::Result<u64> {
        match &mut self.0 {
            UniversalFileInner::FullyLoaded { data, pos } => update_pos(pos, data.len() as u64, style),
            #[cfg(not(target_arch = "wasm32"))]
            UniversalFileInner::LocalFile { path, file } => get_local_file(path, file)?.seek(style),
            #[cfg(target_arch = "wasm32")]
            UniversalFileInner::WasmFile { id: _, size, pos } => update_pos(pos, *size, style),
        }
    }
}

impl crate::ReadSeek for UniversalFile {}

impl std::fmt::Debug for UniversalFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<UniversalFile>")
    }
}
