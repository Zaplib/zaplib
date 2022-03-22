//! WebAssembly platform-specific entry point.

use crate::cx_web::*;
use crate::universal_file::UniversalFile;
use crate::zerde::*;
use crate::*;
use std::alloc;
use std::cell::UnsafeCell;
use std::collections::{BTreeSet, HashMap};
use std::mem;
use std::ptr;
use std::sync::Arc;

// These constants must be kept in sync with the ones in web/zerde_eventloop_events.ts
const MSG_TYPE_END: u32 = 0;
const MSG_TYPE_INIT: u32 = 1;
const MSG_TYPE_RESIZE: u32 = 4;
const MSG_TYPE_ANIMATION_FRAME: u32 = 5;
const MSG_TYPE_POINTER_DOWN: u32 = 6;
const MSG_TYPE_POINTER_UP: u32 = 7;
const MSG_TYPE_POINTER_MOVE: u32 = 8;
const MSG_TYPE_POINTER_HOVER: u32 = 9;
const MSG_TYPE_POINTER_SCROLL: u32 = 10;
const MSG_TYPE_POINTER_OUT: u32 = 11;
const MSG_TYPE_KEY_DOWN: u32 = 12;
const MSG_TYPE_KEY_UP: u32 = 13;
const MSG_TYPE_TEXT_INPUT: u32 = 14;
const MSG_TYPE_TEXT_COPY: u32 = 17;
const MSG_TYPE_TIMER_FIRED: u32 = 18;
const MSG_TYPE_WINDOW_FOCUS: u32 = 19;
const MSG_TYPE_XR_UPDATE: u32 = 20;
const MSG_TYPE_PAINT_DIRTY: u32 = 21;
const MSG_TYPE_HTTP_SEND_RESPONSE: u32 = 22;
const MSG_TYPE_WEBSOCKET_MESSAGE: u32 = 23;
const MSG_TYPE_WEBSOCKET_ERROR: u32 = 24;
const MSG_TYPE_APP_OPEN_FILES: u32 = 25;
const MSG_TYPE_SEND_EVENT_FROM_ANY_THREAD: u32 = 26;
const MSG_TYPE_DRAG_ENTER: u32 = 27;
const MSG_TYPE_DRAG_LEAVE: u32 = 28;
const MSG_TYPE_DRAG_OVER: u32 = 29;
const MSG_TYPE_CALL_RUST: u32 = 30;

impl Cx {
    /// Initialize global error handlers.
    pub fn init_error_handlers() {
        std::alloc::set_alloc_error_hook(|layout: std::alloc::Layout| {
            throw_error("Allocation error! Printing the layout on the next line...");
            // Printing this separately, since it will do an allocation itself and so might fail!
            throw_error(&format!("Allocation layout: {:?}", layout));
        });
        std::panic::set_hook(Box::new(|info: &std::panic::PanicInfo| {
            throw_error(&info.to_string());
        }));
    }

    pub fn process_wasm_events<F>(&mut self, msg: u64, mut event_handler: F) -> u64
    where
        F: FnMut(&mut Cx, &mut Event),
    {
        self.event_handler =
            Some(&mut event_handler as *const dyn FnMut(&mut Cx, &mut Event) as *mut dyn FnMut(&mut Cx, &mut Event));
        let ret = self.event_loop_core(msg);
        self.event_handler = None;
        ret
    }

