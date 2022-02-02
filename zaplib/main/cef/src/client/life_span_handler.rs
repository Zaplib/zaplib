use crate::ptr::{wrap_ptr, BaseRefCountedExt, WrapperFor};
use crate::types::string::CefString;
use crate::{Browser, Frame, ToCef};
use std::sync::Arc;
use zaplib_cef_sys::{
    _cef_popup_features_t, cef_browser_settings_t, cef_browser_t, cef_client_t, cef_dictionary_value_t, cef_frame_t,
    cef_life_span_handler_t, cef_string_t, cef_window_info_t, cef_window_open_disposition_t,
};

pub trait LifeSpanHandler {
    fn on_before_popup(
        &self,
        _browser: &Browser,
        _frame: &Frame,
        _target_url: String,
        _target_frame_name: String,
        _target_disposition: (),
        _user_gesture: bool,
    ) -> bool {
        false
    }
    fn on_after_created(&self, _browser: &Browser) {}
    fn do_close(&self, _browser: &Browser) -> bool {
        false
    }
    fn on_before_close(&self, _browser: &Browser) {}
}
impl LifeSpanHandler for () {}

struct LifeSpanHandlerWrapper<T: LifeSpanHandler> {
    _base: cef_life_span_handler_t,
    internal: Arc<T>,
}
unsafe impl<T: LifeSpanHandler> WrapperFor<cef_life_span_handler_t> for LifeSpanHandlerWrapper<T> {}
impl<T: LifeSpanHandler> LifeSpanHandlerWrapper<T> {
    fn from_ptr<'a>(
        ptr: *mut cef_life_span_handler_t,
    ) -> &'a mut BaseRefCountedExt<cef_life_span_handler_t, LifeSpanHandlerWrapper<T>> {
        unsafe { &mut *(ptr as *mut _) }
    }

    unsafe extern "C" fn on_before_popup(
        handler: *mut cef_life_span_handler_t,
        browser: *mut cef_browser_t,
        frame: *mut cef_frame_t,
        target_url: *const cef_string_t,
        target_frame_name: *const cef_string_t,
        _target_disposition: cef_window_open_disposition_t,
        user_gesture: ::std::os::raw::c_int,
        _popup_features: *const _cef_popup_features_t,
        _window_info: *mut cef_window_info_t,
        _client: *mut *mut cef_client_t,
        _settings: *mut cef_browser_settings_t,
        _extra_info: *mut *mut cef_dictionary_value_t,
        _no_javascript_access: *mut ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int {
        let handler = Self::from_ptr(handler);
        let browser = Browser::from(browser, false);
        let frame = Frame::from(frame, false);
        let target_url = CefString::from_cef(target_url);
        let target_frame_name = CefString::from_cef(target_frame_name);
        // target_disposition

        // TODO - finish
        handler.internal.on_before_popup(
            &browser,
            &frame,
            target_url.to_string(),
            target_frame_name.to_string(),
            (),
            user_gesture != 0,
        ) as i32
    }

    unsafe extern "C" fn on_after_created(handler: *mut cef_life_span_handler_t, browser: *mut cef_browser_t) {
        let handler = Self::from_ptr(handler);
        let browser = Browser::from(browser, false);

        handler.internal.on_after_created(&browser)
    }

    unsafe extern "C" fn do_close(handler: *mut cef_life_span_handler_t, browser: *mut cef_browser_t) -> ::std::os::raw::c_int {
        let handler = Self::from_ptr(handler);
        let browser = Browser::from(browser, false);

        handler.internal.do_close(&browser) as i32
    }

    unsafe extern "C" fn on_before_close(handler: *mut cef_life_span_handler_t, browser: *mut cef_browser_t) {
        let handler = Self::from_ptr(handler);
        let browser = Browser::from(browser, false);

        handler.internal.on_before_close(&browser)
    }
}
impl<T: LifeSpanHandler> ToCef<cef_life_span_handler_t> for Arc<T> {
    fn to_cef(&self) -> *mut cef_life_span_handler_t {
        wrap_ptr(|base| LifeSpanHandlerWrapper {
            _base: cef_life_span_handler_t {
                base,
                on_before_popup: Some(LifeSpanHandlerWrapper::<T>::on_before_popup),
                on_after_created: Some(LifeSpanHandlerWrapper::<T>::on_after_created),
                do_close: Some(LifeSpanHandlerWrapper::<T>::do_close),
                on_before_close: Some(LifeSpanHandlerWrapper::<T>::on_before_close),
            },
            internal: self.clone(),
        })
    }
}
