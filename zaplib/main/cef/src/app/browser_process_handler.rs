use crate::ptr::{wrap_ptr, BaseRefCountedExt, WrapperFor};
use crate::ToCef;
use std::sync::Arc;
use zaplib_cef_sys::cef_browser_process_handler_t;

pub trait BrowserProcessHandler {
    // fn on_context_initialized(&self) {}
    // fn on_before_child_process_launch(&self, _command_line: &CommandLine) {}
    fn on_schedule_message_pump_work(&self, _delay_ms: i64) {}
}
impl BrowserProcessHandler for () {}

struct BrowserProcessHandlerWrapper<T: BrowserProcessHandler> {
    _base: cef_browser_process_handler_t,
    internal: Arc<T>,
}
unsafe impl<T: BrowserProcessHandler> WrapperFor<cef_browser_process_handler_t> for BrowserProcessHandlerWrapper<T> {}
impl<T: BrowserProcessHandler> BrowserProcessHandlerWrapper<T> {
    fn from_ptr<'a>(
        ptr: *mut cef_browser_process_handler_t,
    ) -> &'a mut BaseRefCountedExt<cef_browser_process_handler_t, BrowserProcessHandlerWrapper<T>> {
        unsafe { &mut *(ptr as *mut _) }
    }

    // unsafe extern "C" fn on_context_initialized(handler: *mut cef_browser_process_handler_t) {
    //     let handler = Self::from_ptr(handler);

    //     handler.internal.on_context_initialized()
    // }

    // unsafe extern "C" fn on_before_child_process_launch(
    //     handler: *mut cef_browser_process_handler_t,
    //     command_line: *mut cef_command_line_t,
    // ) {
    //     let handler = Self::from_ptr(handler);
    //     let command_line = CommandLine::from(command_line, false);

    //     handler.internal.on_before_child_process_launch(&command_line);
    // }

    unsafe extern "C" fn on_schedule_message_pump_work(handler: *mut cef_browser_process_handler_t, delay_ms: i64) {
        let handler = Self::from_ptr(handler);

        handler.internal.on_schedule_message_pump_work(delay_ms)
    }

    // unsafe extern "C" fn get_default_client(_handler: *mut cef_browser_process_handler_t) -> *mut cef_client_t {
    //     let handler = Self::from_ptr(handler);

    //     null_mut()
    // }
}
impl<T: BrowserProcessHandler> ToCef<cef_browser_process_handler_t> for Arc<T> {
    fn to_cef(&self) -> *mut cef_browser_process_handler_t {
        wrap_ptr(|base| BrowserProcessHandlerWrapper {
            _base: cef_browser_process_handler_t {
                base,
                on_context_initialized: None,
                on_before_child_process_launch: None,
                get_default_client: None,
                on_schedule_message_pump_work: Some(BrowserProcessHandlerWrapper::<T>::on_schedule_message_pump_work),
            },
            internal: self.clone(),
        })
    }
}
