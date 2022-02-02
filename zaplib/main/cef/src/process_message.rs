use crate::ptr::RefCounterGuard;
use crate::types::string::CefString;
use crate::ToCefAsArg;

use zaplib_cef_sys::{cef_process_message_create, cef_process_message_t};

pub struct ProcessMessage {
    ptr: RefCounterGuard<cef_process_message_t>,
}

impl ToCefAsArg<cef_process_message_t> for ProcessMessage {
    fn to_cef_as_arg(&self) -> *mut cef_process_message_t {
        self.ptr.to_cef_as_arg()
    }
}

impl ProcessMessage {
    pub(crate) fn from(ptr: *mut cef_process_message_t, track_ref: bool) -> Self {
        unsafe { Self { ptr: RefCounterGuard::from(&mut (*ptr).base, ptr, track_ref) } }
    }

    pub fn create(name: &str) -> ProcessMessage {
        let name = CefString::from_str(name);
        unsafe { ProcessMessage::from(cef_process_message_create(&name.into_cef()), true) }
    }

    // TODO(Dmitry): implement more API when needed
    // pub fn is_valid(&self) -> bool {}
    // pub fn is_read_only(&self) -> bool {}
    // pub fn copy(&self) -> *mut _cef_process_message_t {}
    // pub fn get_name(&self) -> cef_string_userfree_t {}
    // pub fn get_argument_list(&self) -> *mut _cef_list_value_t {}
}
