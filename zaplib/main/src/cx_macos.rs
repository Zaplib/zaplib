//! Mac OS X platform-specific entry point.

use std::collections::BTreeSet;
use std::collections::HashMap;

#[cfg(feature = "cef-server")]
use zaplib_objc_sys::msg_send;

use crate::cx_cocoa::*;
use crate::*;

impl Cx {
    pub fn event_loop<F>(&mut self, mut event_handler: F)
    where
        F: FnMut(&mut Cx, &mut Event),
    {
        self.event_handler =
            Some(&mut event_handler as *const dyn FnMut(&mut Cx, &mut Event) as *mut dyn FnMut(&mut Cx, &mut Event));
        self.event_loop_core();
        self.event_handler = None;
    }

    fn event_loop_core(&mut self) {
        self.platform_type = PlatformType::OSX;

        let mut cocoa_app = CocoaApp::new();

        cocoa_app.init();

        let mut metal_cx = MetalCx::new();

        let mut metal_windows: Vec<MetalWindow> = Vec::new();

        self.load_fonts();

        self.call_event_handler(&mut Event::Construct);

        self.request_draw();

        let mut passes_todo = Vec::new();

        cocoa_app.event_loop(|cocoa_app, events| {
            //let mut paint_dirty = false;
            self.last_event_time = cocoa_app.time_now();

            for event in events {
                self.process_pre_event(event);

                match &event {
                    Event::WindowResizeLoop(wr) => {
                        for metal_window in &mut metal_windows {
                            if metal_window.window_id == wr.window_id {
                                if wr.was_started {
                                    metal_window.start_resize();
                                } else {
                                    metal_window.stop_resize();
                                }
                            }
                        }
                    }
                    Event::WindowGeomChange(re) => {
                        // do this here because mac
                        for metal_window in &mut metal_windows {
                            if metal_window.window_id == re.window_id {
                                metal_window.window_geom = re.new_geom.clone();
                                self.windows[re.window_id].window_geom = re.new_geom.clone();
                                if re.old_geom.inner_size != re.new_geom.inner_size {
                                    self.request_draw();
                                }
                                break;
                            }
                        }
                        // ok lets not redraw all, just this window
                        self.call_event_handler(event);
                    }
                    Event::WindowClosed(wc) => {
                        // lets remove the window from the set
                        self.windows[wc.window_id].window_state = CxWindowState::Closed;
                        self.windows_free.push(wc.window_id);
                        // remove the d3d11/win32 window

                        for index in 0..metal_windows.len() {
                            if metal_windows[index].window_id == wc.window_id {
                                metal_windows.remove(index);
                                if metal_windows.is_empty() {
                                    cocoa_app.terminate_event_loop();
                                }
                                for metal_window in &mut metal_windows {
                                    metal_window.cocoa_window.update_ptrs();
                                }
                            }
                        }
                        self.call_event_handler(event);

                        #[cfg(feature = "cef-debug")]
                        zaplib_cef::shutdown();
                    }
                    Event::System(e) => {
                        match e {
                            SystemEvent::Paint => {
                                let _vsync = self.process_desktop_paint_callbacks();

                                // construct or destruct windows
                                for (index, window) in self.windows.iter_mut().enumerate() {
                                    window.window_state = match &window.window_state {
                                        CxWindowState::Create {
                                            inner_size,
                                            position,
                                            title,
                                            add_drop_target_for_app_open_files,
                                            #[cfg(feature = "cef")]
                                            cef_url,
                                            #[cfg(feature = "cef-server")]
                                            get_resource_url_callback,
                                        } => {
                                            // lets create a platformwindow
                                            let metal_window = MetalWindow::new(
                                                index,
                                                &metal_cx,
                                                cocoa_app,
                                                *inner_size,
                                                *position,
                                                title,
                                                *add_drop_target_for_app_open_files,
                                            );
                                            window.window_geom = metal_window.window_geom.clone();

                                            #[cfg(feature = "cef")]
                                            {
                                                use crate::cx_apple::*;
                                                if let Some(url) = cef_url {
                                                    let content_view: id =
                                                        unsafe { msg_send![metal_window.cocoa_window.window, contentView] };
                                                    self.cef_browser.initialize(
                                                        *inner_size,
                                                        url,
                                                        content_view as zaplib_cef::WindowHandle,
                                                        #[cfg(feature = "cef-server")]
                                                        *get_resource_url_callback,
                                                    );
                                                    unsafe {
                                                        // Put our own NSView on top of CEF.
                                                        let () = msg_send![metal_window.cocoa_window.view, removeFromSuperview];
                                                        let () =
                                                            msg_send![content_view, addSubview: metal_window.cocoa_window.view];

                                                        if *add_drop_target_for_app_open_files {
                                                            // See [`disable_cef_dragged_types`] for more information
                                                            disable_cef_dragged_types(content_view);
                                                        }
                                                    }
                                                    cocoa_app.start_cef_timer();
                                                }
                                            }

                                            metal_windows.push(metal_window);
                                            for metal_window in &mut metal_windows {
                                                metal_window.cocoa_window.update_ptrs();
                                            }
                                            CxWindowState::Created
                                        }
                                        CxWindowState::Close => {
                                            for metal_window in &mut metal_windows {
                                                if metal_window.window_id == index {
                                                    metal_window.cocoa_window.close_window();
                                                    break;
                                                }
                                            }
                                            CxWindowState::Closed
                                        }
                                        CxWindowState::Created => CxWindowState::Created,
                                        CxWindowState::Closed => CxWindowState::Closed,
                                    };

                                    window.window_command = match &window.window_command {
                                        CxWindowCmd::Restore => {
                                            for metal_window in &mut metal_windows {
                                                if metal_window.window_id == index {
                                                    metal_window.cocoa_window.restore();
                                                }
                                            }
                                            CxWindowCmd::None
                                        }
                                        CxWindowCmd::Maximize => {
                                            for metal_window in &mut metal_windows {
                                                if metal_window.window_id == index {
                                                    metal_window.cocoa_window.maximize();
                                                }
                                            }
                                            CxWindowCmd::None
                                        }
                                        CxWindowCmd::Minimize => {
                                            for metal_window in &mut metal_windows {
                                                if metal_window.window_id == index {
                                                    metal_window.cocoa_window.minimize();
                                                }
                                            }
                                            CxWindowCmd::None
                                        }
                                        _ => CxWindowCmd::None,
                                    };

                                    if let Some(topmost) = window.window_topmost {
                                        for metal_window in &mut metal_windows {
                                            if metal_window.window_id == index {
                                                metal_window.cocoa_window.set_topmost(topmost);
                                            }
                                        }
                                    }
                                }

                                // set a cursor
                                let mouse_cursor = if self.down_mouse_cursor.is_some() {
                                    self.down_mouse_cursor.as_ref().unwrap().clone()
                                } else if self.hover_mouse_cursor.is_some() {
                                    self.hover_mouse_cursor.as_ref().unwrap().clone()
                                } else {
                                    MouseCursor::Default
                                };

                                #[cfg(not(feature = "cef"))]
                                cocoa_app.set_mouse_cursor(mouse_cursor);
                                #[cfg(feature = "cef")]
                                self.cef_browser.set_mouse_cursor(mouse_cursor);

                                #[cfg(not(feature = "cef"))]
                                if let Some(set_ime_position) = self.platform.set_ime_position {
                                    self.platform.set_ime_position = None;
                                    for metal_window in &mut metal_windows {
                                        metal_window.cocoa_window.set_ime_spot(set_ime_position);
                                    }
                                }

                                while !self.platform.start_timer.is_empty() {
                                    let (timer_id, interval, repeats) = self.platform.start_timer.pop().unwrap();
                                    cocoa_app.start_timer(timer_id, interval, repeats);
                                }

                                while !self.platform.stop_timer.is_empty() {
                                    let timer_id = self.platform.stop_timer.pop().unwrap();
                                    cocoa_app.stop_timer(timer_id);
                                }

                                if self.platform.set_menu {
                                    self.platform.set_menu = false;
                                    if let Some(menu) = &self.platform.last_menu {
                                        cocoa_app.update_app_menu(menu, &self.command_settings)
                                    }
                                }

                                // build a list of renderpasses to repaint
                                let mut windows_need_repaint = 0;
                                self.compute_passes_to_repaint(&mut passes_todo, &mut windows_need_repaint);

                                if !passes_todo.is_empty() {
                                    self.mtl_compile_shaders(&metal_cx);

                                    for pass_id in &passes_todo {
                                        match self.passes[*pass_id].dep_of.clone() {
                                            CxPassDepOf::Window(window_id) => {
                                                // find the accompanying render window
                                                // its a render window
                                                windows_need_repaint -= 1;
                                                for metal_window in &mut metal_windows {
                                                    if metal_window.window_id == window_id {
                                                        let dpi_factor = metal_window.window_geom.dpi_factor;

                                                        metal_window.resize_core_animation_layer(&metal_cx);

                                                        self.draw_pass_to_layer(
                                                            *pass_id,
                                                            dpi_factor,
                                                            metal_window.ca_layer,
                                                            &mut metal_cx,
                                                            metal_window.is_resizing,
                                                        );
                                                        // call redraw if we guessed the dpi wrong on startup
                                                        if metal_window.first_draw {
                                                            metal_window.first_draw = false;
                                                            if dpi_factor != self.default_dpi_factor {
                                                                self.request_draw();
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                            CxPassDepOf::Pass(parent_pass_id) => {
                                                let dpi_factor = self.get_delegated_dpi_factor(parent_pass_id);
                                                self.draw_pass_to_texture(*pass_id, dpi_factor, &metal_cx);
                                            }
                                            CxPassDepOf::None => {
                                                self.draw_pass_to_texture(*pass_id, 1.0, &metal_cx);
                                            }
                                        }
                                    }
                                }
                            }
                            #[cfg(feature = "cef")]
                            SystemEvent::CefDoMessageLoopWork => {
                                self.cef_do_message_loop_work();
                            }
                            _ => {
                                self.call_event_handler(event);
                            }
                        }
                    }
                    Event::None => {}
                    Event::Signal { .. } => {
                        self.call_event_handler(event);
                        self.call_signals();
                    }
                    _ => {
                        self.call_event_handler(event);
                    }
                }
                self.process_post_event(event);
            }

            !(self.requested_draw || self.requested_next_frame)
        })
    }
    /// Schedule another timer in addition to our regular timer, in case we need to do something
    /// earlier. See
    /// https://bitbucket.org/chromiumembedded/cef/issues/2968/documentation-of-external-message-pump
    #[cfg(feature = "cef")]
    pub(crate) fn cef_schedule_message_pump_work(delay_ms: i64) {
        CocoaApp::cef_schedule_message_pump_work(delay_ms);
    }
}

impl CxPlatformCommon for Cx {
    /// See [`CxPlatformCommon::show_text_ime`] for documentation.
    fn show_text_ime(&mut self, x: f32, y: f32) {
        #[cfg(not(feature = "cef"))]
        {
            self.platform.set_ime_position = Some(Vec2 { x, y });
        }

        #[cfg(feature = "cef")]
        {
            self.cef_browser.set_ime_position(Vec2 { x, y });
        }
    }

    /// See [`CxPlatformCommon::hide_text_ime`] for documentation.
    fn hide_text_ime(&mut self) {}

    /// See [`CxPlatformCommon::start_timer`] for documentation.
    fn start_timer(&mut self, interval: f64, repeats: bool) -> Timer {
        self.last_timer_id += 1;
        self.platform.start_timer.push((self.last_timer_id, interval, repeats));
        Timer { timer_id: self.last_timer_id }
    }

    /// See [`CxPlatformCommon::stop_timer`] for documentation.
    fn stop_timer(&mut self, timer: &mut Timer) {
        if timer.timer_id != 0 {
            self.platform.stop_timer.push(timer.timer_id);
            timer.timer_id = 0;
        }
    }

    /// See [`CxPlatformCommon::post_signal`] for documentation.
    fn post_signal(signal: Signal, status: StatusId) {
        if signal.signal_id != 0 {
            let mut signals = HashMap::new();
            let mut new_set = BTreeSet::new();
            new_set.insert(status);
            signals.insert(signal, new_set);
            CocoaApp::send_event_from_any_thread(Event::Signal(SignalEvent { signals }));
        }
    }

    /// See [`CxPlatformCommon::update_menu`] for documentation.
    fn update_menu(&mut self, menu: &Menu) {
        // lets walk the menu and do the cocoa equivalents
        let platform = &mut self.platform;
        if platform.last_menu.is_none() || platform.last_menu.as_ref().unwrap() != menu {
            platform.last_menu = Some(menu.clone());
            platform.set_menu = true;
        }
    }

    /// See [`CxPlatformCommon::update_menu`] for documentation.
    fn copy_text_to_clipboard(&mut self, text: &str) {
        CocoaApp::copy_text_to_clipboard(text);
    }

    /// See [`CxPlatformCommon::send_event_from_any_thread`] for documentation.
    fn send_event_from_any_thread(event: Event) {
        CocoaApp::send_event_from_any_thread(event);
    }
}

#[derive(Clone, Default)]
pub(crate) struct CxPlatform {
    pub(crate) bytes_written: usize,
    pub(crate) draw_calls_done: usize,
    pub(crate) last_menu: Option<Menu>,
    pub(crate) set_menu: bool,
    #[cfg(not(feature = "cef"))]
    pub(crate) set_ime_position: Option<Vec2>,
    pub(crate) start_timer: Vec<(u64, f64, bool)>,
    pub(crate) stop_timer: Vec<u64>,
    pub(crate) desktop: CxDesktop,
}

#[cfg(feature = "cef-server")]
pub(crate) fn get_bundle_directory() -> String {
    #[cfg(feature = "cef-bundle")]
    unsafe {
        use crate::cx_apple::*;
        let file_manager: id = msg_send![class!(NSBundle), mainBundle];
        let current_dir: id = msg_send![file_manager, bundlePath];
        let current_dir = nsstring_to_string(current_dir);
        return current_dir + "/Contents/Resources";
    }

    #[cfg(not(feature = "cef-bundle"))]
    ".".to_string()
}