    /// Incoming Zerde buffer. There is no other entrypoint to general rust codeflow than this function,
    /// only allocators and init. Note that we do have other outgoing functions for synchronous
    /// operations.
    fn event_loop_core(&mut self, msg: u64) -> u64 {
        let mut zerde_parser = ZerdeParser::from(msg);
        self.last_event_time = zerde_parser.parse_f64();
        let mut is_animation_frame = false;
        loop {
            let msg_type = zerde_parser.parse_u32();
            match msg_type {
                MSG_TYPE_END => {
                    break;
                }
                MSG_TYPE_INIT => {
                    assert!(!self.platform.is_initialized);
                    self.platform.is_initialized = true;
                    for _i in 0..10 {
                        self.platform.pointers_down.push(false);
                    }

                    self.platform.window_geom = WindowGeom {
                        is_fullscreen: false,
                        is_topmost: false,
                        inner_size: Vec2 { x: zerde_parser.parse_f32(), y: zerde_parser.parse_f32() },
                        dpi_factor: zerde_parser.parse_f32(),
                        outer_size: Vec2 { x: 0., y: 0. },
                        position: Vec2 { x: 0., y: 0. },
                        xr_is_presenting: false,
                        xr_can_present: zerde_parser.parse_u32() > 0,
                        can_fullscreen: zerde_parser.parse_u32() > 0,
                    };

                    let js_git_sha = zerde_parser.parse_string();
                    // If a JS dev build was used; ignore this check.
                    if js_git_sha != "development" {
                        // If Rust build couldn't find .git directory (e.g. on Heroku); ignore this check.
                        let rust_git_sha = option_env!("VERGEN_GIT_SHA");
                        if let Some(rust_git_sha) = rust_git_sha {
                            if js_git_sha != rust_git_sha {
                                panic!("JS git sha ({js_git_sha}) doesn't match Rust git sha ({rust_git_sha})");
                            }
                        }
                    }

                    self.default_dpi_factor = self.platform.window_geom.dpi_factor;
                    assert!(self.default_dpi_factor > 0.0);

                    if self.windows.len() > 0 {
                        self.windows[0].window_geom = self.platform.window_geom.clone();
                    }

                    self.load_fonts();

                    self.wasm_event_handler(Event::Construct);

                    self.request_draw();
                }
                MSG_TYPE_RESIZE => {
                    let old_geom = self.platform.window_geom.clone();
                    self.platform.window_geom = WindowGeom {
                        is_topmost: false,
                        inner_size: Vec2 { x: zerde_parser.parse_f32(), y: zerde_parser.parse_f32() },
                        dpi_factor: zerde_parser.parse_f32(),
                        outer_size: Vec2 { x: 0., y: 0. },
                        position: Vec2 { x: 0., y: 0. },
                        xr_is_presenting: zerde_parser.parse_u32() > 0,
                        xr_can_present: zerde_parser.parse_u32() > 0,
                        is_fullscreen: zerde_parser.parse_u32() > 0,
                        can_fullscreen: old_geom.can_fullscreen,
                    };
                    assert!(self.platform.window_geom.dpi_factor > 0.0);
                    let new_geom = self.platform.window_geom.clone();

                    if self.windows.len() > 0 {
                        self.windows[0].window_geom = self.platform.window_geom.clone();
                    }
                    if old_geom != new_geom {
                        self.wasm_event_handler(Event::WindowGeomChange(WindowGeomChangeEvent {
                            window_id: 0,
                            old_geom,
                            new_geom,
                        }));
                    }

                    // do our initial redraw and repaint
                    self.request_draw();
                }
                MSG_TYPE_ANIMATION_FRAME => {
                    is_animation_frame = true;
                    if self.requested_next_frame {
                        self.call_next_frame_event();
                    }
                }
                MSG_TYPE_POINTER_DOWN => {
                    let abs = Vec2 { x: zerde_parser.parse_f32(), y: zerde_parser.parse_f32() };
                    let button = zerde_parser.parse_u32() as usize;
                    let digit = zerde_parser.parse_u32() as usize;
                    self.platform.pointers_down[digit] = true;
                    let is_touch = zerde_parser.parse_u32() > 0;
                    let modifiers = unpack_key_modifier(zerde_parser.parse_u32());
                    let time = zerde_parser.parse_f64();
                    self.wasm_event_handler(Event::PointerDown(PointerDownEvent {
                        window_id: 0,
                        abs,
                        rel: abs,
                        rect: Rect::default(),
                        handled: false,
                        digit,
                        button: get_mouse_button(button),
                        input_type: if is_touch { PointerInputType::Touch } else { PointerInputType::Mouse },
                        modifiers,
                        time,
                        tap_count: 0,
                    }));
                }
                MSG_TYPE_POINTER_UP => {
                    let abs = Vec2 { x: zerde_parser.parse_f32(), y: zerde_parser.parse_f32() };
                    let button = zerde_parser.parse_u32() as usize;
                    let digit = zerde_parser.parse_u32() as usize;
                    self.platform.pointers_down[digit] = false;
                    let is_touch = zerde_parser.parse_u32() > 0;
                    let modifiers = unpack_key_modifier(zerde_parser.parse_u32());
                    let time = zerde_parser.parse_f64();
                    self.wasm_event_handler(Event::PointerUp(PointerUpEvent {
                        window_id: 0,
                        abs,
                        rel: abs,
                        rect: Rect::default(),
                        abs_start: Vec2::default(),
                        rel_start: Vec2::default(),
                        digit,
                        button: get_mouse_button(button),
                        is_over: false,
                        input_type: if is_touch { PointerInputType::Touch } else { PointerInputType::Mouse },
                        modifiers,
                        time,
                    }));
                }
                MSG_TYPE_POINTER_MOVE => {
                    let abs = Vec2 { x: zerde_parser.parse_f32(), y: zerde_parser.parse_f32() };
                    let digit = zerde_parser.parse_u32() as usize;
                    let is_touch = zerde_parser.parse_u32() > 0;
                    let modifiers = unpack_key_modifier(zerde_parser.parse_u32());
                    let time = zerde_parser.parse_f64();
                    self.wasm_event_handler(Event::PointerMove(PointerMoveEvent {
                        window_id: 0,
                        abs,
                        rel: abs,
                        rect: Rect::default(),
                        abs_start: Vec2::default(),
                        rel_start: Vec2::default(),
                        is_over: false,
                        digit,
                        input_type: if is_touch { PointerInputType::Touch } else { PointerInputType::Mouse },
                        modifiers,
                        time,
                    }));
                }
                MSG_TYPE_POINTER_HOVER => {
                    let abs = Vec2 { x: zerde_parser.parse_f32(), y: zerde_parser.parse_f32() };
                    let modifiers = unpack_key_modifier(zerde_parser.parse_u32());
                    let time = zerde_parser.parse_f64();
                    self.wasm_event_handler(Event::PointerHover(PointerHoverEvent {
                        any_down: false,
                        digit: 0,
                        window_id: 0,
                        abs,
                        rel: abs,
                        rect: Rect::default(),
                        handled: false,
                        hover_state: HoverState::Over,
                        modifiers,
                        time,
                    }));
                }
                MSG_TYPE_POINTER_SCROLL => {
                    let abs = Vec2 { x: zerde_parser.parse_f32(), y: zerde_parser.parse_f32() };
                    let scroll = Vec2 { x: zerde_parser.parse_f32(), y: zerde_parser.parse_f32() };
                    let is_wheel = zerde_parser.parse_u32() != 0;
                    let modifiers = unpack_key_modifier(zerde_parser.parse_u32());
                    let time = zerde_parser.parse_f64();
                    self.wasm_event_handler(Event::PointerScroll(PointerScrollEvent {
                        window_id: 0,
                        digit: 0,
                        abs,
                        rel: abs,
                        rect: Rect::default(),
                        handled_x: false,
                        handled_y: false,
                        scroll,
                        input_type: if is_wheel { PointerInputType::Mouse } else { PointerInputType::Touch },
                        modifiers,
                        time,
                    }));
                }
                MSG_TYPE_POINTER_OUT => {
                    let abs = Vec2 { x: zerde_parser.parse_f32(), y: zerde_parser.parse_f32() };
                    let modifiers = unpack_key_modifier(zerde_parser.parse_u32());
                    let time = zerde_parser.parse_f64();
                    self.wasm_event_handler(Event::PointerHover(PointerHoverEvent {
                        window_id: 0,
                        digit: 0,
                        any_down: false,
                        abs,
                        rel: abs,
                        rect: Rect::default(),
                        handled: false,
                        hover_state: HoverState::Out,
                        modifiers,
                        time,
                    }));
                }
                MSG_TYPE_KEY_DOWN | MSG_TYPE_KEY_UP | MSG_TYPE_TEXT_INPUT | MSG_TYPE_TEXT_COPY => {
                    let event = parse_keyboard_event_from_js(msg_type, &mut zerde_parser);
                    self.wasm_event_handler(event);
                }
                MSG_TYPE_TIMER_FIRED => {
                    let timer_id = zerde_parser.parse_f64() as u64;
                    self.wasm_event_handler(Event::Timer(TimerEvent { timer_id }));
                }
                MSG_TYPE_WINDOW_FOCUS => {
                    let focus = zerde_parser.parse_u32();
                    if focus == 0 {
                        self.wasm_event_handler(Event::AppFocusLost);
                    } else {
                        self.wasm_event_handler(Event::AppFocus);
                    }
                }
                MSG_TYPE_XR_UPDATE => {
                    // xr_update, TODO(JP): bring this back some day?
                    // let inputs_len = zerde_parser.parse_u32();
                    // let time = zerde_parser.parse_f64();
                    // let head_transform = zerde_parser.parse_transform();
                    // let mut left_input = XRInput::default();
                    // let mut right_input = XRInput::default();
                    // let mut other_inputs = Vec::new();
                    // for _ in 0..inputs_len {
                    //     let skip = zerde_parser.parse_u32();
                    //     if skip == 0 {
                    //         continue;
                    //     }
                    //     let mut input = XRInput::default();
                    //     input.active = true;
                    //     input.grip = zerde_parser.parse_transform();
                    //     input.ray = zerde_parser.parse_transform();

                    //     let hand = zerde_parser.parse_u32();
                    //     let num_buttons = zerde_parser.parse_u32() as usize;
                    //     input.num_buttons = num_buttons;
                    //     for i in 0..num_buttons {
                    //         input.buttons[i].pressed = zerde_parser.parse_u32() > 0;
                    //         input.buttons[i].value = zerde_parser.parse_f32();
                    //     }

                    //     let num_axes = zerde_parser.parse_u32() as usize;
                    //     input.num_axes = num_axes;
                    //     for i in 0..num_axes {
                    //         input.axes[i] = zerde_parser.parse_f32();
                    //     }

                    //     if hand == 1 {
                    //         left_input = input;
                    //     } else if hand == 2 {
                    //         right_input = input;
                    //     } else {
                    //         other_inputs.push(input);
                    //     }
                    // }
                    // // call the VRUpdate event
                    // self.wasm_event_handler(&mut Event::XRUpdate(XRUpdateEvent {
                    //     time,
                    //     head_transform,
                    //     last_left_input: self.platform.xr_last_left_input.clone(),
                    //     last_right_input: self.platform.xr_last_right_input.clone(),
                    //     left_input: left_input.clone(),
                    //     right_input: right_input.clone(),
                    //     other_inputs,
                    // }));

                    // self.platform.xr_last_left_input = left_input;
                    // self.platform.xr_last_right_input = right_input;
                }
                MSG_TYPE_PAINT_DIRTY => {
                    // paint_dirty, only set the passes of the main window to dirty
                    self.passes[self.windows[0].main_pass_id.unwrap()].paint_dirty = true;
                }
                MSG_TYPE_HTTP_SEND_RESPONSE => {
                    let signal_id = zerde_parser.parse_u32();
                    let success = zerde_parser.parse_u32();
                    let mut new_set = BTreeSet::new();
                    new_set.insert(match success {
                        1 => Cx::STATUS_HTTP_SEND_OK,
                        _ => Cx::STATUS_HTTP_SEND_FAIL,
                    });
                    self.signals.insert(Signal { signal_id: signal_id as usize }, new_set);
                }
                MSG_TYPE_WEBSOCKET_MESSAGE => {
                    let data = zerde_parser.parse_vec_ptr();
                    let url = zerde_parser.parse_string();
                    self.wasm_event_handler(Event::WebSocketMessage(WebSocketMessageEvent { url, result: Ok(data) }));
                }
                MSG_TYPE_WEBSOCKET_ERROR => {
                    let url = zerde_parser.parse_string();
                    let err = zerde_parser.parse_string();
                    self.wasm_event_handler(Event::WebSocketMessage(WebSocketMessageEvent { url, result: Err(err) }));
                }
                MSG_TYPE_APP_OPEN_FILES => {
                    let len = zerde_parser.parse_u32();
                    let user_files: Vec<UserFile> = (0..len)
                        .map(|_| {
                            let id = zerde_parser.parse_u32() as usize;
                            let size = zerde_parser.parse_u64();
                            let basename = zerde_parser.parse_string();

                            UserFile { basename, file: UniversalFile::from_wasm_file(id, size) }
                        })
                        .collect();
                    self.wasm_event_handler(Event::AppOpenFiles(AppOpenFilesEvent { user_files }));
                }
                MSG_TYPE_SEND_EVENT_FROM_ANY_THREAD => {
                    let event_ptr = zerde_parser.parse_u64();
                    let event_box = unsafe { Box::from_raw(event_ptr as *mut Event) };
                    self.wasm_event_handler(*event_box);
                }
                MSG_TYPE_DRAG_ENTER => {
                    self.wasm_event_handler(Event::FileDragBegin);
                }
                MSG_TYPE_DRAG_LEAVE => {
                    self.wasm_event_handler(Event::FileDragCancel);
                }
                MSG_TYPE_DRAG_OVER => {
                    let x = zerde_parser.parse_u32() as f32;
                    let y = zerde_parser.parse_u32() as f32;

                    self.wasm_event_handler(Event::FileDragUpdate(FileDragUpdateEvent { abs: Vec2 { x, y } }));
                }
                MSG_TYPE_CALL_RUST => {
                    let name = zerde_parser.parse_string();
                    let params = zerde_parser.parse_zap_params();
                    let callback_id = zerde_parser.parse_u32();
                    self.wasm_event_handler(Event::System(SystemEvent::WebRustCall(Some(WebRustCallEvent {
                        name,
                        params,
                        callback_id,
                    }))));
                }
                _ => {
                    panic!("Message unknown {}", msg_type);
                }
            };
        }
        assert!(self.platform.is_initialized);

        self.call_signals();

        if is_animation_frame && self.requested_draw {
            self.call_draw_event();
        }
        self.call_signals();

        for window in &mut self.windows {
            window.window_state = match &window.window_state {
                CxWindowState::Create { title, add_drop_target_for_app_open_files, .. } => {
                    self.platform.zerde_eventloop_msgs.set_document_title(&title);
                    window.window_geom = self.platform.window_geom.clone();

                    if *add_drop_target_for_app_open_files {
                        self.platform.zerde_eventloop_msgs.enable_global_file_drop_target();
                    }

                    CxWindowState::Created
                }
                CxWindowState::Close => CxWindowState::Closed,
                CxWindowState::Created => CxWindowState::Created,
                CxWindowState::Closed => CxWindowState::Closed,
            };

            window.window_command = match &window.window_command {
                CxWindowCmd::XrStartPresenting => {
                    self.platform.zerde_eventloop_msgs.xr_start_presenting();
                    CxWindowCmd::None
                }
                CxWindowCmd::XrStopPresenting => {
                    self.platform.zerde_eventloop_msgs.xr_stop_presenting();
                    CxWindowCmd::None
                }
                CxWindowCmd::FullScreen => {
                    self.platform.zerde_eventloop_msgs.fullscreen();
                    CxWindowCmd::None
                }
                CxWindowCmd::NormalScreen => {
                    self.platform.zerde_eventloop_msgs.normalscreen();
                    CxWindowCmd::None
                }
                _ => CxWindowCmd::None,
            };
        }

        // check if we need to send a cursor
        if !self.down_mouse_cursor.is_none() {
            self.platform.zerde_eventloop_msgs.set_mouse_cursor(self.down_mouse_cursor.as_ref().unwrap().clone())
        } else if !self.hover_mouse_cursor.is_none() {
            self.platform.zerde_eventloop_msgs.set_mouse_cursor(self.hover_mouse_cursor.as_ref().unwrap().clone())
        } else {
            self.platform.zerde_eventloop_msgs.set_mouse_cursor(MouseCursor::Default);
        }

        let mut passes_todo = Vec::new();
        let mut windows_need_repaint = 0;
        self.compute_passes_to_repaint(&mut passes_todo, &mut windows_need_repaint);

        if is_animation_frame && passes_todo.len() > 0 {
            let mut zerde_webgl = ZerdeWebGLMessages::new();
            self.webgl_compile_shaders(&mut zerde_webgl);
            for pass_id in &passes_todo {
                match self.passes[*pass_id].dep_of.clone() {
                    CxPassDepOf::Window(_) => {
                        // find the accompanying render window
                        // its a render window
                        windows_need_repaint -= 1;
                        let dpi_factor = self.platform.window_geom.dpi_factor;
                        self.draw_pass_to_canvas(*pass_id, dpi_factor, &mut zerde_webgl);
                    }
                    CxPassDepOf::Pass(parent_pass_id) => {
                        let dpi_factor = self.get_delegated_dpi_factor(parent_pass_id);
                        self.draw_pass_to_texture(*pass_id, dpi_factor, &mut zerde_webgl);
                    }
                    CxPassDepOf::None => {
                        self.draw_pass_to_texture(*pass_id, 1.0, &mut zerde_webgl);
                    }
                }
            }
            zerde_webgl.end();
            self.platform.zerde_eventloop_msgs.run_webgl(zerde_webgl.take_ptr());
        }

        // request animation frame if still need to redraw, or repaint
        // we use request animation frame for that.
        if passes_todo.len() != 0 || self.requested_draw || self.requested_next_frame {
            self.platform.zerde_eventloop_msgs.request_animation_frame();
        }

        // mark the end of the message
        self.platform.zerde_eventloop_msgs.end();

        // Return wasm pointer to caller and create a new ZerdeEventloopMsgs.
        std::mem::replace(&mut self.platform.zerde_eventloop_msgs, ZerdeEventloopMsgs::new()).take_ptr()
    }

