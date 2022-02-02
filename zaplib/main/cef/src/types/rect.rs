use std::slice::from_raw_parts;
use zaplib_cef_sys::cef_rect_t;

#[derive(Clone, Debug)]
pub struct CefRect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}
impl CefRect {
    pub(crate) fn from_ptr(raw: *const cef_rect_t) -> Self {
        Self::from(unsafe { &*raw })
    }
    pub(crate) fn from(raw: &cef_rect_t) -> Self {
        CefRect { x: raw.x, y: raw.y, width: raw.width, height: raw.height }
    }
    pub(crate) fn from_array(count: usize, rects: *const cef_rect_t) -> Vec<CefRect> {
        let raw_rects = unsafe { from_raw_parts(rects, count) };
        raw_rects.iter().map(Self::from).collect()
    }
}
impl Default for CefRect {
    fn default() -> Self {
        Self { x: 0, y: 0, width: 0, height: 0 }
    }
}
