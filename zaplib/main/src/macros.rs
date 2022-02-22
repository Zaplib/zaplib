//! Various macros.

/// Define the entry point of your application.
///
/// Your `$app` should implement the `draw` and `handle`
/// functions. Refer to the examples to get started.
#[macro_export]
macro_rules! main_app {
    ( $ app: ident) => {
        #[cfg(not(target_arch = "wasm32"))]
        fn main() {
            //TODO do this with a macro to generate both entrypoints for App and Cx
            let mut cx = Cx::new(std::any::TypeId::of::<$app>());
            let mut app = $app::new(&mut cx);
            let mut cxafterdraw = CxAfterDraw::new(&mut cx);
            cx.event_loop(|cx, mut event| {
                match event {
                    Event::System(e) => {
                        match e {
                            SystemEvent::Draw => {
                                app.draw(cx);
                                cxafterdraw.after_draw(cx);
                            }
                            SystemEvent::WebRustCall(e) => {
                                let WebRustCallEvent { name, params, callback_id } = std::mem::take(e).unwrap();
                                let call_rust_fn = cx.call_rust_fn.expect("call_rust called but no on_call_rust registered");
                                unsafe {
                                    let func = Box::from_raw(
                                        call_rust_fn
                                            as *mut fn(
                                                this: &mut $app,
                                                cx: &mut Cx,
                                                name: String,
                                                params: Vec<ZapParam>,
                                            ) -> Vec<ZapParam>,
                                    );
                                    let mut return_params = func(&mut app, cx, name, params);

                                    // Prevent call_rust_fn from getting dropped
                                    Box::into_raw(func);

                                    cx.return_to_js(callback_id, return_params);
                                }
                            }
                            _ => {}
                        }
                    }
                    _ => {
                        app.handle(cx, &mut event);
                    }
                };
            });
        }

        #[cfg(target_arch = "wasm32")]
        fn main() {}

        #[cfg(target_arch = "wasm32")]
        #[export_name = "createWasmApp"]
        pub extern "C" fn create_wasm_app() -> u64 {
            Cx::init_error_handlers();
            let mut cx = Box::new(Cx::new(std::any::TypeId::of::<$app>()));
            let app = Box::new($app::new(&mut cx));
            let cxafterdraw = Box::new(CxAfterDraw::new(&mut cx));
            Box::into_raw(Box::new((Box::into_raw(app), Box::into_raw(cx), Box::into_raw(cxafterdraw)))) as u64
        }

        #[cfg(target_arch = "wasm32")]
        #[export_name = "processWasmEvents"]
        pub unsafe extern "C" fn process_wasm_events(appcx: u64, msg_bytes: u64) -> u64 {
            let appcx = &*(appcx as *mut (*mut $app, *mut Cx, *mut CxAfterDraw));
            (*appcx.1).process_wasm_events(msg_bytes, |cx, mut event| {
                match event {
                    Event::System(e) => {
                        match e {
                            SystemEvent::Draw => {
                                (*appcx.0).draw(cx);
                                (*appcx.2).after_draw(cx);
                            }
                            SystemEvent::WebRustCall(e) => {
                                let WebRustCallEvent { name, params, callback_id } = std::mem::take(e).unwrap();

                                let call_rust_fn = cx.call_rust_fn.expect("call_rust called but no on_call_rust registered");
                                let func = Box::from_raw(
                                    call_rust_fn
                                        as *mut fn(
                                            this: &mut $app,
                                            cx: &mut Cx,
                                            name: String,
                                            params: Vec<ZapParam>,
                                        ) -> Vec<ZapParam>,
                                );
                                let mut return_params = func(&mut *appcx.0, cx, name, params);
                                // Prevent call_rust_fn from getting dropped
                                Box::into_raw(func);

                                cx.return_to_js(callback_id, return_params);
                            }
                            _ => {}
                        }
                    }
                    _ => {
                        (*appcx.0).handle(cx, event);
                    }
                }
            })
        }

        #[cfg(all(target_arch = "wasm32"))]
        #[export_name = "callRustInSameThreadSync"]
        pub unsafe extern "C" fn call_rust_in_same_thread_sync(appcx: u64, zerde_ptr: u64) -> u64 {
            let appcx = &*(appcx as *mut (*mut $app, *mut Cx, *mut CxAfterDraw));
            (*appcx.1).call_rust_in_same_thread_sync(zerde_ptr)
        }
    };
}

/// Define state-less entry point
///
/// This can be used when application state and canvas drawing are not needed,
/// but instead just JavaScript to Rust communications.
/// TODO(Paras): This is a bit of hack right now, since we end up instantiating a lot
/// of Zaplib features that end up unused. We should split out the framework in the future
/// so that thisuse case does not initialize unused parts of Zaplib as well as JavaScript
/// mouse and keyboard event handlers.
#[macro_export]
macro_rules! register_call_rust {
    ( $ call_rust: ident) => {
        struct App {}
        impl App {
            fn new(cx: &mut Cx) -> Self {
                cx.on_call_rust(Self::on_call_rust);
                cx.on_call_rust_in_same_thread_sync(Self::call_rust_in_same_thread_sync);
                Self {}
            }

            fn handle(&mut self, cx: &mut Cx, event: &mut Event) {}

            fn on_call_rust(&mut self, cx: &mut Cx, name: String, params: Vec<ZapParam>) -> Vec<ZapParam> {
                call_rust(name, params)
            }

            fn call_rust_in_same_thread_sync(name: &str, params: Vec<ZapParam>) -> Vec<ZapParam> {
                call_rust(name.to_string(), params)
            }

            fn draw(&mut self, cx: &mut Cx) {}
        }

        main_app!(App);
    };
}

/// Generates a [`crate::hash::LocationHash`] based on the current file/line/column.
#[macro_export]
macro_rules! location_hash {
    () => {
        LocationHash::new(file!(), line!() as u64, column!() as u64)
    };
}

/// Tag a piece of code with filename+line+col. The line+col are done in a hacky
/// way, exploiting the fact that rustfmt usually puts the multiline string
/// start on a newline, with one greater indentation level. This doesn't always
/// work but it's close enough!
///
/// We could at some point use something like this:
/// <https://github.com/makepad/makepad/blob/719012b9348815acdcaec6365a99443b9208aecc/main/live_body/src/lib.rs#L47>
/// But that depends on the nightly `proc_macro_span` feature (<https://github.com/rust-lang/rust/issues/54725>).
#[macro_export]
macro_rules! code_fragment {
    ( $ code: expr ) => {
        CodeFragment::Static { filename: file!(), line: line!() as usize + 1, col: column!() as usize + 7, code: $code }
    };
}

/// Logging helper that works both on native and WebAssembly targets.
#[cfg(not(target_arch = "wasm32"))]
#[macro_export]
macro_rules! log {
    ( $ ( $t: tt) *) => {{
        println!("{}:{} - {}",file!(),line!(),format!($($t)*));
        std::io::Write::flush(&mut std::io::stdout()).unwrap();
    }}
}

/// Logging helper that works both on native and WebAssembly targets.
///
/// TODO(JP): Would be better to integrate this with the normal println! dbg! etc macros,
/// so we don't need special treatment.
#[cfg(target_arch = "wasm32")]
#[macro_export]
macro_rules! log {
    ( $ ( $ t: tt) *) => {
        console_log(&format!("{}:{} - {}", file!(), line!(), format!( $ ( $ t) *)))
    }
}