    fn wasm_event_handler(&mut self, mut event: Event) {
        self.process_pre_event(&mut event);
        self.call_event_handler(&mut event);
        self.process_post_event(&mut event);
    }

    /// This is unsafe, since we don't have a mutex on [`Cx::call_rust_sync_fn`]! So if someone
    /// were to change it while another thread is about to call it, bad things can happen. We guard against this
    /// by making sure that we only mutate it when the app is being initialized.
    ///
    /// See [`Cx::on_call_rust_sync_internal`] for this guarantee.
    pub unsafe fn call_rust_sync(&self, zerde_ptr: u64) -> u64 {
        assert!(self.finished_app_new);
        if let Some(func) = *self.platform.call_rust_sync_fn.get() {
            let mut zerde_parser = ZerdeParser::from(zerde_ptr);
            let name = zerde_parser.parse_string();
            let params = zerde_parser.parse_zap_params();
            let return_params = Cx::call_rust_sync_dispatch(func, name, params);
            let mut zerde_builder = ZerdeBuilder::new();
            zerde_builder.build_zap_params(return_params);
            zerde_builder.take_ptr()
        } else {
            panic!("call_rust_sync called but no call_rust_sync_fn was registered");
        }
    }

    /// We have to make sure that we only mutate this during initialization, since there are no other threads there.
    /// Note that the assertion also happens in [`Cx::on_call_rust_sync`] for consistency, so we just
    /// check it here for good measure.
    ///
    /// See [`Cx::call_rust_sync`].
    pub(crate) fn on_call_rust_sync_internal(&self, func: CallRustSyncFn) {
        assert!(!self.finished_app_new);
        assert!(!self.platform.is_initialized);
        let fn_ref = unsafe { &mut *self.platform.call_rust_sync_fn.get() };
        *fn_ref = Some(func);
    }
}

