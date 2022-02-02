use crate::types::string::CefString;
use crate::WindowInfo;
use std::ptr::null_mut;
use zaplib_cef_sys::cef_window_info_t;

pub type CefArgs<'a> = zaplib_cef_sys::HINSTANCE;

pub(crate) struct CefMainArgsWrapper {
    pub cef: zaplib_cef_sys::_cef_main_args_t,
}

pub(crate) fn args_to_cef(raw: CefArgs) -> CefMainArgsWrapper {
    CefMainArgsWrapper { cef: zaplib_cef_sys::_cef_main_args_t { instance: raw } }
}

pub(crate) fn default_args() -> CefMainArgsWrapper {
    args_to_cef(null_mut())
}

pub(crate) type CefCursorInternal = zaplib_cef_sys::HCURSOR;

impl<'a> WindowInfo<'a> {
    pub(crate) fn to_cef(&self) -> cef_window_info_t {
        cef_window_info_t {
            window_name: CefString::convert_str_to_cef(self.window_name),
            x: self.x as i32,
            y: self.y as i32,
            width: self.width as i32,
            height: self.height as i32,
            parent_window: self.parent_window,
            windowless_rendering_enabled: self.windowless_rendering_enabled as i32,
            shared_texture_enabled: self.shared_texture_enabled as i32,
            external_begin_frame_enabled: self.external_begin_frame_enabled as i32,
            window: self.window,

            // Windows only values
            ex_style: 0,
            style: 0,
            menu: null_mut(),
        }
    }
}
