use zaplib_cef_sys::cef_request_t;

use crate::{ptr::RefCounterGuard, CefString};

pub(crate) struct Request {
    ptr: RefCounterGuard<cef_request_t>,
}

impl Request {
    pub(crate) fn from(ptr: *mut cef_request_t, track_ref: bool) -> Request {
        unsafe { Request { ptr: RefCounterGuard::from(&mut (*ptr).base, ptr, track_ref) } }
    }

    pub(crate) unsafe fn get_url(&self) -> Option<String> {
        if let Some(func) = self.ptr.as_ref().get_url {
            let ptr = func(self.ptr.get());
            let res = CefString::from_cef(ptr);
            zaplib_cef_sys::cef_string_userfree_utf16_free(ptr);
            Some(res.to_string())
        } else {
            None
        }
    }
}