impl CxDesktopVsWasmCommon for Cx {
    /// See [`CxDesktopVsWasmCommon::get_default_window_size`] for documentation.
    fn get_default_window_size(&self) -> Vec2 {
        return self.platform.window_geom.inner_size;
    }

    /// See [`CxDesktopVsWasmCommon::file_write`] for documentation.
    fn file_write(&mut self, _path: &str, _data: &[u8]) {
        unimplemented!();
    }

    /// See [`CxDesktopVsWasmCommon::http_send`] for documentation.
    fn http_send(
        &mut self,
        verb: &str,
        path: &str,
        proto: &str,
        domain: &str,
        port: u16,
        content_type: &str,
        body: &[u8],
        signal: Signal,
    ) {
        self.platform.zerde_eventloop_msgs.http_send(verb, path, proto, domain, port, content_type, body, signal);
    }

    /// See [`CxDesktopVsWasmCommon::websocket_send`] for documentation.
    fn websocket_send(&mut self, url: &str, data: &[u8]) {
        self.platform.zerde_eventloop_msgs.websocket_send(url, data);
    }

    /// See [`CxDesktopVsWasmCommon::call_js`] for documentation.
    fn call_js(&mut self, name: &str, params: Vec<ZapParam>) {
        self.platform.zerde_eventloop_msgs.call_js(name, params);
    }

