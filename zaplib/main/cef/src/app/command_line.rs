use crate::ptr::RefCounterGuard;
use crate::types::string::CefString;
use std::collections::HashMap;
use zaplib_cef_sys::cef_command_line_t;

#[derive(Clone)]
pub struct CommandLine {
    ptr: RefCounterGuard<cef_command_line_t>,
}
impl CommandLine {
    pub(crate) fn from(ptr: *mut cef_command_line_t, track_ref: bool) -> CommandLine {
        unsafe { CommandLine { ptr: RefCounterGuard::from(&mut (*ptr).base, ptr, track_ref) } }
    }

    pub fn is_valid(&self) -> bool {
        if let Some(func) = self.ptr.as_ref().is_valid {
            unsafe { func(self.ptr.get()) > 0 }
        } else {
            false
        }
    }

    pub fn is_read_only(&self) -> bool {
        if let Some(func) = self.ptr.as_ref().is_read_only {
            unsafe { func(self.ptr.get()) > 0 }
        } else {
            false
        }
    }

    pub fn copy(&self) -> CommandLine {
        self.clone()
    }

    pub fn init_from_argv(&self, _argc: ::std::os::raw::c_int, _argv: *const *const ::std::os::raw::c_char) {
        // TODO
    }

    pub fn init_from_string(&self, command_line: &str) {
        if let Some(func) = self.ptr.as_ref().init_from_string {
            let command_line = CefString::from_str(command_line);
            unsafe { func(self.ptr.get(), &command_line.into_cef()) }
        }
    }

    pub fn reset(&self) {
        if let Some(func) = self.ptr.as_ref().reset {
            unsafe { func(self.ptr.get()) }
        }
    }

    pub fn get_argv(&self) -> Vec<String> {
        if let Some(func) = self.ptr.as_ref().get_argv {
            unsafe {
                let arguments = zaplib_cef_sys::cef_string_list_alloc();
                func(self.ptr.get(), arguments);

                let res = CefString::parse_string_list(arguments);
                zaplib_cef_sys::cef_string_list_free(arguments);

                res
            }
        } else {
            Vec::new()
        }
    }

    pub fn get_command_line_string(&self) -> String {
        if let Some(func) = self.ptr.as_ref().get_command_line_string {
            unsafe { CefString::from_userfree_cef(func(self.ptr.get())) }.to_string()
        } else {
            "".to_string()
        }
    }

    pub fn get_program(&self) -> String {
        if let Some(func) = self.ptr.as_ref().get_program {
            unsafe { CefString::from_userfree_cef(func(self.ptr.get())) }.to_string()
        } else {
            "".to_string()
        }
    }

    pub fn set_program(&self, program: &str) {
        if let Some(func) = self.ptr.as_ref().set_program {
            let program = CefString::from_str(program);
            unsafe { func(self.ptr.get(), &program.into_cef()) }
        }
    }

    pub fn has_switches(&self) -> bool {
        if let Some(func) = self.ptr.as_ref().has_switches {
            unsafe { func(self.ptr.get()) > 0 }
        } else {
            false
        }
    }

    pub fn has_switch(&self, name: &str) -> bool {
        if let Some(func) = self.ptr.as_ref().has_switch {
            let name = CefString::from_str(name);
            unsafe { func(self.ptr.get(), &name.into_cef()) > 0 }
        } else {
            false
        }
    }

    pub fn get_switch_value(&self, name: &str) -> String {
        if let Some(func) = self.ptr.as_ref().get_switch_value {
            let name = CefString::from_str(name);
            unsafe { CefString::from_userfree_cef(func(self.ptr.get(), &name.into_cef())) }.to_string()
        } else {
            "".to_string()
        }
    }

    pub fn get_switches(&self) -> HashMap<String, String> {
        if let Some(func) = self.ptr.as_ref().get_switches {
            unsafe {
                let switches = zaplib_cef_sys::cef_string_map_alloc();
                func(self.ptr.get(), switches);

                let res = CefString::parse_string_map(switches);
                zaplib_cef_sys::cef_string_map_free(switches);

                res
            }
        } else {
            HashMap::new()
        }
    }

    pub fn append_switch(&self, name: &str) {
        if let Some(func) = self.ptr.as_ref().append_switch {
            let name = CefString::from_str(name);
            unsafe { func(self.ptr.get(), &name.into_cef()) }
        }
    }

    pub fn append_switch_with_value(&self, name: &str, value: &str) {
        if let Some(func) = self.ptr.as_ref().append_switch_with_value {
            let name = CefString::from_str(name);
            let value = CefString::from_str(value);
            unsafe { func(self.ptr.get(), &name.into_cef(), &value.into_cef()) }
        }
    }

    pub fn has_arguments(&self) -> bool {
        if let Some(func) = self.ptr.as_ref().has_arguments {
            unsafe { func(self.ptr.get()) > 0 }
        } else {
            false
        }
    }

    pub fn get_arguments(&self) -> Vec<String> {
        if let Some(func) = self.ptr.as_ref().get_arguments {
            unsafe {
                let arguments = zaplib_cef_sys::cef_string_list_alloc();
                func(self.ptr.get(), arguments);

                let res = CefString::parse_string_list(arguments);
                zaplib_cef_sys::cef_string_list_free(arguments);

                res
            }
        } else {
            Vec::new()
        }
    }

    pub fn append_argument(&self, argument: &str) {
        if let Some(func) = self.ptr.as_ref().append_argument {
            let argument = CefString::from_str(argument);
            unsafe { func(self.ptr.get(), &argument.into_cef()) }
        }
    }

    pub fn prepend_wrapper(&self, wrapper: &str) {
        if let Some(func) = self.ptr.as_ref().prepend_wrapper {
            let wrapper = CefString::from_str(wrapper);
            unsafe { func(self.ptr.get(), &wrapper.into_cef()) }
        }
    }
}
