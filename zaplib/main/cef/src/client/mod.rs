mod context_menu_handler;
mod display_handler;
mod life_span_handler;
mod render_handler;
mod request;
mod request_handler;
mod resource_handler;
mod resource_request_handler;
mod response;

pub use context_menu_handler::*;
pub use display_handler::*;
pub use life_span_handler::*;
pub use render_handler::*;
pub use request_handler::*;
pub use resource_handler::*;
pub use resource_request_handler::*;

use crate::ptr::{wrap_ptr, BaseRefCountedExt, WrapperFor};
use crate::ToCef;
use std::ptr::null_mut;
use std::sync::Arc;
use zaplib_cef_sys::{cef_client_t, cef_context_menu_handler_t, cef_request_handler_t};

pub trait Client {
    // type OutAudioHandler: AudioHandler;
    // type OutDisplayHandler: DisplayHandler;
    // type OutLifeSpanHandler: LifeSpanHandler;
    // type OutRenderHandler: RenderHandler;
    type OutContextMenuHandler: ContextMenuHandler;
    type OutRequestHandler: RequestHandler;

    // TODO - fill out
    // fn get_audio_handler(&self) -> Option<Arc<Self::OutAudioHandler>> {
    //     None
    // }
    // fn get_display_handler(&self) -> Option<Arc<Self::OutDisplayHandler>> {
    //     None
    // }
    // fn get_life_span_handler(&self) -> Option<Arc<Self::OutLifeSpanHandler>> {
    //     None
    // }
    // fn get_render_handler(&self) -> Option<Arc<Self::OutRenderHandler>> {
    //     None
    // }

    fn get_context_menu_handler(&self) -> Option<Arc<Self::OutContextMenuHandler>> {
        None
    }

    fn get_request_handler(&self) -> Option<Arc<Self::OutRequestHandler>> {
        None
    }
}

struct ClientWrapper<T: Client> {
    _base: cef_client_t,
    internal: Arc<T>,
}
unsafe impl<T: Client> WrapperFor<cef_client_t> for ClientWrapper<T> {}
impl<T: Client> ClientWrapper<T> {
    fn from_ptr<'a>(ptr: *mut cef_client_t) -> &'a mut BaseRefCountedExt<cef_client_t, ClientWrapper<T>> {
        unsafe { &mut *(ptr as *mut _) }
    }

    // extern "C" fn get_audio_handler(_client: *mut cef_client_t) -> *mut cef_audio_handler_t {
    //     null_mut()
    // }

    extern "C" fn get_context_menu_handler(client: *mut cef_client_t) -> *mut cef_context_menu_handler_t {
        let client = Self::from_ptr(client);
        if let Some(handler) = client.internal.get_context_menu_handler() {
            handler.to_cef()
        } else {
            null_mut()
        }
    }

    // extern "C" fn get_dialog_handler(_client: *mut cef_client_t) -> *mut cef_dialog_handler_t {
    //     null_mut()
    // }

    // extern "C" fn get_display_handler(client: *mut cef_client_t) -> *mut cef_display_handler_t {
    //     let client = Self::from_ptr(client);
    //     if let Some(handler) = client.internal.get_display_handler() {
    //         handler.to_cef()
    //     } else {
    //         null_mut()
    //     }
    // }

    // extern "C" fn get_download_handler(_client: *mut cef_client_t) -> *mut cef_download_handler_t {
    //     null_mut()
    // }

    // extern "C" fn get_drag_handler(_client: *mut cef_client_t) -> *mut cef_drag_handler_t {
    //     null_mut()
    // }

    // extern "C" fn get_find_handler(_client: *mut cef_client_t) -> *mut cef_find_handler_t {
    //     null_mut()
    // }

    // extern "C" fn get_focus_handler(_client: *mut cef_client_t) -> *mut cef_focus_handler_t {
    //     null_mut()
    // }

    // extern "C" fn get_jsdialog_handler(_client: *mut cef_client_t) -> *mut cef_jsdialog_handler_t {
    //     null_mut()
    // }

    // extern "C" fn get_keyboard_handler(_client: *mut cef_client_t) -> *mut cef_keyboard_handler_t {
    //     null_mut()
    // }

    // extern "C" fn get_life_span_handler(client: *mut cef_client_t) -> *mut cef_life_span_handler_t {
    //     let client = Self::from_ptr(client);
    //     if let Some(handler) = client.internal.get_life_span_handler() {
    //         handler.to_cef()
    //     } else {
    //         null_mut()
    //     }
    // }

    // extern "C" fn get_load_handler(_client: *mut cef_client_t) -> *mut cef_load_handler_t {
    //     null_mut()
    // }

    // extern "C" fn get_render_handler(client: *mut cef_client_t) -> *mut cef_render_handler_t {
    //     let client = Self::from_ptr(client);
    //     if let Some(handler) = client.internal.get_render_handler() {
    //         handler.to_cef()
    //     } else {
    //         null_mut()
    //     }
    // }

    // extern "C" fn get_print_handler(_client: *mut cef_client_t) -> *mut cef_print_handler_t {
    //     null_mut()
    // }

    // extern "C" fn on_process_message_received(
    //     _client: *mut cef_client_t,
    //     _browser: *mut cef_browser_t,
    //     _frame: *mut cef_frame_t,
    //     _source_process: cef_process_id_t,
    //     _message: *mut cef_process_message_t,
    // ) -> ::std::os::raw::c_int {
    //     0
    // }
    extern "C" fn get_request_handler(client: *mut cef_client_t) -> *mut cef_request_handler_t {
        let client = Self::from_ptr(client);
        if let Some(handler) = client.internal.get_request_handler() {
            handler.to_cef()
        } else {
            null_mut()
        }
    }
}
impl<T: Client> ToCef<cef_client_t> for Arc<T> {
    fn to_cef(&self) -> *mut cef_client_t {
        wrap_ptr(|base| ClientWrapper {
            _base: cef_client_t {
                base,
                get_audio_handler: None,
                get_context_menu_handler: Some(ClientWrapper::<T>::get_context_menu_handler),
                get_dialog_handler: None,
                get_display_handler: None,
                get_download_handler: None,
                get_drag_handler: None,
                get_find_handler: None,
                get_focus_handler: None,
                get_jsdialog_handler: None,
                get_keyboard_handler: None,
                get_life_span_handler: None,
                get_load_handler: None,
                get_render_handler: None,
                get_request_handler: Some(ClientWrapper::<T>::get_request_handler),
                get_print_handler: None,
                on_process_message_received: None,
            },
            internal: self.clone(),
        })
    }
}