    /// See [`CxDesktopVsWasmCommon::return_to_js`] for documentation.
    fn return_to_js(&mut self, callback_id: u32, mut params: Vec<ZapParam>) {
        params.insert(0, format!("{}", callback_id).into_param());
        self.call_js("_zaplibReturnParams", params);
    }
}

impl CxPlatformCommon for Cx {
    /// See [`CxPlatformCommon::post_signal`] for documentation.
    fn post_signal(signal: Signal, status: StatusId) {
        // TODO(JP): Signals are overcomplicated; let's simplify them..
        if signal.signal_id != 0 {
            let mut signals = HashMap::new();
            let mut new_set = BTreeSet::new();
            new_set.insert(status);
            signals.insert(signal, new_set);
            Cx::send_event_from_any_thread(Event::Signal(SignalEvent { signals }));
        }
    }

    /// See [`CxPlatformCommon::show_text_ime`] for documentation.
    fn show_text_ime(&mut self, x: f32, y: f32) {
        self.platform.zerde_eventloop_msgs.show_text_ime(x, y);
    }

    /// See [`CxPlatformCommon::hide_text_ime`] for documentation.
    fn hide_text_ime(&mut self) {
        self.platform.zerde_eventloop_msgs.hide_text_ime();
    }

    /// See [`CxPlatformCommon::start_timer`] for documentation.
    fn start_timer(&mut self, interval: f64, repeats: bool) -> Timer {
        self.last_timer_id += 1;
        self.platform.zerde_eventloop_msgs.start_timer(self.last_timer_id, interval, repeats);
        Timer { timer_id: self.last_timer_id }
    }

