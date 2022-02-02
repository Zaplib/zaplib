use crate::types::string::CefString;
use crate::WindowInfo;
use std::ffi::CString;
use std::os::raw::c_char;
use std::ptr::null_mut;
use zaplib_cef_sys::cef_window_info_t;

pub type CefArgs<'a> = &'a [String];

pub(crate) struct CefMainArgsWrapper {
    pub cef: zaplib_cef_sys::_cef_main_args_t,
    pub keepalive: Vec<CString>,
    pub keepalive2: Vec<*mut c_char>,
}

pub(crate) fn args_to_cef(raw: CefArgs) -> CefMainArgsWrapper {
    // TODO - won't this cause the types to be freed before the pointers?
    let args = raw.iter().map(|x| CString::new(x.as_str()).unwrap()).collect::<Vec<CString>>();
    let mut res = CefMainArgsWrapper {
        cef: zaplib_cef_sys::_cef_main_args_t { argc: 0, argv: null_mut() },
        keepalive: args,
        keepalive2: Vec::new(),
    };

    res.keepalive2 = res.keepalive.iter().map(|x| x.as_ptr() as *mut _).collect();
    res.cef.argc = res.keepalive2.len() as i32;
    res.cef.argv = res.keepalive2.as_mut_ptr();

    res
}

pub(crate) fn default_args() -> CefMainArgsWrapper {
    args_to_cef(&std::env::args().collect::<Vec<String>>())
}

pub(crate) type CefCursorInternal = *mut ::std::os::raw::c_void;
pub(crate) type CefWindowHandle = *mut ::std::os::raw::c_void;

impl<'a> WindowInfo<'a> {
    pub(crate) fn to_cef(&self) -> cef_window_info_t {
        cef_window_info_t {
            window_name: CefString::convert_str_to_cef(self.window_name),
            x: self.x as i32,
            y: self.y as i32,
            width: self.width as i32,
            height: self.height as i32,
            parent_view: self.parent_window,
            windowless_rendering_enabled: self.windowless_rendering_enabled as i32,
            shared_texture_enabled: self.shared_texture_enabled as i32,
            external_begin_frame_enabled: self.external_begin_frame_enabled as i32,
            view: self.window,
            hidden: 0,
        }
    }
}
