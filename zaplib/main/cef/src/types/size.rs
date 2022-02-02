use zaplib_cef_sys::cef_size_t;

#[derive(Clone, Debug)]
pub struct CefSize {
    pub width: i32,
    pub height: i32,
}
impl CefSize {
    pub(crate) fn from_ptr(raw: *const cef_size_t) -> Self {
        Self::from(unsafe { &*raw })
    }
    pub(crate) fn from(raw: &cef_size_t) -> Self {
        CefSize { width: raw.width, height: raw.height }
    }
}
impl Default for CefSize {
    fn default() -> Self {
        Self { width: 0, height: 0 }
    }
}