    /// See [`CxPlatformCommon::stop_timer`] for documentation.
    fn stop_timer(&mut self, timer: &mut Timer) {
        if timer.timer_id != 0 {
            self.platform.zerde_eventloop_msgs.stop_timer(timer.timer_id);
            timer.timer_id = 0;
        }
    }

    /// See [`CxPlatformCommon::update_menu`] for documentation.
    fn update_menu(&mut self, _menu: &Menu) {}

    /// See [`CxPlatformCommon::update_menu`] for documentation.
    fn copy_text_to_clipboard(&mut self, text: &str) {
        self.platform.zerde_eventloop_msgs.text_copy_response(text);
    }

    fn send_event_from_any_thread(event: Event) {
        let event_ptr = Box::into_raw(Box::new(event));
        unsafe {
            _sendEventFromAnyThread(event_ptr as u64);
        }
    }
}

/// See https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent/button#return_value
fn get_mouse_button(button: usize) -> MouseButton {
    return match button {
        0 => MouseButton::Left,
        2 => MouseButton::Right,
        _ => MouseButton::Other,
    };
}

// storage buffers for graphics API related platform
pub(crate) struct CxPlatform {
    pub(crate) is_initialized: bool,
    pub(crate) window_geom: WindowGeom,
    pub(crate) zerde_eventloop_msgs: ZerdeEventloopMsgs,
    pub(crate) vertex_buffers: usize,
    pub(crate) index_buffers: usize,
    pub(crate) vaos: usize,
    pub(crate) pointers_down: Vec<bool>,
    call_rust_sync_fn: UnsafeCell<Option<CallRustSyncFn>>,
    // pub(crate) xr_last_left_input: XRInput,
    // pub(crate) xr_last_right_input: XRInput,
}

impl Default for CxPlatform {
    fn default() -> CxPlatform {
        CxPlatform {
            is_initialized: false,
            window_geom: WindowGeom::default(),
            zerde_eventloop_msgs: ZerdeEventloopMsgs::new(),
            vertex_buffers: 0,
            index_buffers: 0,
            vaos: 0,
            pointers_down: Vec::new(),
            call_rust_sync_fn: UnsafeCell::new(None),
            // xr_last_left_input: XRInput::default(),
            // xr_last_right_input: XRInput::default(),
        }
    }
}

impl CxPlatform {}

pub(crate) struct ZerdeEventloopMsgs {
    builder: ZerdeBuilder,
}

/// Send messages from wasm to JS.
/// It's important that the id of each message type matches the index of
/// its corresponding function in `sendFnTable` (main_worker.ts)
impl ZerdeEventloopMsgs {
    pub(crate) fn new() -> Self {
        Self { builder: ZerdeBuilder::new() }
    }

    fn take_ptr(self /* move! */) -> u64 {
        self.builder.take_ptr()
    }

    pub(crate) fn end(&mut self) {
        self.builder.send_u32(0);
    }

    pub(crate) fn run_webgl(&mut self, zerde_webgl_ptr: u64) {
        self.builder.send_u32(1);
        self.builder.send_u64(zerde_webgl_ptr);
    }

    pub(crate) fn log(&mut self, msg: &str) {
        self.builder.send_u32(2);
        self.builder.send_string(msg);
    }

    pub(crate) fn request_animation_frame(&mut self) {
        self.builder.send_u32(3);
    }

    pub(crate) fn set_document_title(&mut self, title: &str) {
        self.builder.send_u32(4);
        self.builder.send_string(title);
    }

