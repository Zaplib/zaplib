//! Windows platform-specific entry point.
//!
//! Win 10 only because of DX12 + terminal API.
use crate::cx_win32::*;
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
        self.platform_type = PlatformType::Windows;

        let mut win32_app = Win32App::new();

        win32_app.init();

        let mut d3d11_windows: Vec<D3d11Window> = Vec::new();

        let d3d11_cx = D3d11Cx::new();

        self.platform.d3d11_cx = Some(&d3d11_cx);

        self.load_fonts();

        self.call_event_handler(&mut Event::Construct);

        self.request_draw();
        let mut passes_todo = Vec::new();

        win32_app.event_loop(|win32_app, events| {
            self.last_event_time = win32_app.time_now();

            //if let Ok(d3d11_cx) = d3d11_cx.lock(){
            // acquire d3d11_cx exclusive
            for event in events {
                self.process_pre_event(event);
                match &event {
                    Event::WindowResizeLoop(wr) => {
                        for d3d11_window in &mut d3d11_windows {
                            if d3d11_window.window_id == wr.window_id {
                                if wr.was_started {
                                    d3d11_window.start_resize();
                                } else {
                                    d3d11_window.stop_resize();
                                }
                            }
                        }
                    }
                    Event::WindowGeomChange(re) => {
                        // do this here because mac
                        for d3d11_window in &mut d3d11_windows {
                            if d3d11_window.window_id == re.window_id {
                                d3d11_window.window_geom = re.new_geom.clone();
                                self.windows[re.window_id].window_geom = re.new_geom.clone();
                                //if re.old_geom.inner_size != re.new_geom.inner_size{
                                self.request_draw();
                                //}
                                break;
                            }
                        }
                        // ok lets not redraw all, just this window
                        self.call_event_handler(event);
                    }
                    Event::WindowClosed(wc) => {
                        // do this here because mac
                        // lets remove the window from the set
                        self.windows[wc.window_id].window_state = CxWindowState::Closed;
                        self.windows_free.push(wc.window_id);
                        // remove the d3d11/win32 window

                        for index in 0..d3d11_windows.len() {
                            if d3d11_windows[index].window_id == wc.window_id {
                                d3d11_windows.remove(index);
                                if d3d11_windows.is_empty() {
                                    win32_app.terminate_event_loop();
                                }
                                for d3d11_window in &mut d3d11_windows {
                                    d3d11_window.win32_window.update_ptrs();
                                }
                            }
                        }
                        self.call_event_handler(event);
                    }
                    Event::SystemEvent(e) => {
                        match e {
                            SystemEvent::WindowSetHoverCursor(mc) => {
                                self.set_hover_mouse_cursor(mc.clone());
                            }
                            SystemEvent::Paint => {
                                let vsync = self.process_desktop_paint_callbacks();

                                // construct or destruct windows
                                for (index, window) in self.windows.iter_mut().enumerate() {
                                    window.window_state = match &window.window_state {
                                        CxWindowState::Create { inner_size, position, title, .. } => {
                                            // lets create a platformwindow
                                            let d3d11_window =
                                                D3d11Window::new(index, &d3d11_cx, win32_app, *inner_size, *position, title);
                                            window.window_geom = d3d11_window.window_geom.clone();
                                            d3d11_windows.push(d3d11_window);
                                            for d3d11_window in &mut d3d11_windows {
                                                d3d11_window.win32_window.update_ptrs();
                                            }
                                            CxWindowState::Created
                                        }
                                        CxWindowState::Close => {
                                            // ok we close the window
                                            // lets send it a WM_CLOSE event
                                            for d3d11_window in &mut d3d11_windows {
                                                if d3d11_window.window_id == index {
                                                    d3d11_window.win32_window.close_window();
                                                    if !win32_app.event_loop_running {
                                                        return false;
                                                    }
                                                    break;
                                                }
                                            }
                                            CxWindowState::Closed
                                        }
                                        CxWindowState::Created => CxWindowState::Created,
                                        CxWindowState::Closed => CxWindowState::Closed,
                                    };

                                    if let Some(set_position) = window.window_set_position {
                                        for d3d11_window in &mut d3d11_windows {
                                            if d3d11_window.window_id == index {
                                                d3d11_window.win32_window.set_position(set_position);
                                            }
                                        }
                                    }

                                    window.window_command = match &window.window_command {
                                        CxWindowCmd::Restore => {
                                            for d3d11_window in &mut d3d11_windows {
                                                if d3d11_window.window_id == index {
                                                    d3d11_window.win32_window.restore();
                                                }
                                            }
                                            CxWindowCmd::None
                                        }
                                        CxWindowCmd::Maximize => {
                                            for d3d11_window in &mut d3d11_windows {
                                                if d3d11_window.window_id == index {
                                                    d3d11_window.win32_window.maximize();
                                                }
                                            }
                                            CxWindowCmd::None
                                        }
                                        CxWindowCmd::Minimize => {
                                            for d3d11_window in &mut d3d11_windows {
                                                if d3d11_window.window_id == index {
                                                    d3d11_window.win32_window.minimize();
                                                }
                                            }
                                            CxWindowCmd::None
                                        }
                                        _ => CxWindowCmd::None,
                                    };

                                    window.window_set_position = None;

                                    if let Some(topmost) = window.window_topmost {
                                        for d3d11_window in &mut d3d11_windows {
                                            if d3d11_window.window_id == index {
                                                d3d11_window.win32_window.set_topmost(topmost);
                                            }
                                        }
                                    }
                                }

                                // set a cursor
                                if self.down_mouse_cursor.is_some() {
                                    win32_app.set_mouse_cursor(self.down_mouse_cursor.as_ref().unwrap().clone())
                                } else if self.hover_mouse_cursor.is_some() {
                                    win32_app.set_mouse_cursor(self.hover_mouse_cursor.as_ref().unwrap().clone())
                                } else {
                                    win32_app.set_mouse_cursor(MouseCursor::Default)
                                }

                                if let Some(set_ime_position) = self.platform.set_ime_position {
                                    self.platform.set_ime_position = None;
                                    for d3d11_window in &mut d3d11_windows {
                                        d3d11_window.win32_window.set_ime_spot(set_ime_position);
                                    }
                                }

                                while !self.platform.start_timer.is_empty() {
                                    let (timer_id, interval, repeats) = self.platform.start_timer.pop().unwrap();
                                    win32_app.start_timer(timer_id, interval, repeats);
                                }

                                while !self.platform.stop_timer.is_empty() {
                                    let timer_id = self.platform.stop_timer.pop().unwrap();
                                    win32_app.stop_timer(timer_id);
                                }

                                // build a list of renderpasses to repaint
                                let mut windows_need_repaint = 0;
                                self.compute_passes_to_repaint(&mut passes_todo, &mut windows_need_repaint);

                                if !passes_todo.is_empty() {
                                    self.hlsl_compile_shaders(&d3d11_cx);
                                    for pass_id in &passes_todo {
                                        match self.passes[*pass_id].dep_of.clone() {
                                            CxPassDepOf::Window(window_id) => {
                                                // find the accompanying render window
                                                if let Some(d3d11_window) =
                                                    d3d11_windows.iter_mut().find(|w| w.window_id == window_id)
                                                {
                                                    windows_need_repaint -= 1;

                                                    let dpi_factor = d3d11_window.window_geom.dpi_factor;
                                                    self.passes[*pass_id].set_dpi_factor(dpi_factor);

                                                    d3d11_window.resize_buffers(&d3d11_cx);

                                                    self.draw_pass_to_window(
                                                        *pass_id,
                                                        vsync,
                                                        dpi_factor,
                                                        d3d11_window,
                                                        &d3d11_cx,
                                                    );
                                                    // call redraw if we guessed the dpi wrong on startup
                                                    if d3d11_window.first_draw {
                                                        d3d11_window.first_draw = false;
                                                        if dpi_factor != self.default_dpi_factor {
                                                            self.request_draw();
                                                        }
                                                    }
                                                }
                                            }
                                            CxPassDepOf::Pass(parent_pass_id) => {
                                                let dpi_factor = self.get_delegated_dpi_factor(parent_pass_id);
                                                self.draw_pass_to_texture(*pass_id, dpi_factor, &d3d11_cx);
                                            }
                                            CxPassDepOf::None => {
                                                self.draw_pass_to_texture(*pass_id, 1.0, &d3d11_cx);
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

            !(self.requested_draw || self.requested_next_frame)
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
        Win32App::post_signal(signal, status);
    }

    /// See [`CxPlatformCommon::update_menu`] for documentation.
    fn update_menu(&mut self, _menu: &Menu) {}

    /// See [`CxPlatformCommon::update_menu`] for documentation.
    fn copy_text_to_clipboard(&mut self, text: &str) {
        Win32App::copy_text_to_clipboard(text);
    }

    fn send_event_from_any_thread(_event: Event) {
        todo!();
    }
}

#[derive(Default)]
pub(crate) struct CxPlatform {
    pub(crate) set_ime_position: Option<Vec2>,
    pub(crate) start_timer: Vec<(u64, f64, bool)>,
    pub(crate) stop_timer: Vec<u64>,
    pub(crate) desktop: CxDesktop,
    pub(crate) d3d11_cx: Option<*const D3d11Cx>,
}
