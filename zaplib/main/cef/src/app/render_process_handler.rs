use crate::ptr::{wrap_ptr, BaseRefCountedExt, WrapperFor};
use crate::{Browser, Frame, ProcessId, ProcessMessage, ToCef, V8Context};
use std::sync::Arc;
use zaplib_cef_sys::{
    cef_browser_t, cef_frame_t, cef_process_id_t, cef_process_message_t, cef_render_process_handler_t, cef_v8context_t,
};

pub trait RenderProcessHandler {
    // fn on_web_kit_initialized(&self) {}
    // fn on_browser_created(&self, _browser: &Browser, _extra_info: *mut cef_dictionary_value_t) {}
    // fn on_browser_destroyed(&self, _browser: &Browser) {}
    // fn get_load_handler(&self) -> *mut cef_load_handler_t {
    //     null_mut()
    // }
    fn on_context_created(&self, _browser: &Browser, _frame: &Frame, _context: &V8Context) {}
    // fn on_context_released(&self, _browser: &Browser, _frame: &Frame, _context: &V8Context) {}
    // fn on_uncaught_exception(
    //     &self,
    //     _browser: &Browser,
    //     _frame: &Frame,
    //     _context: &V8Context,
    //     _exception: *mut cef_v8exception_t,
    //     _stack_trace: *mut cef_v8stack_trace_t,
    // ) {
    // }
    // fn on_focused_node_changed(&self, _browser: &Browser, _frame: &Frame, _node: *mut cef_domnode_t) {}
    fn on_process_message_received(
        &self,
        _browser: &Browser,
        _frame: &Frame,
        _source_process: ProcessId,
        _message: &ProcessMessage,
    ) -> bool {
        false
    }
}
impl RenderProcessHandler for () {}

struct RenderProcessHandlerWrapper<T: RenderProcessHandler> {
    _base: cef_render_process_handler_t,
    internal: Arc<T>,
}
unsafe impl<T: RenderProcessHandler> WrapperFor<cef_render_process_handler_t> for RenderProcessHandlerWrapper<T> {}
impl<T: RenderProcessHandler> RenderProcessHandlerWrapper<T> {
    fn from_ptr<'a>(
        ptr: *mut cef_render_process_handler_t,
    ) -> &'a mut BaseRefCountedExt<cef_render_process_handler_t, RenderProcessHandlerWrapper<T>> {
        unsafe { &mut *(ptr as *mut _) }
    }

    // unsafe extern "C" fn on_web_kit_initialized(handler: *mut cef_render_process_handler_t) {
    //     let handler = Self::from_ptr(handler);

    //     handler.internal.on_web_kit_initialized()
    // }

    // unsafe extern "C" fn on_browser_created(
    //     handler: *mut cef_render_process_handler_t,
    //     browser: *mut cef_browser_t,
    //     extra_info: *mut cef_dictionary_value_t,
    // ) {
    //     let handler = Self::from_ptr(handler);
    //     let browser = Browser::from(browser, false);
    //     handler.internal.on_browser_created(&browser, extra_info);
    // }

    // unsafe extern "C" fn on_browser_destroyed(handler: *mut cef_render_process_handler_t, browser: *mut cef_browser_t) {
    //     let handler = Self::from_ptr(handler);
    //     let browser = Browser::from(browser, false);
    //     handler.internal.on_browser_destroyed(&browser);
    // }

    // unsafe extern "C" fn get_load_handler(handler: *mut cef_render_process_handler_t) -> *mut cef_load_handler_t {
    //     let handler = Self::from_ptr(handler);
    //     handler.internal.get_load_handler()
    // }

    unsafe extern "C" fn on_context_created(
        handler: *mut cef_render_process_handler_t,
        browser: *mut cef_browser_t,
        frame: *mut cef_frame_t,
        context: *mut cef_v8context_t,
    ) {
        let handler = Self::from_ptr(handler);
        let browser = Browser::from(browser, false);
        let frame = Frame::from(frame, false);
        let context = V8Context::from(context, false);
        handler.internal.on_context_created(&browser, &frame, &context);
    }

    // unsafe extern "C" fn on_context_released(
    //     handler: *mut cef_render_process_handler_t,
    //     browser: *mut cef_browser_t,
    //     frame: *mut cef_frame_t,
    //     context: *mut cef_v8context_t,
    // ) {
    //     let handler = Self::from_ptr(handler);
    //     let browser = Browser::from(browser, false);
    //     let frame = Frame::from(frame, false);
    //     let context = V8Context::from(context, false);
    //     handler.internal.on_context_released(&browser, &frame, &context);
    // }

    // unsafe extern "C" fn on_uncaught_exception(
    //     handler: *mut cef_render_process_handler_t,
    //     browser: *mut cef_browser_t,
    //     frame: *mut cef_frame_t,
    //     context: *mut cef_v8context_t,
    //     exception: *mut cef_v8exception_t,
    //     stack_trace: *mut cef_v8stack_trace_t,
    // ) {
    //     let handler = Self::from_ptr(handler);
    //     let browser = Browser::from(browser, false);
    //     let frame = Frame::from(frame, false);
    //     let context = V8Context::from(context, false);
    //     handler.internal.on_uncaught_exception(&browser, &frame, &context, exception, stack_trace);
    // }
    // unsafe extern "C" fn on_focused_node_changed(
    //     handler: *mut cef_render_process_handler_t,
    //     browser: *mut cef_browser_t,
    //     frame: *mut cef_frame_t,
    //     node: *mut cef_domnode_t,
    // ) {
    //     let handler = Self::from_ptr(handler);
    //     let browser = Browser::from(browser, false);
    //     let frame = Frame::from(frame, false);
    //     handler.internal.on_focused_node_changed(&browser, &frame, node);
    // }

    unsafe extern "C" fn on_process_message_received(
        handler: *mut cef_render_process_handler_t,
        browser: *mut cef_browser_t,
        frame: *mut cef_frame_t,
        source_process: cef_process_id_t,
        message: *mut cef_process_message_t,
    ) -> i32 {
        let handler = Self::from_ptr(handler);
        let browser = Browser::from(browser, false);
        let frame = Frame::from(frame, false);
        let message = ProcessMessage::from(message, false);
        handler.internal.on_process_message_received(&browser, &frame, source_process, &message) as i32
    }
}
impl<T: RenderProcessHandler> ToCef<cef_render_process_handler_t> for Arc<T> {
    fn to_cef(&self) -> *mut cef_render_process_handler_t {
        wrap_ptr(|base| RenderProcessHandlerWrapper {
            _base: cef_render_process_handler_t {
                base,
                on_web_kit_initialized: None,
                on_browser_created: None,
                on_browser_destroyed: None,
                get_load_handler: None,
                on_context_created: Some(RenderProcessHandlerWrapper::<T>::on_context_created),
                on_context_released: None,
                on_uncaught_exception: None,
                on_focused_node_changed: None,
                on_process_message_received: Some(RenderProcessHandlerWrapper::<T>::on_process_message_received),
            },
            internal: self.clone(),
        })
    }
}