    pub(crate) fn set_mouse_cursor(&mut self, mouse_cursor: MouseCursor) {
        self.builder.send_u32(5);
        self.builder.send_u32(mouse_cursor as u32);
    }

    pub(crate) fn show_text_ime(&mut self, x: f32, y: f32) {
        self.builder.send_u32(6);
        self.builder.send_f32(x);
        self.builder.send_f32(y);
    }

    pub(crate) fn hide_text_ime(&mut self) {
        self.builder.send_u32(7);
    }

    pub(crate) fn text_copy_response(&mut self, response: &str) {
        self.builder.send_u32(8);
        self.builder.send_string(response);
    }

    pub(crate) fn start_timer(&mut self, id: u64, interval: f64, repeats: bool) {
        self.builder.send_u32(9);
        self.builder.send_u32(if repeats { 1 } else { 0 });
        self.builder.send_f64(id as f64);
        self.builder.send_f64(interval);
    }

    pub(crate) fn stop_timer(&mut self, id: u64) {
        self.builder.send_u32(10);
        self.builder.send_f64(id as f64);
    }

    pub(crate) fn xr_start_presenting(&mut self) {
        self.builder.send_u32(11);
    }

    pub(crate) fn xr_stop_presenting(&mut self) {
        self.builder.send_u32(12);
    }

    pub(crate) fn http_send(
        &mut self,
        verb: &str,
        path: &str,
        proto: &str,
        domain: &str,
        port: u16,
        content_type: &str,
        body: &[u8],
        signal: Signal,
    ) {
        self.builder.send_u32(13);
        self.builder.send_u32(port as u32);
        self.builder.send_u32(signal.signal_id as u32);
        self.builder.send_string(verb);
        self.builder.send_string(path);
        self.builder.send_string(proto);
        self.builder.send_string(domain);
        self.builder.send_string(content_type);
        self.builder.send_u8slice(body);
    }

    pub(crate) fn fullscreen(&mut self) {
        self.builder.send_u32(14);
    }

    pub(crate) fn normalscreen(&mut self) {
        self.builder.send_u32(15);
    }

    pub(crate) fn websocket_send(&mut self, url: &str, data: &[u8]) {
        self.builder.send_u32(16);
        self.builder.send_string(url);
        self.builder.send_u8slice(data);
    }

    pub(crate) fn enable_global_file_drop_target(&mut self) {
        self.builder.send_u32(17);
    }

    pub(crate) fn call_js(&mut self, name: &str, params: Vec<ZapParam>) {
        self.builder.send_u32(18);
        self.builder.send_string(name);

        self.builder.build_zap_params(params);
    }
}

// for use with sending wasm vec data
#[export_name = "allocWasmVec"]
pub unsafe extern "C" fn alloc_wasm_vec(bytes: u64) -> u64 {
    let mut vec = std::mem::ManuallyDrop::new(vec![0u8; bytes as usize]);
    // let mut vec = Vec::<u8>::with_capacity(bytes as usize);
    // vec.resize(bytes as usize, 0);
    let ptr = vec.as_mut_ptr();
    return ptr as u64;
}

// for use with message passing
#[export_name = "allocWasmMessage"]
pub unsafe extern "C" fn alloc_wasm_message(bytes: u64) -> u64 {
    let buf = std::alloc::alloc(std::alloc::Layout::from_size_align(bytes as usize, mem::align_of::<u64>()).unwrap()) as usize;
    (buf as *mut u64).write(bytes as u64);
    buf as u64
}

// for use with message passing
#[export_name = "reallocWasmMessage"]
pub unsafe extern "C" fn realloc_wasm_message(in_buf: u64, new_bytes: u64) -> u64 {
    let old_buf = in_buf as *mut u8;
    let old_bytes = (old_buf as *mut u64).read() as usize;
    let new_buf = alloc::alloc(alloc::Layout::from_size_align(new_bytes as usize, mem::align_of::<u64>()).unwrap()) as *mut u8;
    ptr::copy_nonoverlapping(old_buf, new_buf, old_bytes);
    alloc::dealloc(old_buf as *mut u8, alloc::Layout::from_size_align(old_bytes as usize, mem::align_of::<u64>()).unwrap());
    (new_buf as *mut u64).write(new_bytes as u64);
    new_buf as u64
}

#[export_name = "deallocWasmMessage"]
pub unsafe extern "C" fn dealloc_wasm_message(in_buf: u64) {
    let buf = in_buf as *mut u8;
    let bytes = buf.read() as usize;
    std::alloc::dealloc(buf as *mut u8, std::alloc::Layout::from_size_align(bytes as usize, mem::align_of::<u64>()).unwrap());
}

fn create_arc_vec_inner<T>(vec_ptr: u64, vec_len: u64) -> u64 {
    let vec: Vec<T> = unsafe { Vec::from_raw_parts(vec_ptr as *mut T, vec_len as usize, vec_len as usize) };
    let arc = Arc::new(vec);
    Arc::into_raw(arc) as u64
}

