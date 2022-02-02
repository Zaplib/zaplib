use std::{
    io::{Error, ErrorKind, Result},
    sync::Arc,
};

use zaplib_cef_sys::{
    _cef_callback_t, _cef_resource_handler_t, _cef_resource_read_callback_t, _cef_response_t, cef_request_t, cef_string_t,
};

use crate::{
    client::response::Response,
    ptr::{wrap_ptr, BaseRefCountedExt, WrapperFor},
    ToCef,
};

use super::request::Request;

pub trait ResourceHandler {
    fn open(&self, _url: &str) -> bool {
        false
    }

    fn get_mime_type(&self) -> Option<String> {
        None
    }

    fn get_status_code(&self) -> i32 {
        0
    }

    fn get_response_length(&self) -> i64 {
        0
    }

    fn read(&self, _buf: &mut [u8]) -> Result<usize> {
        Err(Error::new(ErrorKind::Unsupported, "Unsupported operation"))
    }
}

struct ResourceHandlerWrapper<T: ResourceHandler> {
    _base: _cef_resource_handler_t,
    internal: Arc<T>,
}

unsafe impl<T: ResourceHandler> WrapperFor<_cef_resource_handler_t> for ResourceHandlerWrapper<T> {}

impl<T: ResourceHandler> ResourceHandlerWrapper<T> {
    fn from_ptr<'a>(
        ptr: *mut _cef_resource_handler_t,
    ) -> &'a mut BaseRefCountedExt<_cef_resource_handler_t, ResourceHandlerWrapper<T>> {
        unsafe { &mut *(ptr as *mut _) }
    }

    unsafe extern "C" fn open(
        resource_handler: *mut _cef_resource_handler_t,
        request: *mut cef_request_t,
        handle_request: *mut ::std::os::raw::c_int,
        _callback: *mut _cef_callback_t,
    ) -> ::std::os::raw::c_int {
        let request = Request::from(request, false);
        let url = request.get_url().unwrap_or("".to_string());

        let handler = Self::from_ptr(resource_handler);
        if handler.internal.open(&url) {
            *handle_request = 1;
            1
        } else {
            0
        }
    }

    // // WARNING: This function is deprecated. Use open instead.
    // extern "C" fn process_request(
    //     resource_handler: *mut _cef_resource_handler_t,
    //     request: *mut cef_request_t,
    //     callback: *mut _cef_callback_t,
    // ) -> ::std::os::raw::c_int {
    //     0
    // }

    unsafe extern "C" fn get_response_headers(
        resource_handler: *mut _cef_resource_handler_t,
        response: *mut _cef_response_t,
        response_length: *mut ::std::os::raw::c_long,
        _redirect_url: *mut cef_string_t,
    ) {
        let handler = Self::from_ptr(resource_handler);

        let response = Response::from(response, false);
        response.set_status(handler.internal.get_status_code());
        // TODO(hernan): get actual status text based on status code
        response.set_status_text("OK");
        response.set_mime_type(&handler.internal.get_mime_type().unwrap());

        *response_length = handler.internal.get_response_length();
    }

    // extern "C" fn skip(
    //     resource_handler: *mut _cef_resource_handler_t,
    //     bytes_to_skip: ::std::os::raw::c_long,
    //     bytes_skipped: *mut ::std::os::raw::c_long,
    //     callback: *mut _cef_resource_skip_callback_t,
    // ) -> ::std::os::raw::c_int {
    //     println!("skip: {}", bytes_to_skip);
    //     0
    // }

    unsafe extern "C" fn read(
        resource_handler: *mut _cef_resource_handler_t,
        data_out: *mut ::std::os::raw::c_void,
        bytes_to_read: ::std::os::raw::c_int,
        bytes_read: *mut ::std::os::raw::c_int,
        _callback: *mut _cef_resource_read_callback_t,
    ) -> ::std::os::raw::c_int {
        let handler = Self::from_ptr(resource_handler);
        let data = std::slice::from_raw_parts_mut(data_out as *mut u8, bytes_to_read as usize);
        *bytes_read = match handler.internal.read(data) {
            Ok(bytes) => bytes as i32,
            _ => 0,
        };
        *bytes_read
    }

    // // WARNING: This function is deprecated. Use read instead.
    // extern "C" fn read_response(
    //     resource_handler: *mut _cef_resource_handler_t,
    //     data_out: *mut ::std::os::raw::c_void,
    //     bytes_to_read: ::std::os::raw::c_int,
    //     bytes_read: *mut ::std::os::raw::c_int,
    //     callback: *mut _cef_callback_t,
    // ) -> ::std::os::raw::c_int {
    //     0
    // }

    // extern "C" fn cancel(resource_handler: *mut _cef_resource_handler_t) {
    //     println!("cancel:");
    // }
}

impl<T: ResourceHandler> ToCef<_cef_resource_handler_t> for Arc<T> {
    fn to_cef(&self) -> *mut _cef_resource_handler_t {
        wrap_ptr(|base| ResourceHandlerWrapper {
            _base: _cef_resource_handler_t {
                base,
                open: Some(ResourceHandlerWrapper::<T>::open),
                process_request: None,
                get_response_headers: Some(ResourceHandlerWrapper::<T>::get_response_headers),
                skip: None,
                read: Some(ResourceHandlerWrapper::<T>::read),
                read_response: None,
                cancel: None,
            },
            internal: self.clone(),
        })
    }
}
