//! Linux platform-specific entry point.

use crate::cx_xlib::*;
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
        self.platform_type = PlatformType::Linux { custom_window_chrome: LINUX_CUSTOM_WINDOW_CHROME };

        let mut xlib_app = XlibApp::new();

        xlib_app.init();

        let opengl_cx = OpenglCx::new(xlib_app.display);

        let mut opengl_windows: Vec<OpenglWindow> = Vec::new();

        self.load_fonts();

        self.call_event_handler(&mut Event::Construct);

        self.request_draw();

        let mut passes_todo = Vec::new();

        xlib_app.event_loop(|xlib_app, events| {
            self.last_event_time = xlib_app.time_now();
            let mut paint_dirty = false;
            for event in events {
                self.process_pre_event(event);

                match &event {
                    Event::WindowGeomChange(re) => {
                        // do this here because mac
                        for opengl_window in &mut opengl_windows {
                            if opengl_window.window_id == re.window_id {
                                opengl_window.window_geom = re.new_geom.clone();
                                self.windows[re.window_id].window_geom = re.new_geom.clone();
                                // redraw just this windows root draw list
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

                        for index in 0..opengl_windows.len() {
                            if opengl_windows[index].window_id == wc.window_id {
                                opengl_windows.remove(index);
                                if opengl_windows.is_empty() {
                                    xlib_app.terminate_event_loop();
                                }
                                for opengl_window in &mut opengl_windows {
                                    opengl_window.xlib_window.update_ptrs();
                                }
                            }
                        }
                        self.call_event_handler(event);
                    }
                    Event::System(e) => {
                        match e {
                            SystemEvent::WindowSetHoverCursor(mc) => {
                                self.set_hover_mouse_cursor(mc.clone());
                            }
                            SystemEvent::Paint => {
                                let _vsync = self.process_desktop_paint_callbacks();

                                // construct or destruct windows
                                for (index, window) in self.windows.iter_mut().enumerate() {
                                    window.window_state = match &window.window_state {
                                        CxWindowState::Create { inner_size, position, title, .. } => {
                                            // lets create a platformwindow
                                            let opengl_window =
                                                OpenglWindow::new(index, &opengl_cx, xlib_app, *inner_size, *position, title);
                                            window.window_geom = opengl_window.window_geom.clone();
                                            opengl_windows.push(opengl_window);
                                            for opengl_window in &mut opengl_windows {
                                                opengl_window.xlib_window.update_ptrs();
                                            }

                                            CxWindowState::Created
                                        }
                                        CxWindowState::Close => {
                                            for opengl_window in &mut opengl_windows {
                                                if opengl_window.window_id == index {
                                                    opengl_window.xlib_window.close_window();
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
                                            for opengl_window in &mut opengl_windows {
                                                if opengl_window.window_id == index {
                                                    opengl_window.xlib_window.restore();
                                                }
                                            }
                                            CxWindowCmd::None
                                        }
                                        CxWindowCmd::Maximize => {
                                            for opengl_window in &mut opengl_windows {
                                                if opengl_window.window_id == index {
                                                    opengl_window.xlib_window.maximize();
                                                }
                                            }
                                            CxWindowCmd::None
                                        }
                                        CxWindowCmd::Minimize => {
                                            for opengl_window in &mut opengl_windows {
                                                if opengl_window.window_id == index {
                                                    opengl_window.xlib_window.minimize();
                                                }
                                            }
                                            CxWindowCmd::None
                                        }
                                        _ => CxWindowCmd::None,
                                    };

                                    if let Some(topmost) = window.window_topmost {
                                        for opengl_window in &mut opengl_windows {
                                            if opengl_window.window_id == index {
                                                opengl_window.xlib_window.set_topmost(topmost);
                                            }
                                        }
                                    }
                                }
                                // set a cursor
                                if self.down_mouse_cursor.is_some() {
                                    xlib_app.set_mouse_cursor(self.down_mouse_cursor.as_ref().unwrap().clone())
                                } else if self.hover_mouse_cursor.is_some() {
                                    xlib_app.set_mouse_cursor(self.hover_mouse_cursor.as_ref().unwrap().clone())
                                } else {
                                    xlib_app.set_mouse_cursor(MouseCursor::Default)
                                }

                                if let Some(set_ime_position) = self.platform.set_ime_position {
                                    self.platform.set_ime_position = None;
                                    for opengl_window in &mut opengl_windows {
                                        opengl_window.xlib_window.set_ime_spot(set_ime_position);
                                    }
                                }

                                while !self.platform.start_timer.is_empty() {
                                    let (timer_id, interval, repeats) = self.platform.start_timer.pop().unwrap();
                                    xlib_app.start_timer(timer_id, interval, repeats);
                                }

                                while !self.platform.stop_timer.is_empty() {
                                    let timer_id = self.platform.stop_timer.pop().unwrap();
                                    xlib_app.stop_timer(timer_id);
                                }

                                // build a list of renderpasses to repaint
                                let mut windows_need_repaint = 0;
                                self.compute_passes_to_repaint(&mut passes_todo, &mut windows_need_repaint);

                                if !passes_todo.is_empty() {
                                    self.opengl_compile_shaders(&opengl_cx);
                                    for pass_id in &passes_todo {
                                        match self.passes[*pass_id].dep_of.clone() {
                                            CxPassDepOf::Window(window_id) => {
                                                // find the accompanying render window
                                                // its a render window
                                                windows_need_repaint -= 1;
                                                for opengl_window in &mut opengl_windows {
                                                    if opengl_window.window_id == window_id {
                                                        if opengl_window.xlib_window.window.is_none() {
                                                            break;
                                                        }
                                                        let dpi_factor = opengl_window.window_geom.dpi_factor;

                                                        self.passes[*pass_id].set_dpi_factor(dpi_factor);

                                                        opengl_window.resize_framebuffer(&opengl_cx);

                                                        self.passes[*pass_id].paint_dirty = false;

                                                        if self.draw_pass_to_window(
                                                            *pass_id,
                                                            dpi_factor,
                                                            opengl_window,
                                                            &opengl_cx,
                                                        ) {
                                                            // paint it again a few times, apparently this is necessary
                                                            self.passes[*pass_id].paint_dirty = true;
                                                            paint_dirty = true;
                                                        }
                                                        if opengl_window.first_draw {
                                                            opengl_window.first_draw = false;
                                                            self.request_draw();
                                                        }
                                                    }
                                                }
                                            }
                                            CxPassDepOf::Pass(parent_pass_id) => {
                                                let dpi_factor = self.get_delegated_dpi_factor(parent_pass_id);
                                                self.draw_pass_to_texture(*pass_id, dpi_factor, &opengl_cx);
                                            }
                                            CxPassDepOf::None => {
                                                self.draw_pass_to_texture(*pass_id, 1.0, &opengl_cx);
                                            }
                                        }
                                    }
                                }
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

            !(paint_dirty || self.requested_draw || self.requested_next_frame)
        })
    }

    /// TODO(JP): Generalize [`Cx::post_signal`] into this.
    #[cfg(feature = "cef")]
    pub(crate) fn send_event_from_any_thread(_event: Event) {
        todo!();
    }

    #[cfg(feature = "cef")]
    pub(crate) fn cef_schedule_message_pump_work(_delay_ms: i64) {
        todo!();
    }
}

impl CxPlatformCommon for Cx {
    /// See [`CxPlatformCommon::show_text_ime`] for documentation.
    fn show_text_ime(&mut self, x: f32, y: f32) {
        self.platform.set_ime_position = Some(Vec2 { x, y });
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
        XlibApp::post_signal(signal, status);
    }

    /// See [`CxPlatformCommon::update_menu`] for documentation.
    fn update_menu(&mut self, _menu: &Menu) {}

    /// See [`CxPlatformCommon::update_menu`] for documentation.
    fn copy_text_to_clipboard(&mut self, text: &str) {
        XlibApp::copy_text_to_clipboard(text);
    }

    fn send_event_from_any_thread(_event: Event) {
        todo!();
    }
}

#[derive(Clone, Default)]
pub(crate) struct CxPlatform {
    pub(crate) set_ime_position: Option<Vec2>,
    pub(crate) start_timer: Vec<(u64, f64, bool)>,
    pub(crate) stop_timer: Vec<u64>,
    pub(crate) desktop: CxDesktop,
}
