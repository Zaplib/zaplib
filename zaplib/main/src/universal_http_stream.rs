/// Create an HTTP stream that you can synchronously read from as data comes in.
///
/// Automatically supports gzip (native) or whatever your browser supports (wasm).
/// TODO(JP): Maybe make compression support optional for native builds?
///
/// Returns a [`std::io::Read`]er that blocks until there is data available. It is
/// highly recommended to only use this in a dedicated thread, and to wrap it in
/// [`std::io::BufReader`].
///
/// TODO(JP): See if there is some way to unify this with [`crate::universal_file::UniversalFile`].
pub fn request(url: &str, method: &str, body: &[u8], headers: &[(&str, &str)]) -> std::io::Result<Box<dyn std::io::Read + Send>> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let mut request = ureq::request(&method.to_ascii_uppercase(), url);

        request = request.set("accept-encoding", "gzip");
        for (name, value) in headers {
            request = request.set(name, value);
        }

        match request.send_bytes(body) {
            Ok(response) => {
                if let Some(encoding) = response.header("content-encoding") {
                    if encoding == "gzip" {
                        Ok(Box::new(flate2::read::GzDecoder::new(response.into_reader())))
                    } else {
                        Err(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            format!("Unsupported response encoding: {}", encoding),
                        ))
                    }
                } else {
                    Ok(Box::new(response.into_reader()))
                }
            }
            Err(error) => Err(std::io::Error::new(std::io::ErrorKind::Other, format!("Error opening stream: {}", error))),
        }
    }

    #[cfg(target_arch = "wasm32")]
    {
        let stream_id = crate::send_task_worker_message_http_stream_new(url, method, body, headers);
        if stream_id == crate::TASK_WORKER_ERROR_RETURN_VALUE {
            Err(std::io::Error::new(std::io::ErrorKind::Other, "Error opening stream"))
        } else {
            Ok(Box::new(UniversalHttpStreamReader(stream_id)))
        }
    }
}

/// Contains just a `stream_id` to make [`std::io::Read::read`] calls with in WebAssembly.
#[cfg(target_arch = "wasm32")]
struct UniversalHttpStreamReader(i32);

#[cfg(target_arch = "wasm32")]
impl std::io::Read for UniversalHttpStreamReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if buf.len() == 0 {
            return Ok(0);
        }
        let bytes_read = crate::send_task_worker_message_http_stream_read(self.0, buf.as_mut_ptr(), buf.len());
        if bytes_read == crate::TASK_WORKER_ERROR_RETURN_VALUE {
            Err(std::io::Error::new(std::io::ErrorKind::Other, "Error reading from stream"))
        } else {
            Ok(bytes_read as usize)
        }
    }
}
