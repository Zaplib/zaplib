#![cfg(not(target_arch = "wasm32"))]
// Clippy TODO
#![warn(clippy::all)]

mod app;
mod browser;
mod browser_host;
mod client;
mod frame;
mod menu_model;
mod platform;
mod process_message;
mod ptr;
mod settings;
mod thread;
mod types;
mod v8;
mod window;

pub use crate::types::string::CefString;
pub use app::*;
pub use browser::*;
pub use browser_host::*;
pub use client::*;
pub use frame::*;
pub use menu_model::*;
pub use platform::*;
pub use process_message::*;
pub use settings::*;
use std::ptr::null_mut;
use std::sync::Arc;
pub use thread::*;
pub use types::*;
pub use v8::*;
pub use window::*;

pub trait ToCef<T> {
    fn to_cef(&self) -> *mut T;
}

pub(crate) trait ToCefAsArg<T> {
    fn to_cef_as_arg(&self) -> *mut T;
}

pub fn execute_process_with_args<TApp: App>(
    args: CefArgs,
    application: &Arc<TApp>,
    //    windows_sandbox_info: *mut ::std::os::raw::c_void,
) -> i32 {
    let args = platform::args_to_cef(args);
    unsafe { zaplib_cef_sys::cef_execute_process(&args.cef, application.to_cef(), null_mut()) }
}
pub fn execute_process<TApp: App>(
    application: &Arc<TApp>,
    //    windows_sandbox_info: *mut ::std::os::raw::c_void,
) -> i32 {
    let args = platform::default_args();
    unsafe { zaplib_cef_sys::cef_execute_process(&args.cef, application.to_cef(), null_mut()) }
}

pub fn initialize_with_args<TApp: App>(
    args: CefArgs,
    settings: Settings,
    application: &Arc<TApp>,
    //    application: *mut zaplib_cef_sys::cef_app_t,
    //    windows_sandbox_info: *mut ::std::os::raw::c_void,
) -> ::std::os::raw::c_int {
    let args = platform::args_to_cef(args);
    unsafe { zaplib_cef_sys::cef_initialize(&args.cef, &settings.to_cef(), application.to_cef(), null_mut()) }
}

pub fn initialize<TApp: App>(
    settings: Settings,
    application: &Arc<TApp>,
    //    application: *mut zaplib_cef_sys::cef_app_t,
    //    windows_sandbox_info: *mut ::std::os::raw::c_void,
) -> ::std::os::raw::c_int {
    let args = platform::default_args();
    unsafe { zaplib_cef_sys::cef_initialize(&args.cef, &settings.to_cef(), application.to_cef(), null_mut()) }
}

pub fn create_browser_sync<TClient: Client>(
    info: WindowInfo,
    client: &Arc<TClient>,
    url: &str,
    settings: BrowserSettings,
) -> Browser {
    // TODO - calling this appears to leak a ref somewhere
    let res = unsafe {
        zaplib_cef_sys::cef_browser_host_create_browser_sync(
            &info.to_cef(),
            client.to_cef(),
            &CefString::from_str(url).into_cef(),
            &settings.to_cef(),
            null_mut(),
            null_mut(),
        )
    };
    Browser::from(res, true)
}

pub fn shutdown() {
    unsafe { zaplib_cef_sys::cef_shutdown() }
}

pub fn do_message_loop_work() {
    unsafe { zaplib_cef_sys::cef_do_message_loop_work() }
}

pub fn run_message_loop() {
    unsafe { zaplib_cef_sys::cef_run_message_loop() }
}

pub fn quit_message_loop() {
    unsafe { zaplib_cef_sys::cef_quit_message_loop() }
}