#[export_name = "createArcVec"]
pub unsafe extern "C" fn create_arc_vec(vec_ptr: u64, vec_len: u64, param_type: u64) -> u64 {
    match param_type as u32 {
        ZAP_PARAM_READ_ONLY_UINT8_BUFFER => create_arc_vec_inner::<u8>(vec_ptr, vec_len),
        ZAP_PARAM_READ_ONLY_FLOAT32_BUFFER => create_arc_vec_inner::<f32>(vec_ptr, vec_len),
        ZAP_PARAM_READ_ONLY_UINT32_BUFFER => create_arc_vec_inner::<u32>(vec_ptr, vec_len),
        v => panic!("create_arc_vec: Invalid param type: {}", v),
    }
}

#[export_name = "incrementArc"]
pub unsafe extern "C" fn increment_arc(arc_ptr: u64) {
    Arc::increment_strong_count(arc_ptr as usize as *const Vec<u8>);
}

#[export_name = "decrementArc"]
pub unsafe extern "C" fn decrement_arc(arc_ptr: u64) {
    Arc::decrement_strong_count(arc_ptr as usize as *const Vec<u8>);
}

#[export_name = "deallocVec"]
pub unsafe extern "C" fn dealloc_vec(vec_ptr: u64, vec_len: u64, vec_cap: u64) {
    let vec: Vec<u8> = Vec::from_raw_parts(vec_ptr as *mut u8, vec_len as usize, vec_cap as usize);
    drop(vec);
}

extern "C" {
    fn _consoleLog(chars: u64, len: u64);
    fn _throwError(chars: u64, len: u64);
    pub fn performanceNow() -> f64;
    fn _sendEventFromAnyThread(event_ptr: u64);
}

pub fn console_log(val: &str) {
    unsafe {
        let chars = val.chars().collect::<Vec<char>>();
        _consoleLog(chars.as_ptr() as u64, chars.len() as u64);
    }
}

pub fn throw_error(val: &str) {
    unsafe {
        let chars = val.chars().collect::<Vec<char>>();
        _throwError(chars.as_ptr() as u64, chars.len() as u64);
    }
}

extern "C" {
    fn sendTaskWorkerMessage(tw_message_ptr: u64);
}

pub(crate) const TASK_WORKER_INITIAL_RETURN_VALUE: i32 = -1;
pub(crate) const TASK_WORKER_ERROR_RETURN_VALUE: i32 = -2;
const TASK_WORKER_MESSAGE_TYPE_HTTP_STREAM_NEW: u32 = 1;
const TASK_WORKER_MESSAGE_TYPE_HTTP_STREAM_READ: u32 = 2;

/// Opens a new HTTP stream, blocks until there's a successful response, and returns a stream id.
pub(crate) fn send_task_worker_message_http_stream_new(url: &str, method: &str, body: &[u8], headers: &[(&str, &str)]) -> i32 {
    let mut stream_id = TASK_WORKER_INITIAL_RETURN_VALUE;
    let mut zerde_builder = ZerdeBuilder::new();
    zerde_builder.send_u32(TASK_WORKER_MESSAGE_TYPE_HTTP_STREAM_NEW);
    zerde_builder.send_u32(&mut stream_id as *mut i32 as u32);
    zerde_builder.send_string(url);
    zerde_builder.send_string(method);
    zerde_builder.send_u8slice(body);
    zerde_builder.send_u32(headers.len() as u32);
    for (name, value) in headers {
        zerde_builder.send_string(name);
        zerde_builder.send_string(value);
    }
    let zerde_ptr = zerde_builder.take_ptr();
    unsafe {
        sendTaskWorkerMessage(zerde_ptr);
        // Wait until the task worker sets `stream_id` to a return value.
        core::arch::wasm32::memory_atomic_wait32(&mut stream_id as *mut i32, TASK_WORKER_INITIAL_RETURN_VALUE, -1);
        dealloc_wasm_message(zerde_ptr);
    }
    stream_id
}

/// Makes a read call for an HTTP stream, blocking until its fulfilled, and returning the number of bytes read
/// (or 0 when we're at the end of the stream).
pub(crate) fn send_task_worker_message_http_stream_read(stream_id: i32, buf_ptr: *mut u8, buf_len: usize) -> i32 {
    let mut bytes_read = TASK_WORKER_INITIAL_RETURN_VALUE;
    let mut zerde_builder = ZerdeBuilder::new();
    zerde_builder.send_u32(TASK_WORKER_MESSAGE_TYPE_HTTP_STREAM_READ);
    zerde_builder.send_u32(&mut bytes_read as *mut i32 as u32);
    zerde_builder.send_u32(stream_id as u32);
    zerde_builder.send_u32(buf_ptr as u32);
    zerde_builder.send_u32(buf_len as u32);
    let zerde_ptr = zerde_builder.take_ptr();
    unsafe {
        sendTaskWorkerMessage(zerde_ptr);
        // Wait until the task worker sets `bytes_read` to a return value.
        core::arch::wasm32::memory_atomic_wait32(&mut bytes_read as *mut i32, TASK_WORKER_INITIAL_RETURN_VALUE, -1);
        dealloc_wasm_message(zerde_ptr);
    }
    bytes_read
}
