use std::{ptr::null_mut, sync::Arc};

use zaplib_cef_sys::{
    _cef_resource_handler_t, _cef_resource_request_handler_t, cef_browser_t, cef_frame_t, cef_request_t,
    cef_resource_request_handler_t,
};

use crate::{
    ptr::{wrap_ptr, BaseRefCountedExt, WrapperFor},
    ResourceHandler, ToCef,
};

pub trait ResourceRequestHandler {
    type OutResourceHandler: ResourceHandler;

    fn get_resource_handler(&self) -> Option<Arc<Self::OutResourceHandler>> {
        None
    }
}

struct ResourceRequestHandlerWrapper<T: ResourceRequestHandler> {
    _base: _cef_resource_request_handler_t,
    internal: Arc<T>,
}

unsafe impl<T: ResourceRequestHandler> WrapperFor<cef_resource_request_handler_t> for ResourceRequestHandlerWrapper<T> {}

impl<T: ResourceRequestHandler> ResourceRequestHandlerWrapper<T> {
    fn from_ptr<'a>(
        ptr: *mut cef_resource_request_handler_t,
    ) -> &'a mut BaseRefCountedExt<cef_resource_request_handler_t, ResourceRequestHandlerWrapper<T>> {
        unsafe { &mut *(ptr as *mut _) }
    }

    // extern "C" fn get_cookie_access_filter(
    //     resource_request_handler: *mut _cef_resource_request_handler_t,
    //     browser: *mut cef_browser_t,
    //     frame: *mut cef_frame_t,
    //     request: *mut cef_request_t,
    // ) -> *mut _cef_cookie_access_filter_t {
    //     // TODO
    //     println!("get_cookie_access_filter");
    //     null_mut()
    // }

    // extern "C" fn on_before_resource_load(
    //     resource_request_handler: *mut _cef_resource_request_handler_t,
    //     browser: *mut cef_browser_t,
    //     frame: *mut cef_frame_t,
    //     request: *mut cef_request_t,
    //     callback: *mut _cef_request_callback_t,
    // ) -> cef_return_value_t {
    //     // TODO
    //     println!("on_before_resource_load");
    //     cef_return_value_t::RV_CONTINUE
    // }

    extern "C" fn get_resource_handler(
        handler: *mut _cef_resource_request_handler_t,
        _browser: *mut cef_browser_t,
        _frame: *mut cef_frame_t,
        _request: *mut cef_request_t,
    ) -> *mut _cef_resource_handler_t {
        let resource_request_handler = Self::from_ptr(handler);
        if let Some(resource_handler) = resource_request_handler.internal.get_resource_handler() {
            resource_handler.to_cef()
        } else {
            null_mut()
        }
    }

    // extern "C" fn on_resource_redirect(
    //     resource_request_handler: *mut _cef_resource_request_handler_t,
    //     browser: *mut cef_browser_t,
    //     frame: *mut cef_frame_t,
    //     request: *mut cef_request_t,
    //     response: *mut _cef_response_t,
    //     new_url: *mut cef_string_t,
    // ) {
    //     // TODO
    //     println!("on_resource_redirect");
    // }

    // extern "C" fn on_resource_response(
    //     resource_request_handler: *mut _cef_resource_request_handler_t,
    //     browser: *mut cef_browser_t,
    //     frame: *mut cef_frame_t,
    //     request: *mut cef_request_t,
    //     response: *mut _cef_response_t,
    // ) -> ::std::os::raw::c_int {
    //     // TODO
    //     println!("on_resource_response");
    //     0
    // }

    // extern "C" fn get_resource_response_filter(
    //     resource_request_handler: *mut _cef_resource_request_handler_t,
    //     browser: *mut cef_browser_t,
    //     frame: *mut cef_frame_t,
    //     request: *mut cef_request_t,
    //     response: *mut _cef_response_t,
    // ) -> *mut cef_response_filter_t {
    //     // TODO
    //     println!("get_resource_response_filter");
    //     null_mut()
    // }

    // extern "C" fn on_resource_load_complete(
    //     resource_request_handler: *mut _cef_resource_request_handler_t,
    //     browser: *mut cef_browser_t,
    //     frame: *mut cef_frame_t,
    //     request: *mut cef_request_t,
    //     response: *mut _cef_response_t,
    //     status: cef_urlrequest_status_t,
    //     received_content_length: ::std::os::raw::c_long,
    // ) {
    //     // TODO
    //     println!("on_resource_load_complete");
    // }

    // extern "C" fn on_protocol_execution(
    //     resource_request_handler: *mut _cef_resource_request_handler_t,
    //     browser: *mut cef_browser_t,
    //     frame: *mut cef_frame_t,
    //     request: *mut cef_request_t,
    //     allow_os_execution: *mut ::std::os::raw::c_int,
    // ) {
    //     // TODO
    //     println!("on_protocol_execution");
    // }
}

impl<T: ResourceRequestHandler> ToCef<cef_resource_request_handler_t> for Arc<T> {
    fn to_cef(&self) -> *mut cef_resource_request_handler_t {
        wrap_ptr(|base| ResourceRequestHandlerWrapper {
            _base: cef_resource_request_handler_t {
                base,
                get_cookie_access_filter: None,
                on_before_resource_load: None,
                get_resource_handler: Some(ResourceRequestHandlerWrapper::<T>::get_resource_handler),
                on_resource_redirect: None,
                on_resource_response: None,
                get_resource_response_filter: None,
                on_resource_load_complete: None,
                on_protocol_execution: None,
            },
            internal: self.clone(),
        })
    }
}
