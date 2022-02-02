use crate::{string::CefString, CefColor};
use zaplib_cef_sys::{cef_browser_settings_t, cef_state_t};

#[cfg(target_os = "windows")]
mod platform {
    use std::ptr::null_mut;

    pub type WindowHandle = zaplib_cef_sys::HWND;

    pub(crate) fn window_handle_default() -> WindowHandle {
        null_mut()
    }
}

#[cfg(target_os = "macos")]
mod platform {
    use std::ptr::null_mut;

    pub type WindowHandle = *mut ::std::os::raw::c_void;

    pub(crate) fn window_handle_default() -> WindowHandle {
        null_mut()
    }
}

#[cfg(target_os = "linux")]
mod platform {
    pub type WindowHandle = u64;

    pub(crate) fn window_handle_default() -> WindowHandle {
        0
    }
}
pub use platform::*;

#[derive(Debug, Copy, Clone)]
pub struct WindowInfo<'a> {
    pub window_name: Option<&'a str>,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,

    pub windowless_rendering_enabled: bool,
    pub shared_texture_enabled: bool,
    pub external_begin_frame_enabled: bool,

    pub parent_window: WindowHandle,
    pub window: WindowHandle,
}
impl<'a> Default for WindowInfo<'a> {
    fn default() -> WindowInfo<'a> {
        WindowInfo {
            window_name: None,
            x: 0,
            y: 0,
            width: 640,
            height: 480,

            windowless_rendering_enabled: false,
            shared_texture_enabled: false,
            external_begin_frame_enabled: false,

            parent_window: window_handle_default(),
            window: window_handle_default(),
        }
    }
}

fn optional_bool_to_cef_state(val: Option<bool>) -> cef_state_t {
    match val {
        None => zaplib_cef_sys::cef_state_t::STATE_DEFAULT,
        Some(false) => zaplib_cef_sys::cef_state_t::STATE_DISABLED,
        Some(true) => zaplib_cef_sys::cef_state_t::STATE_ENABLED,
    }
}

#[derive(Debug, Copy, Clone)]
pub struct BrowserSettings<'a> {
    pub windowless_frame_rate: i32,
    pub standard_font_family: Option<&'a str>,
    pub fixed_font_family: Option<&'a str>,
    pub serif_font_family: Option<&'a str>,
    pub sans_serif_font_family: Option<&'a str>,
    pub cursive_font_family: Option<&'a str>,
    pub fantasy_font_family: Option<&'a str>,
    pub default_font_size: i32,
    pub default_fixed_font_size: i32,
    pub minimum_font_size: i32,
    pub minimum_logical_font_size: i32,
    pub default_encoding: Option<&'a str>,
    pub remote_fonts: Option<bool>,
    pub javascript: Option<bool>,
    pub javascript_close_windows: Option<bool>,
    pub javascript_access_clipboard: Option<bool>,
    pub javascript_dom_paste: Option<bool>,
    pub plugins: Option<bool>,
    pub universal_access_from_file_urls: Option<bool>,
    pub file_access_from_file_urls: Option<bool>,
    pub image_loading: Option<bool>,
    pub image_shrink_standalone_to_fit: Option<bool>,
    pub text_area_resize: Option<bool>,
    pub tab_to_links: Option<bool>,
    pub local_storage: Option<bool>,
    pub databases: Option<bool>,
    pub application_cache: Option<bool>,
    pub webgl: Option<bool>,
    pub background_color: CefColor,
    pub accept_language_list: Option<&'a str>,
}
impl<'a> Default for BrowserSettings<'a> {
    fn default() -> BrowserSettings<'a> {
        BrowserSettings {
            windowless_frame_rate: 0,
            standard_font_family: None,
            fixed_font_family: None,
            serif_font_family: None,
            sans_serif_font_family: None,
            cursive_font_family: None,
            fantasy_font_family: None,
            default_font_size: 0,
            default_fixed_font_size: 0,
            minimum_font_size: 0,
            minimum_logical_font_size: 0,
            default_encoding: None,
            remote_fonts: None,
            javascript: None,
            javascript_close_windows: None,
            javascript_access_clipboard: None,
            javascript_dom_paste: None,
            plugins: None,
            universal_access_from_file_urls: None,
            file_access_from_file_urls: None,
            image_loading: None,
            image_shrink_standalone_to_fit: None,
            text_area_resize: None,
            tab_to_links: None,
            local_storage: None,
            databases: None,
            application_cache: None,
            webgl: None,
            background_color: CefColor::from_u32(0x0000_0000),
            accept_language_list: None,
        }
    }
}
impl<'a> BrowserSettings<'a> {
    pub(crate) fn to_cef(&self) -> cef_browser_settings_t {
        cef_browser_settings_t {
            size: std::mem::size_of::<cef_browser_settings_t>() as u64,
            windowless_frame_rate: self.windowless_frame_rate,
            standard_font_family: CefString::convert_str_to_cef(self.standard_font_family),
            fixed_font_family: CefString::convert_str_to_cef(self.fixed_font_family),
            serif_font_family: CefString::convert_str_to_cef(self.serif_font_family),
            sans_serif_font_family: CefString::convert_str_to_cef(self.sans_serif_font_family),
            cursive_font_family: CefString::convert_str_to_cef(self.cursive_font_family),
            fantasy_font_family: CefString::convert_str_to_cef(self.fantasy_font_family),
            default_font_size: self.default_font_size,
            default_fixed_font_size: self.default_fixed_font_size,
            minimum_font_size: self.minimum_font_size,
            minimum_logical_font_size: self.minimum_logical_font_size,
            default_encoding: CefString::convert_str_to_cef(self.default_encoding),
            remote_fonts: optional_bool_to_cef_state(self.remote_fonts),
            javascript: optional_bool_to_cef_state(self.javascript),
            javascript_close_windows: optional_bool_to_cef_state(self.javascript_close_windows),
            javascript_access_clipboard: optional_bool_to_cef_state(self.javascript_access_clipboard),
            javascript_dom_paste: optional_bool_to_cef_state(self.javascript_dom_paste),
            plugins: optional_bool_to_cef_state(self.plugins),
            universal_access_from_file_urls: optional_bool_to_cef_state(self.universal_access_from_file_urls),
            file_access_from_file_urls: optional_bool_to_cef_state(self.file_access_from_file_urls),
            image_loading: optional_bool_to_cef_state(self.image_loading),
            image_shrink_standalone_to_fit: optional_bool_to_cef_state(self.image_shrink_standalone_to_fit),
            text_area_resize: optional_bool_to_cef_state(self.text_area_resize),
            tab_to_links: optional_bool_to_cef_state(self.tab_to_links),
            local_storage: optional_bool_to_cef_state(self.local_storage),
            databases: optional_bool_to_cef_state(self.databases),
            application_cache: optional_bool_to_cef_state(self.application_cache),
            webgl: optional_bool_to_cef_state(self.webgl),
            background_color: self.background_color.to_cef(),
            accept_language_list: CefString::convert_str_to_cef(self.accept_language_list),
        }
    }
}
