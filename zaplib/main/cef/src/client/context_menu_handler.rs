use crate::ptr::{wrap_ptr, BaseRefCountedExt, WrapperFor};
use crate::{Browser, Frame, MenuModel, ToCef};
use std::sync::Arc;
use zaplib_cef_sys::{
    _cef_browser_t, _cef_context_menu_handler_t, _cef_context_menu_params_t, _cef_frame_t, _cef_menu_model_t,
    cef_context_menu_handler_t,
};

pub trait ContextMenuHandler {
    fn on_before_context_menu(
        &self,
        _browser: &Browser,
        _frame: &Frame,
        // _params: *mut _cef_context_menu_params_t,
        _model: &MenuModel,
    ) {
    }
    // TODO(Dmitry): implement other methods
    // #[doc = ""]
    // pub run_context_menu: ::std::option::Option<
    //     unsafe extern "C" fn(
    //         self_: *mut _cef_context_menu_handler_t,
    //         browser: *mut _cef_browser_t,
    //         frame: *mut _cef_frame_t,
    //         params: *mut _cef_context_menu_params_t,
    //         model: *mut _cef_menu_model_t,
    //         callback: *mut _cef_run_context_menu_callback_t,
    //     ) -> ::std::os::raw::c_int,
    // >,
    // #[doc = ""]
    // pub on_context_menu_command: ::std::option::Option<
    //     unsafe extern "C" fn(
    //         self_: *mut _cef_context_menu_handler_t,
    //         browser: *mut _cef_browser_t,
    //         frame: *mut _cef_frame_t,
    //         params: *mut _cef_context_menu_params_t,
    //         command_id: ::std::os::raw::c_int,
    //         event_flags: cef_event_flags_t,
    //     ) -> ::std::os::raw::c_int,
    // >,
    // #[doc = ""]
    // pub on_context_menu_dismissed: ::std::option::Option<
    //     unsafe extern "C" fn(self_: *mut _cef_context_menu_handler_t, browser: *mut _cef_browser_t, frame: *mut _cef_frame_t),
    // >,
}

impl ContextMenuHandler for () {}

struct ContextMenuHandlerWrapper<T: ContextMenuHandler> {
    _base: cef_context_menu_handler_t,
    internal: Arc<T>,
}
unsafe impl<T: ContextMenuHandler> WrapperFor<cef_context_menu_handler_t> for ContextMenuHandlerWrapper<T> {}
impl<T: ContextMenuHandler> ContextMenuHandlerWrapper<T> {
    fn from_ptr<'a>(
        ptr: *mut cef_context_menu_handler_t,
    ) -> &'a mut BaseRefCountedExt<cef_context_menu_handler_t, ContextMenuHandlerWrapper<T>> {
        unsafe { &mut *(ptr as *mut _) }
    }

    unsafe extern "C" fn on_before_context_menu(
        handler: *mut _cef_context_menu_handler_t,
        browser: *mut _cef_browser_t,
        frame: *mut _cef_frame_t,
        _params: *mut _cef_context_menu_params_t,
        model: *mut _cef_menu_model_t,
    ) {
        // TODO: support other params
        let handler = Self::from_ptr(handler);
        let browser = Browser::from(browser, false);
        let frame = Frame::from(frame, false);
        let model = MenuModel::from(model, false);
        handler.internal.on_before_context_menu(&browser, &frame, &model);
    }

    // unsafe extern "C" fn run_context_menu(
    //     _handler: *mut _cef_context_menu_handler_t,
    //     _browser: *mut _cef_browser_t,
    //     _frame: *mut _cef_frame_t,
    //     _params: *mut _cef_context_menu_params_t,
    //     _model: *mut _cef_menu_model_t,
    //     _callback: *mut _cef_run_context_menu_callback_t,
    // ) -> ::std::os::raw::c_int {
    //     // TODO
    //     0
    // }

    // unsafe extern "C" fn on_context_menu_command(
    //     _handler: *mut _cef_context_menu_handler_t,
    //     _browser: *mut _cef_browser_t,
    //     _frame: *mut _cef_frame_t,
    //     _params: *mut _cef_context_menu_params_t,
    //     _command_id: ::std::os::raw::c_int,
    //     _event_flags: cef_event_flags_t,
    // ) -> ::std::os::raw::c_int {
    //     // TODO
    //     0
    // }
    // unsafe extern "C" fn on_context_menu_dismissed(
    //     _handler: *mut _cef_context_menu_handler_t,
    //     _browser: *mut _cef_browser_t,
    //     _frame: *mut _cef_frame_t,
    // ) {
    //     // TODO
    // }
}

impl<T: ContextMenuHandler> ToCef<cef_context_menu_handler_t> for Arc<T> {
    fn to_cef(&self) -> *mut cef_context_menu_handler_t {
        wrap_ptr(|base| ContextMenuHandlerWrapper {
            _base: cef_context_menu_handler_t {
                base,
                on_before_context_menu: Some(ContextMenuHandlerWrapper::<T>::on_before_context_menu),
                run_context_menu: None,
                on_context_menu_command: None,
                on_context_menu_dismissed: None,
            },
            internal: self.clone(),
        })
    }
}
