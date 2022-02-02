use std::{ptr::null_mut, sync::Arc};

use zaplib_cef_sys::{
    _cef_resource_request_handler_t, cef_browser_t, cef_frame_t, cef_request_handler_t, cef_request_t, cef_string_t,
};

use crate::{
    client::request::Request,
    ptr::{wrap_ptr, BaseRefCountedExt, WrapperFor},
    ResourceRequestHandler, ToCef,
};

pub trait RequestHandler {
    type OutResourceRequestHandler: ResourceRequestHandler;

    fn get_resource_request_handler(&self, _url: &str) -> Option<Arc<Self::OutResourceRequestHandler>> {
        None
    }
}

struct RequestHandlerWrapper<T: RequestHandler> {
    _base: cef_request_handler_t,
    internal: Arc<T>,
}

unsafe impl<T: RequestHandler> WrapperFor<cef_request_handler_t> for RequestHandlerWrapper<T> {}

impl<T: RequestHandler> RequestHandlerWrapper<T> {
    fn from_ptr<'a>(
        ptr: *mut cef_request_handler_t,
    ) -> &'a mut BaseRefCountedExt<cef_request_handler_t, RequestHandlerWrapper<T>> {
        unsafe { &mut *(ptr as *mut _) }
    }

    // extern "C" fn on_before_browse(
    //     request_handler: *mut cef_request_handler_t,
    //     browser: *mut cef_browser_t,
    //     frame: *mut cef_frame_t,
    //     request: *mut cef_request_t,
    //     user_gesture: ::std::os::raw::c_int,
    //     is_redirect: ::std::os::raw::c_int,
    // ) -> ::std::os::raw::c_int {
    //     // TODO
    //     println!("on_before_browse");
    //     0
    // }

    // extern "C" fn on_open_urlfrom_tab(
    //     request_handler: *mut cef_request_handler_t,
    //     browser: *mut cef_browser_t,
    //     frame: *mut cef_frame_t,
    //     target_url: *const cef_string_t,
    //     target_disposition: cef_window_open_disposition_t,
    //     user_gesture: ::std::os::raw::c_int,
    // ) -> ::std::os::raw::c_int {
    //     // TODO
    //     let target_url = unsafe { CefString::from_cef(target_url).to_string() };
    //     println!("on_open_urlfrom_tab: {}", target_url);
    //     0
    // }

    unsafe extern "C" fn get_resource_request_handler(
        request_handler: *mut cef_request_handler_t,
        _browser: *mut cef_browser_t,
        _frame: *mut cef_frame_t,
        request: *mut cef_request_t,
        _is_navigation: ::std::os::raw::c_int,
        _is_download: ::std::os::raw::c_int,
        _request_initiator: *const cef_string_t,
        _disable_default_handling: *mut ::std::os::raw::c_int,
    ) -> *mut _cef_resource_request_handler_t {
        let request = Request::from(request, false);
        let url = request.get_url().unwrap_or("".to_string());

        let request_handler = Self::from_ptr(request_handler);
        if let Some(handler) = request_handler.internal.get_resource_request_handler(&url) {
            handler.to_cef()
        } else {
            null_mut()
        }
    }

    // extern "C" fn get_auth_credentials(
    //     request_handler: *mut cef_request_handler_t,
    //     browser: *mut cef_browser_t,
    //     origin_url: *const cef_string_t,
    //     is_proxy: ::std::os::raw::c_int,
    //     host: *const cef_string_t,
    //     port: ::std::os::raw::c_int,
    //     realm: *const cef_string_t,
    //     scheme: *const cef_string_t,
    //     callback: *mut cef_auth_callback_t,
    // ) -> ::std::os::raw::c_int {
    //     // TODO
    //     println!("get_auth_credentials");
    //     0
    // }

    // extern "C" fn on_quota_request(
    //     request_handler: *mut cef_request_handler_t,
    //     browser: *mut cef_browser_t,
    //     origin_url: *const cef_string_t,
    //     new_size: ::std::os::raw::c_long,
    //     callback: *mut _cef_request_callback_t,
    // ) -> ::std::os::raw::c_int {
    //     // TODO
    //     println!("on_quota_request");
    //     0
    // }

    // extern "C" fn on_certificate_error(
    //     request_handler: *mut cef_request_handler_t,
    //     browser: *mut cef_browser_t,
    //     cert_error: cef_errorcode_t,
    //     request_url: *const cef_string_t,
    //     ssl_info: *mut _cef_sslinfo_t,
    //     callback: *mut _cef_request_callback_t,
    // ) -> ::std::os::raw::c_int {
    //     // TODO
    //     println!("on_certificate_error");
    //     0
    // }

    // extern "C" fn on_select_client_certificate(
    //     request_handler: *mut cef_request_handler_t,
    //     browser: *mut cef_browser_t,
    //     is_proxy: ::std::os::raw::c_int,
    //     host: *const cef_string_t,
    //     port: ::std::os::raw::c_int,
    //     certificates_count: ::std::os::raw::c_ulong,
    //     certificates: *const *mut _cef_x509certificate_t,
    //     callback: *mut _cef_select_client_certificate_callback_t,
    // ) -> ::std::os::raw::c_int {
    //     // TODO
    //     println!("on_select_client_certificate");
    //     0
    // }

    // extern "C" fn on_plugin_crashed(
    //     request_handler: *mut cef_request_handler_t,
    //     browser: *mut cef_browser_t,
    //     plugin_path: *const cef_string_t,
    // ) {
    //     // TODO
    //     println!("on_plugin_crashed");
    // }

    // extern "C" fn on_render_view_ready(request_handler: *mut cef_request_handler_t, browser: *mut cef_browser_t) {
    //     // TODO
    //     println!("on_render_view_ready");
    // }

    // extern "C" fn on_render_process_terminated(
    //     request_handler: *mut cef_request_handler_t,
    //     browser: *mut cef_browser_t,
    //     status: cef_termination_status_t,
    // ) {
    //     // TODO
    //     println!("on_render_process_terminated");
    // }

    // extern "C"
    // fn on_document_available_in_main_frame(request_handler: *mut cef_request_handler_t, browser: *mut cef_browser_t) {
    //     // TODO
    //     println!("on_document_available_in_main_frame");
    // }
}

impl<T: RequestHandler> ToCef<cef_request_handler_t> for Arc<T> {
    fn to_cef(&self) -> *mut cef_request_handler_t {
        wrap_ptr(|base| RequestHandlerWrapper {
            _base: cef_request_handler_t {
                base,
                on_before_browse: None,
                on_open_urlfrom_tab: None,
                get_resource_request_handler: Some(RequestHandlerWrapper::<T>::get_resource_request_handler),
                get_auth_credentials: None,
                on_quota_request: None,
                on_certificate_error: None,
                on_select_client_certificate: None,
                on_plugin_crashed: None,
                on_render_view_ready: None,
                on_render_process_terminated: None,
                on_document_available_in_main_frame: None,
            },
            internal: self.clone(),
        })
    }
}
