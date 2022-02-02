use crate::ptr::{wrap_ptr, BaseRefCountedExt, WrapperFor};
use crate::types::string::CefString;
use crate::{Browser, CefCursorInternal, CefSize, Frame, LogSeverity, ToCef};
use std::sync::Arc;
use zaplib_cef_sys::{
    cef_browser_t, cef_cursor_info_t, cef_cursor_type_t, cef_display_handler_t, cef_frame_t, cef_log_severity_t, cef_size_t,
    cef_string_list_t, cef_string_t,
};

pub trait DisplayHandler {
    fn on_address_change(&self, _browser: &Browser, _frame: &Frame, _url: String) {}
    fn on_title_change(&self, _browser: &Browser, _title: String) {}
    fn on_favicon_url_change(&self, _browser: &Browser, _icon_urls: Vec<String>) {}
    fn on_fullscreen_mode_change(&self, _browser: &Browser, _fullscreen: bool) {}
    fn on_tooltip(&self, _browser: &Browser, _text: String) -> bool {
        false
    }
    fn on_status_message(&self, _browser: &Browser, _value: String) {}
    fn on_console_message(&self, _browser: &Browser, _level: LogSeverity, _message: String, _source: String, _line: i32) -> bool {
        false
    }
    fn on_auto_resize(&self, _browser: &Browser, _size: CefSize) -> bool {
        false
    }
    fn on_loading_progress_change(&self, _browser: &Browser, _progress: f64) {}
    fn on_cursor_change(
        &self,
        _browser: &Browser,
        _cursor: CefCursorInternal,
        _type: cef_cursor_type_t,
        _custom_cursor_info: *const cef_cursor_info_t,
    ) -> i32 {
        0
    }
}
impl DisplayHandler for () {}

