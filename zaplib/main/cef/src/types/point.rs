use zaplib_cef_sys::cef_point_t;

#[derive(Clone, Debug)]
pub struct CefPoint {
    pub x: i32,
    pub y: i32,
}
impl CefPoint {
    #[allow(dead_code)]
    pub(crate) fn from_ptr(raw: *const cef_point_t) -> Self {
        Self::from(unsafe { &*raw })
    }
    #[allow(dead_code)]
    pub(crate) fn from(raw: &cef_point_t) -> Self {
        Self { x: raw.x, y: raw.y }
    }
}
impl Default for CefPoint {
    fn default() -> Self {
        Self { x: 0, y: 0 }
    }
}
