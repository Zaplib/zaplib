use crate::ptr::RefCounterGuard;
use crate::types::string::CefString;
use crate::{BrowserHost, Frame};
use zaplib_cef_sys::cef_browser_t;

pub struct Browser {
    ptr: RefCounterGuard<cef_browser_t>,
}
impl Browser {
    pub(crate) fn from(ptr: *mut cef_browser_t, track_ref: bool) -> Browser {
        unsafe { Browser { ptr: RefCounterGuard::from(&mut (*ptr).base, ptr, track_ref) } }
    }

    pub fn get_host(&self) -> Option<BrowserHost> {
        if let Some(func) = self.ptr.as_ref().get_host {
            Some(BrowserHost::from(unsafe { func(self.ptr.get()) }, true))
        } else {
            None
        }
    }

    pub fn can_go_back(&self) -> bool {
        if let Some(func) = self.ptr.as_ref().can_go_back {
            unsafe { func(self.ptr.get()) > 0 }
        } else {
            false
        }
    }

    pub fn go_back(&self) {
        if let Some(func) = self.ptr.as_ref().go_back {
            unsafe { func(self.ptr.get()) }
        }
    }

    pub fn can_go_forward(&self) -> bool {
        if let Some(func) = self.ptr.as_ref().can_go_forward {
            unsafe { func(self.ptr.get()) > 0 }
        } else {
            false
        }
    }

    pub fn go_forward(&self) {
        if let Some(func) = self.ptr.as_ref().go_forward {
            unsafe { func(self.ptr.get()) }
        }
    }

    pub fn is_loading(&self) -> bool {
        if let Some(func) = self.ptr.as_ref().is_loading {
            unsafe { func(self.ptr.get()) > 0 }
        } else {
            false
        }
    }

    pub fn reload(&self) {
        if let Some(func) = self.ptr.as_ref().reload {
            unsafe { func(self.ptr.get()) }
        }
    }

    pub fn reload_ignore_cache(&self) {
        if let Some(func) = self.ptr.as_ref().reload_ignore_cache {
            unsafe { func(self.ptr.get()) }
        }
    }

    pub fn stop_load(&self) {
        if let Some(func) = self.ptr.as_ref().stop_load {
            unsafe { func(self.ptr.get()) }
        }
    }

    pub fn get_identifier(&self) -> i32 {
        if let Some(func) = self.ptr.as_ref().get_identifier {
            unsafe { func(self.ptr.get()) }
        } else {
            0
        }
    }

    pub fn is_same(&self, that: &Self) -> bool {
        if let Some(func) = self.ptr.as_ref().is_same {
            unsafe { func(self.ptr.get(), that.ptr.get()) > 0 }
        } else {
            false
        }
    }

    pub fn is_popup(&self) -> bool {
        if let Some(func) = self.ptr.as_ref().is_popup {
            unsafe { func(self.ptr.get()) > 0 }
        } else {
            false
        }
    }

    pub fn has_document(&self) -> bool {
        if let Some(func) = self.ptr.as_ref().has_document {
            unsafe { func(self.ptr.get()) > 0 }
        } else {
            false
        }
    }

    pub fn get_main_frame(&self) -> Option<Frame> {
        if let Some(func) = self.ptr.as_ref().get_main_frame {
            let ptr = unsafe { func(self.ptr.get()) };
            if !ptr.is_null() {
                Some(Frame::from(ptr, true))
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn get_focused_frame(&self) -> Option<Frame> {
        if let Some(func) = self.ptr.as_ref().get_focused_frame {
            let ptr = unsafe { func(self.ptr.get()) };
            if !ptr.is_null() {
                Some(Frame::from(ptr, true))
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn get_frame_byident(&self, identifier: i64) -> Option<Frame> {
        if let Some(func) = self.ptr.as_ref().get_frame_byident {
            let ptr = unsafe { func(self.ptr.get(), identifier) };
            if !ptr.is_null() {
                Some(Frame::from(ptr, true))
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn get_frame(&self, name: &str) -> Option<Frame> {
        if let Some(func) = self.ptr.as_ref().get_frame {
            let name = CefString::from_str(name);
            let ptr = unsafe { func(self.ptr.get(), &name.into_cef()) };
            if !ptr.is_null() {
                Some(Frame::from(ptr, true))
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn get_frame_count(&self) -> usize {
        if let Some(func) = self.ptr.as_ref().get_frame_count {
            unsafe { func(self.ptr.get()) as usize }
        } else {
            0
        }
    }

    pub fn get_frame_identifiers(&self) -> Vec<i64> {
        if let Some(func) = self.ptr.as_ref().get_frame_identifiers {
            unsafe {
                let mut identifiers = vec![0; self.get_frame_count()];

                let mut count = 0;
                func(self.ptr.get(), &mut count, identifiers.as_mut_ptr());

                identifiers.resize(count as usize, 0);
                identifiers
            }
        } else {
            Vec::new()
        }
    }

    pub fn get_frame_names(&self) -> Vec<String> {
        if let Some(func) = self.ptr.as_ref().get_frame_names {
            unsafe {
                let names = zaplib_cef_sys::cef_string_list_alloc();
                func(self.ptr.get(), names);

                let res = CefString::parse_string_list(names);
                zaplib_cef_sys::cef_string_list_free(names);

                res
            }
        } else {
            Vec::new()
        }
    }
}