struct DisplayHandlerWrapper<T: DisplayHandler> {
    _base: cef_display_handler_t,
    internal: Arc<T>,
}
unsafe impl<T: DisplayHandler> WrapperFor<cef_display_handler_t> for DisplayHandlerWrapper<T> {}
impl<T: DisplayHandler> DisplayHandlerWrapper<T> {
    fn from_ptr<'a>(
        ptr: *mut cef_display_handler_t,
    ) -> &'a mut BaseRefCountedExt<cef_display_handler_t, DisplayHandlerWrapper<T>> {
        unsafe { &mut *(ptr as *mut _) }
    }

    unsafe extern "C" fn on_address_change(
        handler: *mut cef_display_handler_t,
        browser: *mut cef_browser_t,
        frame: *mut cef_frame_t,
        url: *const cef_string_t,
    ) {
        let handler = Self::from_ptr(handler);
        let browser = Browser::from(browser, false);
        let frame = Frame::from(frame, false);
        let url = CefString::from_cef(url);

        handler.internal.on_address_change(&browser, &frame, url.to_string())
    }

    unsafe extern "C" fn on_title_change(
        handler: *mut cef_display_handler_t,
        browser: *mut cef_browser_t,
        title: *const cef_string_t,
    ) {
        let handler = Self::from_ptr(handler);
        let browser = Browser::from(browser, false);
        let title = CefString::from_cef(title);

        handler.internal.on_title_change(&browser, title.to_string())
    }

    unsafe extern "C" fn on_favicon_urlchange(
        handler: *mut cef_display_handler_t,
        browser: *mut cef_browser_t,
        icon_urls: cef_string_list_t,
    ) {
        let handler = Self::from_ptr(handler);
        let browser = Browser::from(browser, false);
        let icon_urls = CefString::parse_string_list(icon_urls);

        handler.internal.on_favicon_url_change(&browser, icon_urls)
    }

    unsafe extern "C" fn on_fullscreen_mode_change(
        handler: *mut cef_display_handler_t,
        browser: *mut cef_browser_t,
        fullscreen: ::std::os::raw::c_int,
    ) {
        let handler = Self::from_ptr(handler);
        let browser = Browser::from(browser, false);

        handler.internal.on_fullscreen_mode_change(&browser, fullscreen != 0)
    }

    unsafe extern "C" fn on_tooltip(
        handler: *mut cef_display_handler_t,
        browser: *mut cef_browser_t,
        text: *mut cef_string_t,
    ) -> ::std::os::raw::c_int {
        let handler = Self::from_ptr(handler);
        let browser = Browser::from(browser, false);
        let text = CefString::from_cef(text);

        handler.internal.on_tooltip(&browser, text.to_string()) as i32
    }

    unsafe extern "C" fn on_status_message(
        handler: *mut cef_display_handler_t,
        browser: *mut cef_browser_t,
        value: *const cef_string_t,
    ) {
        let handler = Self::from_ptr(handler);
        let browser = Browser::from(browser, false);
        let value = CefString::from_cef(value);

        handler.internal.on_status_message(&browser, value.to_string())
    }

    unsafe extern "C" fn on_console_message(
        handler: *mut cef_display_handler_t,
        browser: *mut cef_browser_t,
        level: cef_log_severity_t,
        message: *const cef_string_t,
        source: *const cef_string_t,
        line: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int {
        let handler = Self::from_ptr(handler);
        let browser = Browser::from(browser, false);
        let level = std::mem::transmute(level);
        let message = CefString::from_cef(message);
        let source = CefString::from_cef(source);

        handler.internal.on_console_message(&browser, level, message.to_string(), source.to_string(), line) as i32
    }

    unsafe extern "C" fn on_auto_resize(
        handler: *mut cef_display_handler_t,
        browser: *mut cef_browser_t,
        new_size: *const cef_size_t,
    ) -> ::std::os::raw::c_int {
        let handler = Self::from_ptr(handler);
        let browser = Browser::from(browser, false);
        let new_size = CefSize::from_ptr(new_size);

        handler.internal.on_auto_resize(&browser, new_size) as i32
    }

    unsafe extern "C" fn on_loading_progress_change(
        handler: *mut cef_display_handler_t,
        browser: *mut cef_browser_t,
        progress: f64,
    ) {
        let handler = Self::from_ptr(handler);
        let browser = Browser::from(browser, false);

        handler.internal.on_loading_progress_change(&browser, progress)
    }

    unsafe extern "C" fn on_cursor_change(
        handler: *mut cef_display_handler_t,
        browser: *mut cef_browser_t,
        cursor: CefCursorInternal,
        cursor_type: cef_cursor_type_t,
        custom_cursor_info: *const cef_cursor_info_t,
    ) -> i32 {
        let handler = Self::from_ptr(handler);
        let browser = Browser::from(browser, false);

        handler.internal.on_cursor_change(&browser, cursor, cursor_type, custom_cursor_info)
    }
}
impl<T: DisplayHandler> ToCef<cef_display_handler_t> for Arc<T> {
    fn to_cef(&self) -> *mut cef_display_handler_t {
        wrap_ptr(|base| DisplayHandlerWrapper {
            _base: cef_display_handler_t {
                base,
                on_address_change: Some(DisplayHandlerWrapper::<T>::on_address_change),
                on_title_change: Some(DisplayHandlerWrapper::<T>::on_title_change),
                on_favicon_urlchange: Some(DisplayHandlerWrapper::<T>::on_favicon_urlchange),
                on_fullscreen_mode_change: Some(DisplayHandlerWrapper::<T>::on_fullscreen_mode_change),
                on_tooltip: Some(DisplayHandlerWrapper::<T>::on_tooltip),
                on_status_message: Some(DisplayHandlerWrapper::<T>::on_status_message),
                on_console_message: Some(DisplayHandlerWrapper::<T>::on_console_message),
                on_auto_resize: Some(DisplayHandlerWrapper::<T>::on_auto_resize),
                on_loading_progress_change: Some(DisplayHandlerWrapper::<T>::on_loading_progress_change),
                on_cursor_change: Some(DisplayHandlerWrapper::<T>::on_cursor_change),
            },
            internal: self.clone(),
        })
    }
}
