use zaplib_cef_sys::cef_response_t;

use crate::{ptr::RefCounterGuard, CefString};

pub(crate) struct Response {
    ptr: RefCounterGuard<cef_response_t>,
}

impl Response {
    pub(crate) fn from(ptr: *mut cef_response_t, track_ref: bool) -> Response {
        unsafe { Response { ptr: RefCounterGuard::from(&mut (*ptr).base, ptr, track_ref) } }
    }

    pub(crate) fn set_status(&self, status: i32) {
        if let Some(func) = self.ptr.as_ref().set_status {
            unsafe { func(self.ptr.get(), status) };
        }
    }

    pub(crate) fn set_status_text(&self, status: &str) {
        if let Some(func) = self.ptr.as_ref().set_status_text {
            let status = CefString::from_str(status);
            unsafe { func(self.ptr.get(), &status.into_cef()) };
        }
    }

    pub(crate) fn set_mime_type(&self, mime_type: &str) {
        if let Some(func) = self.ptr.as_ref().set_mime_type {
            let mime_type = CefString::from_str(mime_type);
            unsafe { func(self.ptr.get(), &mime_type.into_cef()) };
        }
    }
}
