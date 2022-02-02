//! OS X Cocoa bindings.
//!
//! Adapted from <https://github.com/tomaka/winit/blob/master/src/platform/macos/>

#![allow(dead_code)]

use crate::cx_apple::*;
use crate::*;
use std::collections::HashMap;
use std::convert::TryInto;
use std::os::raw::c_void;
use std::path::Path;
use std::time::{Duration, Instant};
//use core_graphics::display::CGDisplay;
//use time::precise_time_ns;

const NSVIEW_CLASS_NAME: &str = "RenderViewClass";

static mut GLOBAL_COCOA_APP: *mut CocoaApp = 0 as *mut _;

//extern {
//    pub(crate) fn mach_absolute_time() -> u64;
//}

#[derive(Clone)]
pub(crate) struct CocoaWindow {
    pub(crate) window_id: usize,
    pub(crate) window_delegate: id,
    //pub(crate) layer_delegate: id,
    pub(crate) view: id,
    pub(crate) window: id,
    pub(crate) live_resize_timer: id,
    pub(crate) cocoa_app: *mut CocoaApp,
    pub(crate) last_window_geom: Option<WindowGeom>,
    #[cfg(not(feature = "cef"))]
    pub(crate) ime_spot: Vec2,
    pub(crate) time_start: Instant,
    pub(crate) is_fullscreen: bool,
    pub(crate) pointers_down: Vec<bool>,
    pub(crate) last_mouse_pos: Vec2,
}

#[derive(Clone)]
pub(crate) struct CocoaTimer {
    timer_id: u64,
    nstimer: id,
    repeats: bool,
}

pub(crate) struct CocoaApp {
    pub(crate) window_class: *const Class,
    pub(crate) window_delegate_class: *const Class,
    pub(crate) timer_delegate_class: *const Class,
    pub(crate) menu_delegate_class: *const Class,
    pub(crate) app_delegate_class: *const Class,
    pub(crate) menu_target_class: *const Class,
    pub(crate) view_class: *const Class,
    pub(crate) menu_delegate_instance: id,
    pub(crate) app_delegate_instance: id,
    pub(crate) const_attributes_for_marked_text: id,
    pub(crate) const_empty_string: id,
    pub(crate) time_start: Instant,
    pub(crate) timer_delegate_instance: id,
    pub(crate) timers: Vec<CocoaTimer>,
    pub(crate) cocoa_windows: Vec<(id, id)>,
    pub(crate) last_key_mod: KeyModifiers,
    pub(crate) startup_focus_hack_ran: bool,
    pub(crate) event_callback: Option<*mut dyn FnMut(&mut CocoaApp, &mut Vec<Event>) -> bool>,
    pub(crate) event_recur_block: bool,
    pub(crate) event_loop_running: bool,
    pub(crate) loop_block: bool,
    last_paint_start_time: Instant,
    scheduled_paint_event: bool,
    #[cfg(not(feature = "cef"))]
    pub(crate) cursors: HashMap<MouseCursor, id>,
    #[cfg(not(feature = "cef"))]
    pub(crate) current_cursor: MouseCursor,
    #[cfg(feature = "cef")]
    cef_timer: std::sync::RwLock<id>,
    // TODO(Paras): Should we actually use this field to block handling events in Rust?
    // This is currently set in ZapApplication but never used.
    #[cfg(feature = "cef")]
    cef_is_handling_event: bool,
}

impl Default for CocoaApp {
    fn default() -> Self {
        Self::new()
    }
}

impl CocoaApp {
    pub(crate) fn new() -> CocoaApp {
        unsafe {
            let timer_delegate_class = define_cocoa_timer_delegate();
            let timer_delegate_instance: id = msg_send![timer_delegate_class, new];
            let menu_delegate_class = define_menu_delegate();
            let menu_delegate_instance: id = msg_send![menu_delegate_class, new];
            let app_delegate_class = define_app_delegate();
            let app_delegate_instance: id = msg_send![app_delegate_class, new];

            let const_attributes = vec![str_to_nsstring("NSMarkedClauseSegment"), str_to_nsstring("NSGlyphInfo")];

            // Construct the bits that are shared between windows
            CocoaApp {
                const_attributes_for_marked_text: msg_send![
                    class!(NSArray),
                    arrayWithObjects: const_attributes.as_ptr()
                    count: const_attributes.len()
                ],
                startup_focus_hack_ran: false,
                const_empty_string: str_to_nsstring(""),
                time_start: Instant::now(),
                timer_delegate_instance,
                timer_delegate_class,
                window_class: define_cocoa_window_class(),
                window_delegate_class: define_cocoa_window_delegate(),
                view_class: define_cocoa_view_class(),
                menu_target_class: define_menu_target_class(),
                menu_delegate_class,
                menu_delegate_instance,
                app_delegate_class,
                app_delegate_instance,
                timers: Vec::new(),
                cocoa_windows: Vec::new(),
                loop_block: false,
                last_key_mod: KeyModifiers { ..Default::default() },
                event_callback: None,
                event_recur_block: false,
                event_loop_running: true,
                // Set to 10 seconds in the past so we'll definitely immediately paint the first frame.
                last_paint_start_time: Instant::now() - Duration::from_secs(10),
                scheduled_paint_event: false,
                #[cfg(not(feature = "cef"))]
                cursors: HashMap::new(),
                #[cfg(not(feature = "cef"))]
                current_cursor: MouseCursor::Default,
                #[cfg(feature = "cef")]
                cef_timer: std::sync::RwLock::new(nil),
                #[cfg(feature = "cef")]
                cef_is_handling_event: false,
            }
        }
    }

    pub(crate) fn update_app_menu(&mut self, menu: &Menu, command_settings: &HashMap<CommandId, CxCommandSetting>) {
        unsafe fn make_menu(
            parent_menu: id,
            delegate: id,
            menu_target_class: *const Class,
            menu: &Menu,
            command_settings: &HashMap<CommandId, CxCommandSetting>,
        ) {
            match menu {
                Menu::Main { items } => {
                    let main_menu: id = msg_send![class!(NSMenu), new];
                    let () = msg_send![main_menu, setTitle: str_to_nsstring("MainMenu")];
                    let () = msg_send![main_menu, setAutoenablesItems: NO];
                    let () = msg_send![main_menu, setDelegate: delegate];

                    for item in items {
                        make_menu(main_menu, delegate, menu_target_class, item, command_settings);
                    }
                    let ns_app: id = msg_send![class!(NSApplication), sharedApplication];
                    let () = msg_send![ns_app, setMainMenu: main_menu];
                }
                Menu::Sub { name, items } => {
                    let sub_menu: id = msg_send![class!(NSMenu), new];
                    let () = msg_send![sub_menu, setTitle: str_to_nsstring(name)];
                    let () = msg_send![sub_menu, setAutoenablesItems: NO];
                    let () = msg_send![sub_menu, setDelegate: delegate];
                    // append item to parebt
                    let sub_item: id = msg_send![
                        parent_menu,
                        addItemWithTitle: str_to_nsstring(name)
                        action: nil
                        keyEquivalent: str_to_nsstring("")
                    ];
                    // connect submenu
                    let () = msg_send![parent_menu, setSubmenu: sub_menu forItem: sub_item];
                    for item in items {
                        make_menu(sub_menu, delegate, menu_target_class, item, command_settings);
                    }
                }
                Menu::Item { name, command } => {
                    let settings =
                        if let Some(settings) = command_settings.get(command) { *settings } else { CxCommandSetting::default() };
                    let sub_item: id = msg_send![
                        parent_menu,
                        addItemWithTitle: str_to_nsstring(name)
                        action: sel!(menuAction:)
                        keyEquivalent: str_to_nsstring(keycode_to_menu_key(settings.key_code, settings.shift))
                    ];
                    let target: id = msg_send![menu_target_class, new];
                    let () = msg_send![sub_item, setTarget: target];
                    let () = msg_send![sub_item, setEnabled: if settings.enabled {YES}else {NO}];

                    let command_usize = command.0;
                    (*target).set_ivar("cocoa_app_ptr", GLOBAL_COCOA_APP as *mut _ as *mut c_void);
                    (*target).set_ivar("command_usize", command_usize);
                }
                Menu::Line => {
                    let sep_item: id = msg_send![class!(NSMenuItem), separatorItem];
                    let () = msg_send![parent_menu, addItem: sep_item];
                }
            }
        }
        unsafe {
            make_menu(nil, self.menu_delegate_instance, self.menu_target_class, menu, command_settings);
        }
    }

    pub(crate) fn startup_focus_hack(&mut self) {
        unsafe {
            if !self.startup_focus_hack_ran {
                self.startup_focus_hack_ran = true;
                let ns_app: id = msg_send![class!(NSApplication), sharedApplication];
                let active: bool = msg_send![ns_app, isActive];
                if !active {
                    let dock_bundle_id: id = str_to_nsstring("com.apple.dock");
                    let dock_array: id =
                        msg_send![class!(NSRunningApplication), runningApplicationsWithBundleIdentifier: dock_bundle_id];
                    let dock_array_len: u64 = msg_send![dock_array, count];
                    if dock_array_len == 0 {
                        panic!("Dock not running");
                    } else {
                        let dock: id = msg_send![dock_array, objectAtIndex: 0];
                        let _status: BOOL = msg_send![
                            dock,
                            activateWithOptions: NSApplicationActivationOptions::NSApplicationActivateIgnoringOtherApps
                        ];
                        let ns_running_app: id = msg_send![class!(NSRunningApplication), currentApplication];
                        let () = msg_send![
                            ns_running_app,
                            activateWithOptions: NSApplicationActivationOptions::NSApplicationActivateIgnoringOtherApps
                        ];
                    }
                }
            }
        }
    }

    pub(crate) fn init(&mut self) {
        unsafe {
            GLOBAL_COCOA_APP = self;

            #[cfg(feature = "cef")]
            let ns_application_class = define_ns_application();
            #[cfg(feature = "cef")]
            let ns_app: id = msg_send![ns_application_class, sharedApplication];

            #[cfg(not(feature = "cef"))]
            let ns_app: id = msg_send![class!(NSApplication), sharedApplication];

            (*self.timer_delegate_instance).set_ivar("cocoa_app_ptr", self as *mut _ as *mut c_void);
            (*self.menu_delegate_instance).set_ivar("cocoa_app_ptr", self as *mut _ as *mut c_void);
            (*self.app_delegate_instance).set_ivar("cocoa_app_ptr", self as *mut _ as *mut c_void);
            let () = msg_send![ns_app, setDelegate: self.app_delegate_instance];
            let () = msg_send![
                ns_app,
                setActivationPolicy: NSApplicationActivationPolicy::NSApplicationActivationPolicyRegular as i64
            ];
            let () = msg_send![ns_app, activateIgnoringOtherApps: YES];
        }
    }

    pub(crate) fn time_now(&self) -> f64 {
        let time_now = Instant::now(); //unsafe {mach_absolute_time()};
        (time_now.duration_since(self.time_start)).as_micros() as f64 / 1_000_000.0
    }

    unsafe fn process_ns_event(&mut self, ns_event: id) {
        let ev_type: NSEventType = msg_send![ns_event, type];

        let ns_app: id = msg_send![class!(NSApplication), sharedApplication];
        // Note that callbacks (e.g. mouse move) are called during this `sendEvent:` call.
        let () = msg_send![ns_app, sendEvent: ns_event];

        if ev_type as u64 == 21 {
            // some missing event from cocoa-rs crate
            return;
        }

        match ev_type {
            #[cfg(not(feature = "cef"))]
            NSEventType::NSKeyUp => {
                if let Some(key_code) = get_event_keycode(ns_event) {
                    let modifiers = get_event_key_modifier(ns_event);
                    //let key_char = get_event_char(ns_event);
                    let is_repeat: bool = msg_send![ns_event, isARepeat];
                    self.do_callback(&mut vec![Event::KeyUp(KeyEvent {
                        key_code,
                        //key_char: key_char,
                        is_repeat,
                        modifiers,
                        time: self.time_now(),
                    })]);
                }
            }
            #[cfg(not(feature = "cef"))]
            NSEventType::NSKeyDown => {
                if let Some(key_code) = get_event_keycode(ns_event) {
                    let modifiers = get_event_key_modifier(ns_event);
                    //let key_char = get_event_char(ns_event);
                    let is_repeat: bool = msg_send![ns_event, isARepeat];
                    //let is_return = if let KeyCode::Return = key_code{true} else{false};

                    match key_code {
                        KeyCode::KeyV => {
                            if modifiers.logo || modifiers.control {
                                // was a paste
                                let pasteboard: id = msg_send![class!(NSPasteboard), generalPasteboard];
                                let ns_string: id = msg_send![pasteboard, stringForType: NSStringPboardType];
                                let string = nsstring_to_string(ns_string);
                                self.do_callback(&mut vec![Event::TextInput(TextInputEvent {
                                    input: string,
                                    was_paste: true,
                                    replace_last: false,
                                })]);
                            }
                        }
                        KeyCode::KeyX | KeyCode::KeyC => {
                            if modifiers.logo || modifiers.control {
                                // cut or copy.
                                let mut events = vec![Event::TextCopy];
                                self.do_callback(&mut events);
                            }
                        }
                        _ => {}
                    }

                    self.do_callback(&mut vec![Event::KeyDown(KeyEvent {
                        key_code,
                        //key_char: key_char,
                        is_repeat,
                        modifiers,
                        time: self.time_now(),
                    })]);
                }
            }
            NSEventType::NSFlagsChanged => {
                let modifiers = get_event_key_modifier(ns_event);
                let last_key_mod = self.last_key_mod.clone();
                self.last_key_mod = modifiers.clone();
                let mut events = Vec::new();
                fn add_event(
                    time: f64,
                    old: bool,
                    new: bool,
                    modifiers: KeyModifiers,
                    events: &mut Vec<Event>,
                    key_code: KeyCode,
                ) {
                    if old != new {
                        let event = KeyEvent {
                            key_code,
                            //key_char: '\0',
                            is_repeat: false,
                            modifiers,
                            time,
                        };
                        if new {
                            events.push(Event::KeyDown(event));
                        } else {
                            events.push(Event::KeyUp(event));
                        }
                    }
                }
                let time = self.time_now();
                add_event(time, last_key_mod.shift, modifiers.shift, modifiers.clone(), &mut events, KeyCode::Shift);
                add_event(time, last_key_mod.alt, modifiers.alt, modifiers.clone(), &mut events, KeyCode::Alt);
                add_event(time, last_key_mod.logo, modifiers.logo, modifiers.clone(), &mut events, KeyCode::Logo);
                add_event(time, last_key_mod.control, modifiers.control, modifiers, &mut events, KeyCode::Control);
                if !events.is_empty() {
                    self.do_callback(&mut events);
                }
            }
            _ => (),
        }
    }

    pub(crate) fn terminate_event_loop(&mut self) {
        self.event_loop_running = false;
    }

    pub(crate) fn event_loop<F>(&mut self, mut event_handler: F)
    where
        F: FnMut(&mut CocoaApp, &mut Vec<Event>) -> bool,
    {
        unsafe {
            let ns_app: id = msg_send![class!(NSApplication), sharedApplication];
            let () = msg_send![ns_app, finishLaunching];

            self.event_callback = Some(
                &mut event_handler as *const dyn FnMut(&mut CocoaApp, &mut Vec<Event>) -> bool
                    as *mut dyn FnMut(&mut CocoaApp, &mut Vec<Event>) -> bool,
            );

            while self.event_loop_running {
                let pool: id = msg_send![class!(NSAutoreleasePool), new];

                let ns_until: id = if self.loop_block {
                    // This blocks the `nextEventMatchingMask:untilDate:inMode:dequeue:` call below until there is a new event.
                    msg_send![class!(NSDate), distantFuture]
                } else {
                    // This doesn't block and instead makes the call return `nil` if there is currently no new event available.
                    msg_send![class!(NSDate), distantPast]
                };
                let ns_event: id = msg_send![
                    ns_app,
                    nextEventMatchingMask: NSEventMask::NSAnyEventMask as u64 | NSEventMask::NSEventMaskPressure as u64
                    untilDate: ns_until
                    inMode: NSDefaultRunLoopMode
                    dequeue: YES
                ];

                if ns_event != nil {
                    // Note that callbacks (e.g. mouse move) are called within here.
                    self.process_ns_event(ns_event);
                }

                // We should paint if `ns_event == nil` because that means that we were not blocking the loop
                // previously, so a repaint was requested in a previous event. If `self.loop_block` is now
                // set we should also paint, because we're about to block the loop, and if we don't pain now
                // then the changes won't be reflected on the screen. If we already have a paint event scheduled
                // then we just shouldn't paint at all right now.
                let mut should_paint = (ns_event == nil || self.loop_block) && !self.scheduled_paint_event;
                if !should_paint {
                    let ev_type: NSEventType = msg_send![ns_event, type];
                    // Injected by `CocoaApp::unblock_event_loop_and_paint`.
                    if ev_type == NSEventType::NSApplicationDefined {
                        should_paint = true;
                    }
                }

                if should_paint {
                    let now = Instant::now();
                    // Rate-limit paint events to 16ms / ~60fps. This is mostly important for if the paint is
                    // is super cheap, because otherwise we end up just busy-waiting all the time!
                    match Duration::from_millis(16).checked_sub(now - self.last_paint_start_time) {
                        Some(time_to_wait) => {
                            self.loop_block = true;
                            self.scheduled_paint_event = true;
                            make_timer(self.timer_delegate_instance, sel!(receivedScheduledPaint:), time_to_wait, false, nil);
                        }
                        None => {
                            self.last_paint_start_time = now;
                            self.do_callback(&mut vec![Event::SystemEvent(SystemEvent::Paint)]);
                        }
                    }
                }

                let () = msg_send![pool, release];
            }
            self.event_callback = None;
        }
    }

    pub(crate) fn do_callback(&mut self, events: &mut Vec<Event>) {
        unsafe {
            if self.event_callback.is_none() || self.event_recur_block {
                return;
            };
            self.event_recur_block = true;
            let callback = self.event_callback.unwrap();
            self.loop_block = (*callback)(self, events);
            self.event_recur_block = false;
        }
    }

    #[cfg(not(feature = "cef"))]
    pub(crate) fn set_mouse_cursor(&mut self, cursor: MouseCursor) {
        if self.current_cursor != cursor {
            self.current_cursor = cursor;
            // todo set it on all windows
            unsafe {
                for (window, view) in &self.cocoa_windows {
                    let _: () = msg_send![
                        *window,
                        invalidateCursorRectsForView: *view
                    ];
                }
            }
        }
    }

    pub(crate) fn send_event_from_any_thread(event: Event) {
        unsafe {
            let pool: id = msg_send![class!(NSAutoreleasePool), new];
            let event_ptr = Box::into_raw(Box::new(event));
            let event_value: id = msg_send![class!(NSValue), valueWithPointer: event_ptr];
            let cocoa_app = &(*GLOBAL_COCOA_APP);
            make_timer(cocoa_app.timer_delegate_instance, sel!(receivedEvent:), Duration::from_secs(0), false, event_value);
            let () = msg_send![pool, release];
        }
    }

    pub(crate) fn start_timer(&mut self, timer_id: u64, interval: f64, repeats: bool) {
        let nstimer =
            make_timer(self.timer_delegate_instance, sel!(receivedTimer:), Duration::from_secs_f64(interval), repeats, nil);
        self.timers.push(CocoaTimer { timer_id, nstimer, repeats });
    }

    pub(crate) fn stop_timer(&mut self, timer_id: u64) {
        for i in 0..self.timers.len() {
            if self.timers[i].timer_id == timer_id {
                unsafe {
                    let () = msg_send![self.timers[i].nstimer, invalidate];
                }
                self.timers.remove(i);
                return;
            }
        }
    }

    /// Break the event loop if its in blocked mode, and paint. We should always
    /// call this after manually calling `do_callback`, because either that event
    /// or the paint event itself might call [`Cx::request_next_frame`] or
    /// [`Cx::request_draw`], which means that we'll need to unblock the event loop.
    fn unblock_event_loop_and_paint() {
        unsafe {
            let pool: id = msg_send![class!(NSAutoreleasePool), new];
            let nsevent: id = msg_send![
                class!(NSEvent),
                otherEventWithType: NSEventType::NSApplicationDefined
                location: NSPoint {x: 0.0, y: 0.0}
                modifierFlags: 0u64
                timestamp: 0f64
                windowNumber: 1u64
                context: nil
                subtype: 0i16
                data1: 0u64
                data2: 0u64
            ];
            let ns_app: id = msg_send![class!(NSApplication), sharedApplication];
            let () = msg_send![ns_app, postEvent: nsevent atStart: NO];
            let () = msg_send![pool, release];
        }
    }

    pub(crate) fn send_timer_received(&mut self, nstimer: id) {
        for i in 0..self.timers.len() {
            if self.timers[i].nstimer == nstimer {
                let timer_id = self.timers[i].timer_id;
                if !self.timers[i].repeats {
                    self.timers.remove(i);
                }
                self.do_callback(&mut vec![Event::Timer(TimerEvent { timer_id })]);
                Self::unblock_event_loop_and_paint();
                return;
            }
        }
    }

    pub(crate) fn send_command_event(&mut self, command: CommandId) {
        self.do_callback(&mut vec![Event::Command(command)]);
        Self::unblock_event_loop_and_paint();
    }

    pub(crate) fn copy_text_to_clipboard(text: &str) {
        unsafe {
            // plug it into the apple clipboard
            let pasteboard: id = msg_send![class!(NSPasteboard), generalPasteboard];
            let ns_string: id = str_to_nsstring(text);
            let array: id = msg_send![class!(NSArray), arrayWithObject: NSStringPboardType];
            let () = msg_send![pasteboard, declareTypes: array owner: nil];
            let () = msg_send![pasteboard, setString: ns_string forType: NSStringPboardType];
        }
    }

    /// Start a regular timer every 30ms, per
    /// https://bitbucket.org/chromiumembedded/cef/issues/2968/documentation-of-external-message-pump
    #[cfg(feature = "cef")]
    pub(crate) fn start_cef_timer(&mut self) {
        make_timer(self.timer_delegate_instance, sel!(receivedCefDoMessageLoopWork:), Duration::from_millis(30), true, nil);
    }

    /// Schedule another timer in addition to our regular timer, in case we need to do something
    /// earlier. See
    /// https://bitbucket.org/chromiumembedded/cef/issues/2968/documentation-of-external-message-pump
    #[cfg(feature = "cef")]
    pub(crate) fn cef_schedule_message_pump_work(delay_ms: i64) {
        unsafe {
            let cocoa_app = &mut (*GLOBAL_COCOA_APP);
            let mut cef_timer = cocoa_app.cef_timer.write().unwrap();
            if *cef_timer != nil {
                let () = msg_send![*cef_timer, invalidate];
            }
            *cef_timer = make_timer(
                cocoa_app.timer_delegate_instance,
                sel!(receivedCefDoMessageLoopWork:),
                Duration::from_millis(delay_ms as u64),
                false,
                nil,
            );
        }
    }
}

impl CocoaWindow {
    pub(crate) fn new(cocoa_app: &mut CocoaApp, window_id: usize) -> CocoaWindow {
        unsafe {
            let pool: id = msg_send![class!(NSAutoreleasePool), new];

            let window: id = msg_send![cocoa_app.window_class, alloc];
            let window_delegate: id = msg_send![cocoa_app.window_delegate_class, new];
            let view: id = msg_send![cocoa_app.view_class, alloc];

            let () = msg_send![pool, drain];
            cocoa_app.cocoa_windows.push((window, view));
            CocoaWindow {
                is_fullscreen: false,
                time_start: cocoa_app.time_start,
                live_resize_timer: nil,
                cocoa_app,
                window_delegate,
                //layer_delegate:layer_delegate,
                window,
                window_id,
                view,
                last_window_geom: None,
                #[cfg(not(feature = "cef"))]
                ime_spot: Vec2::default(),
                pointers_down: Vec::new(),
                last_mouse_pos: Vec2::default(),
            }
        }
    }

    // complete window initialization with pointers to self
    pub(crate) fn init(&mut self, title: &str, size: Vec2, position: Option<Vec2>, add_drop_target_for_app_open_files: bool) {
        unsafe {
            //(*self.cocoa_app).init_app_after_first_window();
            self.pointers_down.resize(NUM_POINTERS, false);

            let pool: id = msg_send![class!(NSAutoreleasePool), new];

            // set the backpointeers
            (*self.window_delegate).set_ivar("cocoa_window_ptr", self as *mut _ as *mut c_void);
            //(*self.layer_delegate).set_ivar("cocoa_window_ptr", self as *mut _ as *mut c_void);
            let () = msg_send![self.view, initWithPtr: self as *mut _ as *mut c_void];

            let left_top = if let Some(position) = position {
                NSPoint { x: position.x as f64, y: position.y as f64 }
            } else {
                NSPoint { x: 0.0, y: 0.0 }
            };
            let ns_size = NSSize { width: size.x as f64, height: size.y as f64 };
            let window_frame = NSRect { origin: left_top, size: ns_size };
            let window_masks = NSWindowStyleMask::NSClosableWindowMask as u64
                | NSWindowStyleMask::NSMiniaturizableWindowMask as u64
                | NSWindowStyleMask::NSResizableWindowMask as u64
                | NSWindowStyleMask::NSTitledWindowMask as u64
                | NSWindowStyleMask::NSFullSizeContentViewWindowMask as u64;

            let () = msg_send![
                self.window,
                initWithContentRect: window_frame
                styleMask: window_masks as u64
                backing: NSBackingStoreType::NSBackingStoreBuffered as u64
                defer: NO
            ];

            let nscolor: id = msg_send![class!(NSColor), colorWithCGColor: CGColorCreateSRGB(0.2, 0.2, 0.2, 1.0)];
            let () = msg_send![self.window, setBackgroundColor: nscolor];
            let () = msg_send![self.window, setDelegate: self.window_delegate];

            let title = str_to_nsstring(title);
            let () = msg_send![self.window, setReleasedWhenClosed: NO];
            let () = msg_send![self.window, setTitle: title];
            let () = msg_send![self.window, setTitleVisibility: NSWindowTitleVisibility::NSWindowTitleHidden];
            let () = msg_send![self.window, setTitlebarAppearsTransparent: YES];

            //let subviews:id = msg_send![self.window, getSubviews];
            //println!("{}", subviews as u64);
            let () = msg_send![self.window, setAcceptsMouseMovedEvents: YES];

            let () = msg_send![self.view, setLayerContentsRedrawPolicy: 2]; //duringViewResize

            // Add `self.view` as a subview to the automatically created `contentView` and set up auto-resizing.
            let content_view: id = msg_send![self.window, contentView];
            let bounds: NSRect = msg_send![content_view, bounds];
            let () = msg_send![self.view, setFrame: bounds];
            let () = msg_send![self.view, setAutoresizingMask: (1 << 4) | (1 << 1)];
            let () = msg_send![content_view, addSubview: self.view];

            let () = msg_send![self.window, makeFirstResponder: self.view];
            let () = msg_send![self.window, makeKeyAndOrderFront: nil];

            if position.is_none() {
                let () = msg_send![self.window, center];
            }
            let input_context: id = msg_send![self.view, inputContext];
            let () = msg_send![input_context, invalidateCharacterCoordinates];

            if add_drop_target_for_app_open_files {
                // Old style, based on https://stackoverflow.com/a/8567836
                let array: id = msg_send![class!(NSArray), arrayWithObject: NSFilenamesPboardType];
                let () = msg_send![self.window, registerForDraggedTypes: array];
            }

            let () = msg_send![pool, drain];
        }
    }

    pub(crate) fn update_ptrs(&mut self) {
        unsafe {
            //(*self.layer_delegate).set_ivar("cocoa_window_ptr", self as *mut _ as *mut c_void);
            (*self.window_delegate).set_ivar("cocoa_window_ptr", self as *mut _ as *mut c_void);
            (*self.view).set_ivar("cocoa_window_ptr", self as *mut _ as *mut c_void);
        }
    }

    #[cfg(not(feature = "cef"))]
    pub(crate) fn set_ime_spot(&mut self, spot: Vec2) {
        self.ime_spot = spot;
    }

    pub(crate) fn start_live_resize(&mut self) {
        unsafe {
            let cocoa_app = &(*self.cocoa_app);
            self.live_resize_timer = make_timer(
                cocoa_app.timer_delegate_instance,
                sel!(receivedPaint:),
                Duration::from_secs_f64(0.01666666),
                true,
                nil,
            );
        }
        let mut events = vec![Event::WindowResizeLoop(WindowResizeLoopEvent { window_id: self.window_id, was_started: true })];
        self.do_callback(&mut events);
    }

    pub(crate) fn end_live_resize(&mut self) {
        unsafe {
            let () = msg_send![self.live_resize_timer, invalidate];
            self.live_resize_timer = nil;
        }
        let mut events = vec![Event::WindowResizeLoop(WindowResizeLoopEvent { window_id: self.window_id, was_started: false })];
        self.do_callback(&mut events);
    }

    pub(crate) fn close_window(&mut self) {
        unsafe {
            (*self.cocoa_app).event_recur_block = false;
            let () = msg_send![self.window, close];
        }
    }

    pub(crate) fn restore(&mut self) {
        unsafe {
            let () = msg_send![self.window, toggleFullScreen: nil];
        }
    }

    pub(crate) fn maximize(&mut self) {
        unsafe {
            let () = msg_send![self.window, toggleFullScreen: nil];
        }
    }

    pub(crate) fn minimize(&mut self) {
        unsafe {
            let () = msg_send![self.window, miniaturize: nil];
        }
    }

    pub(crate) fn set_topmost(&mut self, _topmost: bool) {}

    pub(crate) fn time_now(&self) -> f64 {
        let time_now = Instant::now(); //unsafe {mach_absolute_time()};
        (time_now.duration_since(self.time_start)).as_micros() as f64 / 1_000_000.0
    }

    pub(crate) fn get_window_geom(&self) -> WindowGeom {
        WindowGeom {
            xr_is_presenting: false,
            xr_can_present: false,
            is_topmost: false,
            is_fullscreen: self.is_fullscreen,
            can_fullscreen: false,
            inner_size: self.get_inner_size(),
            outer_size: self.get_outer_size(),
            dpi_factor: self.get_dpi_factor(),
            position: self.get_position(),
        }
    }

    pub(crate) fn do_callback(&mut self, events: &mut Vec<Event>) {
        unsafe {
            (*self.cocoa_app).do_callback(events);
        }
    }

    pub(crate) fn set_position(&mut self, pos: Vec2) {
        let mut window_frame: NSRect = unsafe { msg_send![self.window, frame] };
        window_frame.origin.x = pos.x as f64;
        window_frame.origin.y = pos.y as f64;
        //not very nice: CGDisplay::main().pixels_high() as f64
        unsafe {
            let () = msg_send![self.window, setFrame: window_frame display: YES];
        };
    }

    pub(crate) fn get_position(&self) -> Vec2 {
        let window_frame: NSRect = unsafe { msg_send![self.window, frame] };
        Vec2 { x: window_frame.origin.x as f32, y: window_frame.origin.y as f32 }
    }

    fn get_ime_origin(&self) -> Vec2 {
        let rect = NSRect {
            origin: NSPoint { x: 0.0, y: 0.0 },
            //view_frame.size.height),
            size: NSSize { width: 0.0, height: 0.0 },
        };
        let out: NSRect = unsafe { msg_send![self.window, convertRectToScreen: rect] };
        Vec2 { x: out.origin.x as f32, y: out.origin.y as f32 }
    }

    pub(crate) fn get_inner_size(&self) -> Vec2 {
        let view_frame: NSRect = unsafe { msg_send![self.view, frame] };
        Vec2 { x: view_frame.size.width as f32, y: view_frame.size.height as f32 }
    }

    pub(crate) fn get_outer_size(&self) -> Vec2 {
        let window_frame: NSRect = unsafe { msg_send![self.window, frame] };
        Vec2 { x: window_frame.size.width as f32, y: window_frame.size.height as f32 }
    }

    pub(crate) fn set_outer_size(&self, size: Vec2) {
        let mut window_frame: NSRect = unsafe { msg_send![self.window, frame] };
        window_frame.size.width = size.x as f64;
        window_frame.size.height = size.y as f64;
        unsafe {
            let () = msg_send![self.window, setFrame: window_frame display: YES];
        };
    }

    pub(crate) fn get_dpi_factor(&self) -> f32 {
        let scale: f64 = unsafe { msg_send![self.window, backingScaleFactor] };
        scale as f32
    }

    pub(crate) fn send_change_event(&mut self) {
        //return;
        let new_geom = self.get_window_geom();
        let old_geom = if let Some(old_geom) = &self.last_window_geom { old_geom.clone() } else { new_geom.clone() };
        self.last_window_geom = Some(new_geom.clone());
        self.do_callback(&mut vec![Event::WindowGeomChange(WindowGeomChangeEvent {
            window_id: self.window_id,
            old_geom,
            new_geom,
        })]);
        CocoaApp::unblock_event_loop_and_paint();
        // we should schedule a timer for +16ms another Paint
    }

    pub(crate) fn send_focus_event(&mut self) {
        self.do_callback(&mut vec![Event::AppFocus]);
    }

    pub(crate) fn send_focus_lost_event(&mut self) {
        self.do_callback(&mut vec![Event::AppFocusLost]);
    }

    pub(crate) fn mouse_down_can_drag_window(&mut self) -> bool {
        let mut events = vec![Event::WindowDragQuery(WindowDragQueryEvent {
            window_id: self.window_id,
            abs: self.last_mouse_pos,
            response: WindowDragQueryResponse::NoAnswer,
        })];
        self.do_callback(&mut events);
        if let Event::WindowDragQuery(wd) = &events[0] {
            match &wd.response {
                WindowDragQueryResponse::Client => (),
                WindowDragQueryResponse::Caption | WindowDragQueryResponse::SysMenu => {
                    // we start a window drag
                    return true;
                }
                _ => (),
            }
        }
        false
    }

    pub(crate) fn send_pointer_down(&mut self, digit: usize, button: MouseButton, modifiers: KeyModifiers) {
        self.pointers_down[digit] = true;
        self.do_callback(&mut vec![Event::PointerDown(PointerDownEvent {
            window_id: self.window_id,
            abs: self.last_mouse_pos,
            rel: self.last_mouse_pos,
            rect: Rect::default(),
            digit,
            button,
            handled: false,
            input_type: PointerInputType::Mouse,
            modifiers,
            tap_count: 0,
            time: self.time_now(),
        })]);
    }

    pub(crate) fn send_pointer_up(&mut self, digit: usize, button: MouseButton, modifiers: KeyModifiers) {
        self.pointers_down[digit] = false;
        self.do_callback(&mut vec![Event::PointerUp(PointerUpEvent {
            window_id: self.window_id,
            abs: self.last_mouse_pos,
            rel: self.last_mouse_pos,
            rect: Rect::default(),
            abs_start: Vec2::default(),
            rel_start: Vec2::default(),
            digit,
            button,
            is_over: false,
            input_type: PointerInputType::Mouse,
            modifiers,
            time: self.time_now(),
        })]);
    }

    pub(crate) fn send_pointer_hover_and_move(&mut self, pos: Vec2, modifiers: KeyModifiers) {
        self.last_mouse_pos = pos;
        let mut events = Vec::new();

        unsafe {
            (*self.cocoa_app).startup_focus_hack();
        }

        for (digit, down) in self.pointers_down.iter().enumerate() {
            if *down {
                events.push(Event::PointerMove(PointerMoveEvent {
                    window_id: self.window_id,
                    abs: pos,
                    rel: pos,
                    rect: Rect::default(),
                    digit,
                    abs_start: Vec2::default(),
                    rel_start: Vec2::default(),
                    is_over: false,
                    input_type: PointerInputType::Mouse,
                    modifiers: modifiers.clone(),
                    time: self.time_now(),
                }));
            }
        }
        events.push(Event::PointerHover(PointerHoverEvent {
            digit: 0,
            window_id: self.window_id,
            abs: pos,
            rel: pos,
            any_down: false,
            rect: Rect::default(),
            handled: false,
            hover_state: HoverState::Over,
            modifiers,
            time: self.time_now(),
        }));
        self.do_callback(&mut events);
    }

    pub(crate) fn send_window_close_requested_event(&mut self) -> bool {
        let mut events =
            vec![Event::WindowCloseRequested(WindowCloseRequestedEvent { window_id: self.window_id, accept_close: true })];
        self.do_callback(&mut events);
        if let Event::WindowCloseRequested(cre) = &events[0] {
            return cre.accept_close;
        }
        true
    }

    pub(crate) fn send_window_closed_event(&mut self) {
        self.do_callback(&mut vec![Event::WindowClosed(WindowClosedEvent { window_id: self.window_id })])
    }

    pub(crate) fn send_text_input(&mut self, input: String, replace_last: bool) {
        self.do_callback(&mut vec![Event::TextInput(TextInputEvent { input, was_paste: false, replace_last })])
    }

    pub(crate) fn send_scroll(&mut self, dx: f64, dy: f64, has_prec: bool, modifiers: KeyModifiers) {
        let scroll =
            if has_prec { Vec2 { x: -dx as f32, y: -dy as f32 } } else { Vec2 { x: -dx as f32 * 32., y: -dy as f32 * 32. } };

        self.do_callback(&mut vec![Event::PointerScroll(PointerScrollEvent {
            digit: 0,
            window_id: self.window_id,
            scroll,
            abs: self.last_mouse_pos,
            rel: self.last_mouse_pos,
            rect: Rect::default(),
            input_type: PointerInputType::Mouse,
            modifiers,
            handled_x: false,
            handled_y: false,
            time: self.time_now(),
        })]);
    }
}

fn get_event_char(event: id) -> char {
    unsafe {
        let characters: id = msg_send![event, characters];
        if characters == nil {
            return '\0';
        }
        let chars = nsstring_to_string(characters);

        if chars.is_empty() {
            return '\0';
        }
        chars.chars().next().unwrap()
    }
}

fn get_event_key_modifier(event: id) -> KeyModifiers {
    let flags: u64 = unsafe { msg_send![event, modifierFlags] };
    KeyModifiers {
        shift: flags & NSEventModifierFlags::NSShiftKeyMask as u64 != 0,
        control: flags & NSEventModifierFlags::NSControlKeyMask as u64 != 0,
        alt: flags & NSEventModifierFlags::NSAlternateKeyMask as u64 != 0,
        logo: flags & NSEventModifierFlags::NSCommandKeyMask as u64 != 0,
    }
}

fn get_event_keycode(event: id) -> Option<KeyCode> {
    let scan_code: std::os::raw::c_ushort = unsafe { msg_send![event, keyCode] };

    Some(match scan_code {
        0x00 => KeyCode::KeyA,
        0x01 => KeyCode::KeyS,
        0x02 => KeyCode::KeyD,
        0x03 => KeyCode::KeyF,
        0x04 => KeyCode::KeyH,
        0x05 => KeyCode::KeyG,
        0x06 => KeyCode::KeyZ,
        0x07 => KeyCode::KeyX,
        0x08 => KeyCode::KeyC,
        0x09 => KeyCode::KeyV,
        //0x0a => World 1,
        0x0b => KeyCode::KeyB,
        0x0c => KeyCode::KeyQ,
        0x0d => KeyCode::KeyW,
        0x0e => KeyCode::KeyE,
        0x0f => KeyCode::KeyR,
        0x10 => KeyCode::KeyY,
        0x11 => KeyCode::KeyT,
        0x12 => KeyCode::Key1,
        0x13 => KeyCode::Key2,
        0x14 => KeyCode::Key3,
        0x15 => KeyCode::Key4,
        0x16 => KeyCode::Key6,
        0x17 => KeyCode::Key5,
        0x18 => KeyCode::Equals,
        0x19 => KeyCode::Key9,
        0x1a => KeyCode::Key7,
        0x1b => KeyCode::Minus,
        0x1c => KeyCode::Key8,
        0x1d => KeyCode::Key0,
        0x1e => KeyCode::RBracket,
        0x1f => KeyCode::KeyO,
        0x20 => KeyCode::KeyU,
        0x21 => KeyCode::LBracket,
        0x22 => KeyCode::KeyI,
        0x23 => KeyCode::KeyP,
        0x24 => KeyCode::Return,
        0x25 => KeyCode::KeyL,
        0x26 => KeyCode::KeyJ,
        0x27 => KeyCode::Backtick,
        0x28 => KeyCode::KeyK,
        0x29 => KeyCode::Semicolon,
        0x2a => KeyCode::Backslash,
        0x2b => KeyCode::Comma,
        0x2c => KeyCode::Slash,
        0x2d => KeyCode::KeyN,
        0x2e => KeyCode::KeyM,
        0x2f => KeyCode::Period,
        0x30 => KeyCode::Tab,
        0x31 => KeyCode::Space,
        0x32 => KeyCode::Backtick,
        0x33 => KeyCode::Backspace,
        //0x34 => unkown,
        0x35 => KeyCode::Escape,
        //0x36 => KeyCode::RLogo,
        //0x37 => KeyCode::LLogo,
        //0x38 => KeyCode::LShift,
        0x39 => KeyCode::Capslock,
        //0x3a => KeyCode::LAlt,
        //0x3b => KeyCode::LControl,
        //0x3c => KeyCode::RShift,
        //0x3d => KeyCode::RAlt,
        //0x3e => KeyCode::RControl,
        //0x3f => Fn key,
        //0x40 => KeyCode::F17,
        0x41 => KeyCode::NumpadDecimal,
        //0x42 -> unkown,
        0x43 => KeyCode::NumpadMultiply,
        //0x44 => unkown,
        0x45 => KeyCode::NumpadAdd,
        //0x46 => unkown,
        0x47 => KeyCode::Numlock,
        //0x48 => KeypadClear,
        //0x49 => KeyCode::VolumeUp,
        //0x4a => KeyCode::VolumeDown,
        0x4b => KeyCode::NumpadDivide,
        0x4c => KeyCode::NumpadEnter,
        0x4e => KeyCode::NumpadSubtract,
        //0x4d => unkown,
        //0x4e => KeyCode::Subtract,
        //0x4f => KeyCode::F18,
        //0x50 => KeyCode::F19,
        0x51 => KeyCode::NumpadEquals,
        0x52 => KeyCode::Numpad0,
        0x53 => KeyCode::Numpad1,
        0x54 => KeyCode::Numpad2,
        0x55 => KeyCode::Numpad3,
        0x56 => KeyCode::Numpad4,
        0x57 => KeyCode::Numpad5,
        0x58 => KeyCode::Numpad6,
        0x59 => KeyCode::Numpad7,
        //0x5a => KeyCode::F20,
        0x5b => KeyCode::Numpad8,
        0x5c => KeyCode::Numpad9,
        //0x5d => KeyCode::Yen,
        //0x5e => JIS Ro,
        //0x5f => unkown,
        0x60 => KeyCode::F5,
        0x61 => KeyCode::F6,
        0x62 => KeyCode::F7,
        0x63 => KeyCode::F3,
        0x64 => KeyCode::F8,
        0x65 => KeyCode::F9,
        //0x66 => JIS Eisuu (macOS),
        0x67 => KeyCode::F11,
        //0x68 => JIS Kana (macOS),
        0x69 => KeyCode::PrintScreen,
        //0x6a => KeyCode::F16,
        //0x6b => KeyCode::F14,
        //0x6c => unkown,
        0x6d => KeyCode::F10,
        //0x6e => unkown,
        0x6f => KeyCode::F12,
        //0x70 => unkown,
        //0x71 => KeyCode::F15,
        0x72 => KeyCode::Insert,
        0x73 => KeyCode::Home,
        0x74 => KeyCode::PageUp,
        0x75 => KeyCode::Delete,
        0x76 => KeyCode::F4,
        0x77 => KeyCode::End,
        0x78 => KeyCode::F2,
        0x79 => KeyCode::PageDown,
        0x7a => KeyCode::F1,
        0x7b => KeyCode::ArrowLeft,
        0x7c => KeyCode::ArrowRight,
        0x7d => KeyCode::ArrowDown,
        0x7e => KeyCode::ArrowUp,
        //0x7f =>  unkown,
        //0xa => KeyCode::Caret,
        _ => return None,
    })
}

fn keycode_to_menu_key(keycode: KeyCode, shift: bool) -> &'static str {
    if !shift {
        match keycode {
            KeyCode::Backtick => "`",
            KeyCode::Key0 => "0",
            KeyCode::Key1 => "1",
            KeyCode::Key2 => "2",
            KeyCode::Key3 => "3",
            KeyCode::Key4 => "4",
            KeyCode::Key5 => "5",
            KeyCode::Key6 => "6",
            KeyCode::Key7 => "7",
            KeyCode::Key8 => "8",
            KeyCode::Key9 => "9",
            KeyCode::Minus => "-",
            KeyCode::Equals => "=",

            KeyCode::KeyQ => "q",
            KeyCode::KeyW => "w",
            KeyCode::KeyE => "e",
            KeyCode::KeyR => "r",
            KeyCode::KeyT => "t",
            KeyCode::KeyY => "y",
            KeyCode::KeyU => "u",
            KeyCode::KeyI => "i",
            KeyCode::KeyO => "o",
            KeyCode::KeyP => "p",
            KeyCode::LBracket => "[",
            KeyCode::RBracket => "]",

            KeyCode::KeyA => "a",
            KeyCode::KeyS => "s",
            KeyCode::KeyD => "d",
            KeyCode::KeyF => "f",
            KeyCode::KeyG => "g",
            KeyCode::KeyH => "h",
            KeyCode::KeyJ => "j",
            KeyCode::KeyK => "l",
            KeyCode::KeyL => "l",
            KeyCode::Semicolon => ";",
            KeyCode::Quote => "'",
            KeyCode::Backslash => "\\",

            KeyCode::KeyZ => "z",
            KeyCode::KeyX => "x",
            KeyCode::KeyC => "c",
            KeyCode::KeyV => "v",
            KeyCode::KeyB => "b",
            KeyCode::KeyN => "n",
            KeyCode::KeyM => "m",
            KeyCode::Comma => ",",
            KeyCode::Period => ".",
            KeyCode::Slash => "/",
            _ => "",
        }
    } else {
        match keycode {
            KeyCode::Backtick => "~",
            KeyCode::Key0 => "!",
            KeyCode::Key1 => "@",
            KeyCode::Key2 => "#",
            KeyCode::Key3 => "$",
            KeyCode::Key4 => "%",
            KeyCode::Key5 => "^",
            KeyCode::Key6 => "&",
            KeyCode::Key7 => "*",
            KeyCode::Key8 => "(",
            KeyCode::Key9 => ")",
            KeyCode::Minus => "_",
            KeyCode::Equals => "+",

            KeyCode::KeyQ => "Q",
            KeyCode::KeyW => "W",
            KeyCode::KeyE => "E",
            KeyCode::KeyR => "R",
            KeyCode::KeyT => "T",
            KeyCode::KeyY => "Y",
            KeyCode::KeyU => "U",
            KeyCode::KeyI => "I",
            KeyCode::KeyO => "O",
            KeyCode::KeyP => "P",
            KeyCode::LBracket => "{",
            KeyCode::RBracket => "}",

            KeyCode::KeyA => "A",
            KeyCode::KeyS => "S",
            KeyCode::KeyD => "D",
            KeyCode::KeyF => "F",
            KeyCode::KeyG => "G",
            KeyCode::KeyH => "H",
            KeyCode::KeyJ => "J",
            KeyCode::KeyK => "K",
            KeyCode::KeyL => "L",
            KeyCode::Semicolon => ":",
            KeyCode::Quote => "\"",
            KeyCode::Backslash => "|",

            KeyCode::KeyZ => "Z",
            KeyCode::KeyX => "X",
            KeyCode::KeyC => "C",
            KeyCode::KeyV => "V",
            KeyCode::KeyB => "B",
            KeyCode::KeyN => "N",
            KeyCode::KeyM => "M",
            KeyCode::Comma => "<",
            KeyCode::Period => ">",
            KeyCode::Slash => "?",
            _ => "",
        }
    }
}

#[allow(clippy::mut_from_ref)]
fn get_cocoa_window(this: &Object) -> &mut CocoaWindow {
    unsafe {
        let ptr: *mut c_void = *this.get_ivar("cocoa_window_ptr");
        &mut *(ptr as *mut CocoaWindow)
    }
}

#[allow(clippy::mut_from_ref)]
fn get_cocoa_app(this: &Object) -> &mut CocoaApp {
    unsafe {
        let ptr: *mut c_void = *this.get_ivar("cocoa_app_ptr");
        &mut *(ptr as *mut CocoaApp)
    }
}

pub(crate) fn define_cocoa_timer_delegate() -> *const Class {
    extern "C" fn received_timer(this: &Object, _: Sel, nstimer: id) {
        let ca = get_cocoa_app(this);
        ca.send_timer_received(nstimer);
    }

    extern "C" fn received_scheduled_paint(this: &Object, _: Sel, _nstimer: id) {
        let ca = get_cocoa_app(this);
        ca.scheduled_paint_event = false;
        CocoaApp::unblock_event_loop_and_paint();
    }

    extern "C" fn received_paint(this: &Object, _: Sel, _nstimer: id) {
        let ca = get_cocoa_app(this);
        ca.do_callback(&mut vec![Event::SystemEvent(SystemEvent::Paint)]);
        CocoaApp::unblock_event_loop_and_paint();
    }

    extern "C" fn received_event(this: &Object, _: Sel, nstimer: id) {
        let ca = get_cocoa_app(this);
        unsafe {
            let event_value: id = msg_send![nstimer, userInfo];
            let event_ptr: *mut Event = msg_send![event_value, pointerValue];
            let event_box = Box::from_raw(event_ptr);
            ca.do_callback(&mut vec![event_box.as_ref().clone()]);
            CocoaApp::unblock_event_loop_and_paint();
        };
    }

    #[cfg(feature = "cef")]
    extern "C" fn received_cef_do_message_loop_work(this: &Object, _: Sel, _nstimer: id) {
        let ca = get_cocoa_app(this);
        unsafe {
            // Cancel the other CEF timer if it exists, if we happend to have our regular
            // timer fire early!
            let mut cef_timer = ca.cef_timer.write().unwrap();
            if *cef_timer != nil {
                let () = msg_send![*cef_timer, invalidate];
                *cef_timer = nil;
            }
        }
        ca.do_callback(&mut vec![Event::SystemEvent(SystemEvent::CefDoMessageLoopWork)]);
        // No need to call CocoaApp::unblock_event_loop_and_paint since this is an internal event that shouldn't
        // ever case a repaint.
    }

    let superclass = class!(NSObject);
    let mut decl = ClassDecl::new("TimerDelegate", superclass).unwrap();

    // Add callback methods
    unsafe {
        decl.add_method(sel!(receivedTimer:), received_timer as extern "C" fn(&Object, Sel, id));
        decl.add_method(sel!(receivedScheduledPaint:), received_scheduled_paint as extern "C" fn(&Object, Sel, id));
        decl.add_method(sel!(receivedPaint:), received_paint as extern "C" fn(&Object, Sel, id));
        decl.add_method(sel!(receivedEvent:), received_event as extern "C" fn(&Object, Sel, id));
        #[cfg(feature = "cef")]
        decl.add_method(
            sel!(receivedCefDoMessageLoopWork:),
            received_cef_do_message_loop_work as extern "C" fn(&Object, Sel, id),
        );
    }
    // Store internal state as user data
    decl.add_ivar::<*mut c_void>("cocoa_app_ptr");

    decl.register()
}

pub(crate) fn define_app_delegate() -> *const Class {
    extern "C" fn open_files(this: &Object, _: Sel, sender_app: id, filenames: id) {
        // TODO(JP): We probably have to do some debouncing at some point per https://stackoverflow.com/q/37623734
        // (but let's not worry about that until we actually want to deal with multiple file types..)

        unsafe {
            let count: u64 = msg_send![filenames, count];
            let user_files: Vec<UserFile> = (0..count)
                .map(|i| {
                    let path = nsstring_to_string(msg_send![filenames, objectAtIndex: i]);
                    let basename = Path::new(&path).file_name().unwrap().to_str().unwrap().to_string();
                    UserFile { basename, file: UniversalFile::open(&path).unwrap() }
                })
                .collect();

            if !user_files.is_empty() {
                let ca = get_cocoa_app(this);
                ca.do_callback(&mut vec![Event::AppOpenFiles(AppOpenFilesEvent { user_files })]);
                CocoaApp::unblock_event_loop_and_paint();
            }

            msg_send![sender_app, replyToOpenOrPrint: NSApplicationDelegateReply::NSApplicationDelegateReplySuccess]
        }
    }

    let superclass = class!(NSObject);
    let mut decl = ClassDecl::new("AppDelegate", superclass).unwrap();
    unsafe {
        decl.add_method(sel!(application:openFiles:), open_files as extern "C" fn(&Object, Sel, id, id));
    }
    decl.add_ivar::<*mut c_void>("cocoa_app_ptr");
    decl.register()
}

#[cfg(feature = "cef")]
pub(crate) fn define_ns_application() -> *const Class {
    extern "C" fn is_handling_send_event(_this: &Object, _: Sel) -> bool {
        let cocoa_app = unsafe { &(*GLOBAL_COCOA_APP) };
        cocoa_app.cef_is_handling_event
    }

    extern "C" fn set_handling_send_event(_this: &Object, _: Sel, handling_event: bool) {
        let mut cocoa_app = unsafe { &mut (*GLOBAL_COCOA_APP) };
        cocoa_app.cef_is_handling_event = handling_event;
    }

    let superclass = class!(NSApplication);
    let mut decl = ClassDecl::new("ZapApplication", superclass).unwrap();
    unsafe {
        decl.add_method(sel!(isHandlingSendEvent), is_handling_send_event as extern "C" fn(&Object, Sel) -> bool);
        decl.add_method(sel!(setHandlingSendEvent:), set_handling_send_event as extern "C" fn(&Object, Sel, bool));
    }
    decl.register()
}

pub(crate) fn define_menu_target_class() -> *const Class {
    extern "C" fn menu_action(this: &Object, _sel: Sel, _item: id) {
        //println!("markedRange");
        let ca = get_cocoa_app(this);
        unsafe {
            let command_usize: usize = *this.get_ivar("command_usize");

            // Panic if we are not 64-bit
            let cmd = LocationHash(command_usize.try_into().unwrap());
            ca.send_command_event(cmd);
        }
    }

    let superclass = class!(NSObject);
    let mut decl = ClassDecl::new("MenuTarget", superclass).unwrap();
    unsafe {
        decl.add_method(sel!(menuAction:), menu_action as extern "C" fn(&Object, Sel, id));
    }
    decl.add_ivar::<*mut c_void>("cocoa_app_ptr");
    decl.add_ivar::<usize>("command_usize");
    decl.register()
}

pub(crate) fn define_menu_delegate() -> *const Class {
    // NSMenuDelegate protocol
    extern "C" fn menu_will_open(this: &Object, _sel: Sel, _item: id) {
        //println!("markedRange");
        let _ca = get_cocoa_app(this);
    }

    let superclass = class!(NSObject);
    let mut decl = ClassDecl::new("MenuDelegate", superclass).unwrap();
    unsafe {
        decl.add_method(sel!(menuWillOpen:), menu_will_open as extern "C" fn(&Object, Sel, id));
    }
    decl.add_ivar::<*mut c_void>("cocoa_app_ptr");
    decl.add_protocol(Protocol::get("NSMenuDelegate").unwrap());
    decl.register()
}

pub(crate) fn define_cocoa_window_delegate() -> *const Class {
    extern "C" fn window_should_close(this: &Object, _: Sel, _: id) -> BOOL {
        let cw = get_cocoa_window(this);
        if cw.send_window_close_requested_event() {
            YES
        } else {
            NO
        }
    }

    extern "C" fn window_will_close(this: &Object, _: Sel, _: id) {
        let cw = get_cocoa_window(this);
        cw.send_window_closed_event();
    }

    extern "C" fn window_did_resize(this: &Object, _: Sel, _: id) {
        let _cw = get_cocoa_window(this);
        //cw.send_change_event();
    }

    extern "C" fn window_will_start_live_resize(this: &Object, _: Sel, _: id) {
        let cw = get_cocoa_window(this);
        cw.start_live_resize();
    }

    extern "C" fn window_did_end_live_resize(this: &Object, _: Sel, _: id) {
        let cw = get_cocoa_window(this);
        cw.end_live_resize();
    }

    /// This won't be triggered if the move was part of a resize.
    extern "C" fn window_did_move(this: &Object, _: Sel, _: id) {
        let cw = get_cocoa_window(this);
        cw.send_change_event();
    }

    extern "C" fn window_changed_screen(this: &Object, _: Sel, _: id) {
        let cw = get_cocoa_window(this);
        cw.send_change_event();
    }

    /// This will always be called before [`window_changed_screen`].
    extern "C" fn window_changed_backing_properties(this: &Object, _: Sel, _: id) {
        let cw = get_cocoa_window(this);
        cw.send_change_event();
    }

    extern "C" fn window_did_become_key(this: &Object, _: Sel, _: id) {
        let cw = get_cocoa_window(this);
        cw.send_focus_event();
    }

    extern "C" fn window_did_resign_key(this: &Object, _: Sel, _: id) {
        let cw = get_cocoa_window(this);
        cw.send_focus_lost_event();
    }

    /// Invoked when the dragged file enters destination bounds or frame
    extern "C" fn dragging_entered(this: &Object, _: Sel, _sender: id) -> BOOL {
        let cw = get_cocoa_window(this);
        cw.do_callback(&mut vec![Event::FileDragBegin]);

        CocoaApp::unblock_event_loop_and_paint();

        YES
    }

    /// Invoked when the dragged file is updated. As of now this is only
    /// used for mouse position updates
    extern "C" fn dragging_updated(this: &Object, _: Sel, sender: id) -> BOOL {
        let cw = get_cocoa_window(this);
        unsafe {
            let view_point: NSPoint = msg_send![sender, draggingLocation];
            let view_rect: NSRect = msg_send![cw.view, frame];
            cw.do_callback(&mut vec![Event::FileDragUpdate(FileDragUpdateEvent {
                abs: Vec2 { x: view_point.x as f32, y: view_rect.size.height as f32 - view_point.y as f32 },
            })]);
        }

        YES
    }

    /// Invoked when the file is released
    extern "C" fn prepare_for_drag_operation(_: &Object, _: Sel, _: id) -> BOOL {
        YES
    }

    /// Invoked after the released file has been removed from the screen
    extern "C" fn perform_drag_operation(_this: &Object, _: Sel, sender: id) -> BOOL {
        unsafe {
            // Redirect filenames to `openFiles`, per https://stackoverflow.com/a/8567836
            let pboard: id = msg_send![sender, draggingPasteboard];
            let filenames: id = msg_send![pboard, propertyListForType: NSFilenamesPboardType];
            let count: u64 = msg_send![filenames, count];
            if count > 0 {
                let ns_app: id = msg_send![class!(NSApplication), sharedApplication];
                let app_delegate: id = msg_send![ns_app, delegate];
                let () = msg_send![app_delegate, application: ns_app openFiles: filenames];
            }
        }
        YES
    }

    /// Invoked when the dragging operation is complete
    extern "C" fn conclude_drag_operation(_: &Object, _: Sel, _: id) {}

    /// Invoked when the dragging operation is cancelled
    extern "C" fn dragging_exited(this: &Object, _: Sel, _: id) {
        let cw = get_cocoa_window(this);
        cw.do_callback(&mut vec![Event::FileDragCancel]);
    }

    /// Invoked when entered fullscreen
    extern "C" fn window_did_enter_fullscreen(this: &Object, _: Sel, _: id) {
        let cw = get_cocoa_window(this);
        cw.is_fullscreen = true;
        cw.send_change_event();
    }

    /// Invoked when before enter fullscreen
    extern "C" fn window_will_enter_fullscreen(this: &Object, _: Sel, _: id) {
        let _cw = get_cocoa_window(this);
    }

    /// Invoked when exited fullscreen
    extern "C" fn window_did_exit_fullscreen(this: &Object, _: Sel, _: id) {
        let cw = get_cocoa_window(this);
        cw.is_fullscreen = false;
        cw.send_change_event();
    }

    extern "C" fn window_did_fail_to_enter_fullscreen(_this: &Object, _: Sel, _: id) {}

    let superclass = class!(NSObject);
    let mut decl = ClassDecl::new("RenderWindowDelegate", superclass).unwrap();

    // Add callback methods
    unsafe {
        decl.add_method(sel!(windowShouldClose:), window_should_close as extern "C" fn(&Object, Sel, id) -> BOOL);
        decl.add_method(sel!(windowWillClose:), window_will_close as extern "C" fn(&Object, Sel, id));
        decl.add_method(sel!(windowDidResize:), window_did_resize as extern "C" fn(&Object, Sel, id));
        decl.add_method(sel!(windowWillStartLiveResize:), window_will_start_live_resize as extern "C" fn(&Object, Sel, id));
        decl.add_method(sel!(windowDidEndLiveResize:), window_did_end_live_resize as extern "C" fn(&Object, Sel, id));

        decl.add_method(sel!(windowDidMove:), window_did_move as extern "C" fn(&Object, Sel, id));
        decl.add_method(sel!(windowChangedScreen:), window_changed_screen as extern "C" fn(&Object, Sel, id));
        decl.add_method(
            sel!(windowChangedBackingProperties:),
            window_changed_backing_properties as extern "C" fn(&Object, Sel, id),
        );
        decl.add_method(sel!(windowDidBecomeKey:), window_did_become_key as extern "C" fn(&Object, Sel, id));
        decl.add_method(sel!(windowDidResignKey:), window_did_resign_key as extern "C" fn(&Object, Sel, id));

        // callbacks for drag and drop events
        decl.add_method(sel!(draggingEntered:), dragging_entered as extern "C" fn(&Object, Sel, id) -> BOOL);
        decl.add_method(sel!(draggingUpdated:), dragging_updated as extern "C" fn(&Object, Sel, id) -> BOOL);
        decl.add_method(sel!(prepareForDragOperation:), prepare_for_drag_operation as extern "C" fn(&Object, Sel, id) -> BOOL);
        decl.add_method(sel!(performDragOperation:), perform_drag_operation as extern "C" fn(&Object, Sel, id) -> BOOL);
        decl.add_method(sel!(concludeDragOperation:), conclude_drag_operation as extern "C" fn(&Object, Sel, id));
        decl.add_method(sel!(draggingExited:), dragging_exited as extern "C" fn(&Object, Sel, id));

        // callbacks for fullscreen events
        decl.add_method(sel!(windowDidEnterFullScreen:), window_did_enter_fullscreen as extern "C" fn(&Object, Sel, id));
        decl.add_method(sel!(windowWillEnterFullScreen:), window_will_enter_fullscreen as extern "C" fn(&Object, Sel, id));
        decl.add_method(sel!(windowDidExitFullScreen:), window_did_exit_fullscreen as extern "C" fn(&Object, Sel, id));
        decl.add_method(
            sel!(windowDidFailToEnterFullScreen:),
            window_did_fail_to_enter_fullscreen as extern "C" fn(&Object, Sel, id),
        );
        // custom timer fn
        //decl.add_method(sel!(windowReceivedTimer:), window_received_timer as extern fn(&Object, Sel, id));
    }
    // Store internal state as user data
    decl.add_ivar::<*mut c_void>("cocoa_window_ptr");

    decl.register()
}

pub(crate) fn define_cocoa_window_class() -> *const Class {
    extern "C" fn yes(_: &Object, _: Sel) -> BOOL {
        YES
    }

    extern "C" fn is_movable_by_window_background(_: &Object, _: Sel) -> BOOL {
        YES
    }

    /// Override `[NSWindow sendEvent:]` in order to process most mouse events here
    /// We do it this way in order to ensure we always have a valid `window` instance
    /// (which is not always true when processing events earlier).
    extern "C" fn send_event(this: &Object, _sel: Sel, ns_event: id) {
        let ev_type: NSEventType = unsafe { msg_send![ns_event, type] };

        let cocoa_window = unsafe {
            let window_delegate: id = msg_send![this, delegate];
            let ptr: *mut c_void = *(*window_delegate).get_ivar("cocoa_window_ptr");
            &mut *(ptr as *mut CocoaWindow)
        };

        match ev_type {
            NSEventType::NSMouseEntered => {}
            NSEventType::NSMouseExited => {}

            NSEventType::NSMouseMoved
            | NSEventType::NSLeftMouseDragged
            | NSEventType::NSOtherMouseDragged
            | NSEventType::NSRightMouseDragged => unsafe {
                // mouse_pos_from_event
                let view: id = cocoa_window.view;
                let window_point: NSPoint = msg_send![ns_event, locationInWindow];
                let view_point: NSPoint = msg_send![view, convertPoint: window_point fromView: nil];
                let view_rect: NSRect = msg_send![view, frame];
                let pos = Vec2 { x: view_point.x as f32, y: view_rect.size.height as f32 - view_point.y as f32 };

                // mouse_motion
                let modifiers = get_event_key_modifier(ns_event);
                cocoa_window.send_pointer_hover_and_move(pos, modifiers);
            },
            NSEventType::NSLeftMouseDown => {
                let modifiers = get_event_key_modifier(ns_event);
                cocoa_window.send_pointer_down(0, MouseButton::Left, modifiers);
            }
            NSEventType::NSLeftMouseUp => {
                let modifiers = get_event_key_modifier(ns_event);
                cocoa_window.send_pointer_up(0, MouseButton::Left, modifiers);
            }
            NSEventType::NSRightMouseDown => {
                let modifiers = get_event_key_modifier(ns_event);
                cocoa_window.send_pointer_down(1, MouseButton::Right, modifiers);
            }
            NSEventType::NSRightMouseUp => {
                let modifiers = get_event_key_modifier(ns_event);
                cocoa_window.send_pointer_up(1, MouseButton::Right, modifiers);
            }
            NSEventType::NSOtherMouseDown => {
                let modifiers = get_event_key_modifier(ns_event);
                cocoa_window.send_pointer_down(2, MouseButton::Other, modifiers);
            }
            NSEventType::NSOtherMouseUp => {
                let modifiers = get_event_key_modifier(ns_event);
                cocoa_window.send_pointer_up(2, MouseButton::Other, modifiers);
            }
            NSEventType::NSScrollWheel => {
                let dx: f64 = unsafe { msg_send![ns_event, scrollingDeltaX] };
                let dy: f64 = unsafe { msg_send![ns_event, scrollingDeltaY] };
                let has_prec: BOOL = unsafe { msg_send![ns_event, hasPreciseScrollingDeltas] };
                let modifiers = get_event_key_modifier(ns_event);

                cocoa_window.send_scroll(dx, dy, has_prec == YES, modifiers);
            }
            _ => (),
        }

        unsafe {
            // Always send the events to the superclass so system events are properly handled.
            let superclass = superclass(this);
            let () = msg_send![super(this, superclass), sendEvent: ns_event];
        }
    }

    /// When running CEF, we have all keyboard events go through the browser, so we never want our NSView
    /// to become the "firstResponder".
    #[cfg(feature = "cef")]
    extern "C" fn make_first_responder_exclude_nsview_for_cef(this: &Object, _sel: Sel, responder: id) {
        unsafe {
            let responder_class: id = msg_send![responder, class];
            let responder_class_name = nsstring_to_string(NSStringFromClass(responder_class));
            if responder_class_name != NSVIEW_CLASS_NAME {
                let superclass = superclass(this);
                let () = msg_send![super(this, superclass), makeFirstResponder: responder];
            }
        }
    }

    let window_superclass = class!(NSWindow);
    let mut decl = ClassDecl::new("RenderWindow", window_superclass).unwrap();
    unsafe {
        decl.add_method(sel!(canBecomeMainWindow), yes as extern "C" fn(&Object, Sel) -> BOOL);
        decl.add_method(sel!(canBecomeKeyWindow), yes as extern "C" fn(&Object, Sel) -> BOOL);
        decl.add_method(sel!(sendEvent:), send_event as extern "C" fn(&Object, Sel, id));

        #[cfg(feature = "cef")]
        decl.add_method(
            sel!(makeFirstResponder:),
            make_first_responder_exclude_nsview_for_cef as extern "C" fn(&Object, Sel, id),
        );
    }
    decl.register()
}

pub(crate) fn define_cocoa_view_class() -> *const Class {
    let mut decl = ClassDecl::new(NSVIEW_CLASS_NAME, class!(NSView)).unwrap();

    extern "C" fn dealloc(this: &Object, _sel: Sel) {
        unsafe {
            let marked_text: id = *this.get_ivar("markedText");
            let _: () = msg_send![marked_text, release];
        }
    }

    extern "C" fn init_with_ptr(this: &Object, _sel: Sel, cx: *mut c_void) -> id {
        unsafe {
            let this: id = msg_send![this, init];
            if this != nil {
                (*this).set_ivar("cocoa_window_ptr", cx);
                let marked_text = <id as NSMutableAttributedString>::init(NSMutableAttributedString::alloc(nil));
                (*this).set_ivar("markedText", marked_text);
            }
            this
        }
    }

    extern "C" fn draw_rect(this: &Object, _sel: Sel, rect: NSRect) {
        let _cw = get_cocoa_window(this);
        unsafe {
            let superclass = superclass(this);
            let () = msg_send![super(this, superclass), drawRect: rect];
        }
    }

    extern "C" fn display_layer(this: &Object, _: Sel, _calayer: id) {
        let cw = get_cocoa_window(this);
        cw.send_change_event();
    }

    // So you don't get annoying noises when pressing e.g. cmd+c.
    extern "C" fn do_command_by_selector(_this: &Object, _sel: Sel, _command: Sel) {}

    extern "C" fn hit_test(_this: &Object, _: Sel, _point: NSPoint) -> id {
        nil
    }

    unsafe {
        decl.add_method(sel!(dealloc), dealloc as extern "C" fn(&Object, Sel));
        decl.add_method(sel!(initWithPtr:), init_with_ptr as extern "C" fn(&Object, Sel, *mut c_void) -> id);
        decl.add_method(sel!(drawRect:), draw_rect as extern "C" fn(&Object, Sel, NSRect));
        decl.add_method(sel!(displayLayer:), display_layer as extern "C" fn(&Object, Sel, id));
        decl.add_method(sel!(doCommandBySelector:), do_command_by_selector as extern "C" fn(&Object, Sel, Sel));
        decl.add_method(sel!(hitTest:), hit_test as extern "C" fn(&Object, Sel, NSPoint) -> id);
    }

    decl.add_ivar::<*mut c_void>("cocoa_window_ptr");
    decl.add_protocol(Protocol::get("CALayerDelegate").unwrap());
    decl.add_ivar::<id>("markedText");

    // When using CEF we have all cursor and keyboard handling go through the browser.
    #[cfg(not(feature = "cef"))]
    {
        extern "C" fn reset_cursor_rects(this: &Object, _sel: Sel) {
            let cw = get_cocoa_window(this);
            unsafe {
                let cocoa_app = &mut (*cw.cocoa_app);
                let current_cursor = cocoa_app.current_cursor.clone();
                let cursor_id =
                    *cocoa_app.cursors.entry(current_cursor.clone()).or_insert_with(|| load_mouse_cursor(current_cursor.clone()));
                let bounds: NSRect = msg_send![this, bounds];
                let _: () = msg_send![
                    this,
                    addCursorRect: bounds
                    cursor: cursor_id
                ];
            }
        }

        /// NSTextInput protocol
        extern "C" fn marked_range(this: &Object, _sel: Sel) -> NSRange {
            //println!("markedRange");
            unsafe {
                let marked_text: id = *this.get_ivar("markedText");
                let length = marked_text.length();
                if length > 0 {
                    NSRange { location: 0, length: length - 1 }
                } else {
                    NSRange { location: i64::max_value() as u64, length: 0 }
                }
            }
        }

        extern "C" fn selected_range(_this: &Object, _sel: Sel) -> NSRange {
            NSRange { location: 0, length: 1 }
        }

        extern "C" fn has_marked_text(this: &Object, _sel: Sel) -> BOOL {
            unsafe {
                let marked_text: id = *this.get_ivar("markedText");
                (marked_text.length() > 0) as BOOL
            }
        }

        extern "C" fn set_marked_text(
            this: &mut Object,
            _sel: Sel,
            string: id,
            _selected_range: NSRange,
            _replacement_range: NSRange,
        ) {
            unsafe {
                let marked_text_ref: &mut id = this.get_mut_ivar("markedText");
                let _: () = msg_send![(*marked_text_ref), release];
                let marked_text = NSMutableAttributedString::alloc(nil);
                let has_attr = msg_send![string, isKindOfClass: class!(NSAttributedString)];
                if has_attr {
                    marked_text.init_with_attributed_string(string);
                } else {
                    marked_text.init_with_string(string);
                };
                *marked_text_ref = marked_text;
            }
        }

        extern "C" fn unmark_text(this: &Object, _sel: Sel) {
            let cw = get_cocoa_window(this);
            unsafe {
                let cocoa_app = &(*cw.cocoa_app);
                let marked_text: id = *this.get_ivar("markedText");
                let mutable_string = marked_text.mutable_string();
                let _: () = msg_send![mutable_string, setString: cocoa_app.const_empty_string];
                let input_context: id = msg_send![this, inputContext];
                let _: () = msg_send![input_context, discardMarkedText];
            }
        }

        extern "C" fn valid_attributes_for_marked_text(this: &Object, _sel: Sel) -> id {
            let cw = get_cocoa_window(this);
            unsafe {
                let cocoa_app = &(*cw.cocoa_app);
                cocoa_app.const_attributes_for_marked_text
            }
        }

        extern "C" fn attributed_substring_for_proposed_range(
            _this: &Object,
            _sel: Sel,
            _range: NSRange,
            _actual_range: *mut c_void,
        ) -> id {
            nil
        }

        extern "C" fn character_index_for_point(_this: &Object, _sel: Sel, _point: NSPoint) -> u64 {
            // println!("character_index_for_point");
            0
        }

        extern "C" fn first_rect_for_character_range(
            this: &Object,
            _sel: Sel,
            _range: NSRange,
            _actual_range: *mut c_void,
        ) -> NSRect {
            let cw = get_cocoa_window(this);

            let view: id = this as *const _ as *mut _;
            //let window_point = event.locationInWindow();
            //et view_point = view.convertPoint_fromView_(window_point, nil);
            let view_rect: NSRect = unsafe { msg_send![view, frame] };
            let window_rect: NSRect = unsafe { msg_send![cw.window, frame] };

            let origin = cw.get_ime_origin();
            let bar = (window_rect.size.height - view_rect.size.height) as f32 - 5.;
            NSRect {
                origin: NSPoint {
                    x: (origin.x + cw.ime_spot.x) as f64,
                    y: (origin.y + (view_rect.size.height as f32 - cw.ime_spot.y - bar)) as f64,
                },
                // as _, y as _),
                size: NSSize { width: 0.0, height: 0.0 },
            }
        }

        extern "C" fn insert_text(this: &Object, _sel: Sel, string: id, replacement_range: NSRange) {
            let cw = get_cocoa_window(this);
            unsafe {
                let has_attr = msg_send![string, isKindOfClass: class!(NSAttributedString)];
                let characters = if has_attr { msg_send![string, string] } else { string };
                let string = nsstring_to_string(characters);
                cw.send_text_input(string, replacement_range.length != 0);
                let input_context: id = msg_send![this, inputContext];
                let () = msg_send![input_context, invalidateCharacterCoordinates];
                let () = msg_send![cw.view, setNeedsDisplay: YES];
                unmark_text(this, _sel);
            }
        }

        extern "C" fn key_down(this: &Object, _sel: Sel, event: id) {
            let _cw = get_cocoa_window(this);
            unsafe {
                let input_context: id = msg_send![this, inputContext];
                let () = msg_send![input_context, handleEvent: event];
            }
        }

        extern "C" fn key_up(_this: &Object, _sel: Sel, _event: id) {}

        extern "C" fn yes_function(_this: &Object, _se: Sel, _event: id) -> BOOL {
            YES
        }

        unsafe {
            decl.add_method(sel!(resetCursorRects), reset_cursor_rects as extern "C" fn(&Object, Sel));
            decl.add_method(sel!(hasMarkedText), has_marked_text as extern "C" fn(&Object, Sel) -> BOOL);
            decl.add_method(sel!(markedRange), marked_range as extern "C" fn(&Object, Sel) -> NSRange);
            decl.add_method(sel!(selectedRange), selected_range as extern "C" fn(&Object, Sel) -> NSRange);
            decl.add_method(
                sel!(setMarkedText: selectedRange: replacementRange:),
                set_marked_text as extern "C" fn(&mut Object, Sel, id, NSRange, NSRange),
            );
            decl.add_method(sel!(unmarkText), unmark_text as extern "C" fn(&Object, Sel));
            decl.add_method(
                sel!(attributedSubstringForProposedRange: actualRange:),
                attributed_substring_for_proposed_range as extern "C" fn(&Object, Sel, NSRange, *mut c_void) -> id,
            );
            decl.add_method(
                sel!(validAttributesForMarkedText),
                valid_attributes_for_marked_text as extern "C" fn(&Object, Sel) -> id,
            );
            decl.add_method(sel!(insertText: replacementRange:), insert_text as extern "C" fn(&Object, Sel, id, NSRange));
            decl.add_method(
                sel!(characterIndexForPoint:),
                character_index_for_point as extern "C" fn(&Object, Sel, NSPoint) -> u64,
            );
            decl.add_method(
                sel!(firstRectForCharacterRange: actualRange:),
                first_rect_for_character_range as extern "C" fn(&Object, Sel, NSRange, *mut c_void) -> NSRect,
            );
            decl.add_method(sel!(keyDown:), key_down as extern "C" fn(&Object, Sel, id));
            decl.add_method(sel!(keyUp:), key_up as extern "C" fn(&Object, Sel, id));
            decl.add_method(sel!(wantsKeyDownForEvent:), yes_function as extern "C" fn(&Object, Sel, id) -> BOOL);
            decl.add_method(sel!(acceptsFirstResponder:), yes_function as extern "C" fn(&Object, Sel, id) -> BOOL);
            decl.add_method(sel!(becomeFirstResponder:), yes_function as extern "C" fn(&Object, Sel, id) -> BOOL);
            decl.add_method(sel!(resignFirstResponder:), yes_function as extern "C" fn(&Object, Sel, id) -> BOOL);
        }

        decl.add_protocol(Protocol::get("NSTextInputClient").unwrap());
    }

    decl.register()
}

pub(crate) unsafe fn superclass(this: &Object) -> &Class {
    let superclass: id = msg_send![this, superclass];
    &*(superclass as *const _)
}

pub(crate) fn bottom_left_to_top_left(rect: NSRect) -> f64 {
    let height = unsafe { CGDisplayPixelsHigh(CGMainDisplayID()) };
    height as f64 - (rect.origin.y + rect.size.height)
}

#[cfg(not(feature = "cef"))]
fn load_mouse_cursor(cursor: MouseCursor) -> id {
    match cursor {
        MouseCursor::Arrow | MouseCursor::Default | MouseCursor::Hidden => load_native_cursor("arrowCursor"),
        MouseCursor::Hand => load_native_cursor("pointingHandCursor"),
        MouseCursor::Text => load_native_cursor("IBeamCursor"),
        // ` | MouseCursor::NoDrop`
        MouseCursor::NotAllowed => load_native_cursor("operationNotAllowedCursor"),
        MouseCursor::Crosshair => load_native_cursor("crosshairCursor"),
        /*
        MouseCursor::Grabbing | MouseCursor::Grab => load_native_cursor("closedHandCursor"),
        MouseCursor::VerticalText => load_native_cursor("IBeamCursorForVerticalLayout"),
        MouseCursor::Copy => load_native_cursor("dragCopyCursor"),
        MouseCursor::Alias => load_native_cursor("dragLinkCursor"),
        MouseCursor::ContextMenu => load_native_cursor("contextualMenuCursor"),
        */
        MouseCursor::EResize => load_native_cursor("resizeRightCursor"),
        MouseCursor::NResize => load_native_cursor("resizeUpCursor"),
        MouseCursor::WResize => load_native_cursor("resizeLeftCursor"),
        MouseCursor::SResize => load_native_cursor("resizeDownCursor"),
        MouseCursor::NeResize => load_undocumented_cursor("_windowResizeNorthEastCursor"),
        MouseCursor::NwResize => load_undocumented_cursor("_windowResizeNorthWestCursor"),
        MouseCursor::SeResize => load_undocumented_cursor("_windowResizeSouthEastCursor"),
        MouseCursor::SwResize => load_undocumented_cursor("_windowResizeSouthWestCursor"),

        MouseCursor::EwResize | MouseCursor::ColResize => load_native_cursor("resizeLeftRightCursor"),
        MouseCursor::NsResize | MouseCursor::RowResize => load_native_cursor("resizeUpDownCursor"),

        // Undocumented cursors: https://stackoverflow.com/a/46635398/5435443
        MouseCursor::Help => load_undocumented_cursor("_helpCursor"),
        //MouseCursor::ZoomIn => load_undocumented_cursor("_zoomInCursor"),
        //MouseCursor::ZoomOut => load_undocumented_cursor("_zoomOutCursor"),
        MouseCursor::NeswResize => load_undocumented_cursor("_windowResizeNorthEastSouthWestCursor"),
        MouseCursor::NwseResize => load_undocumented_cursor("_windowResizeNorthWestSouthEastCursor"),

        // While these are available, the former just loads a white arrow,
        // and the latter loads an ugly deflated beachball!
        // MouseCursor::Move => Cursor::Undocumented("_moveCursor"),
        // MouseCursor::Wait => Cursor::Undocumented("_waitCursor"),
        // An even more undocumented cursor...
        // https://bugs.eclipse.org/bugs/show_bug.cgi?id=522349
        // This is the wrong semantics for `Wait`, but it's the same as
        // what's used in Safari and Chrome.
        // ` | MouseCursor::Progress`
        MouseCursor::Wait => load_undocumented_cursor("busyButClickableCursor"),

        // For the rest, we can just snatch the cursors from WebKit...
        // They fit the style of the native cursors, and will seem
        // completely standard to macOS users.
        // https://stackoverflow.com/a/21786835/5435443
        // ` | MouseCursor::AllScroll`
        MouseCursor::Move => load_webkit_cursor("move"),
        // MouseCursor::Cell => load_webkit_cursor("cell"),
    }
}

#[cfg(feature = "cef")]
/// If Cef is enabled, we need to make sure it does not interfere with dragging
/// files to the main window. This function will recursively traverse the view
/// hierarchy disabling specific drag types.
pub(crate) fn disable_cef_dragged_types(root: crate::cx_apple::id) {
    unsafe {
        let registered_dragged_types: id = msg_send![root, registeredDraggedTypes];
        let registered_dragged_types_len: i32 = msg_send![registered_dragged_types, count];
        if registered_dragged_types_len > 0 {
            // TODO(hernan): I couldn't find a way to use `filteredUsingPredicate`, which is less verbose
            let filtered: id = msg_send![registered_dragged_types, mutableCopy];

            // This was a result of several try and error. We need to remove all of these
            // type in order to make sure the view will not interfere with file dragging.
            let types_to_remove = [NSFilenamesPboardType, NSURLPboardType];

            for i in 0..registered_dragged_types_len {
                let dragged_type: id = msg_send![registered_dragged_types, objectAtIndex: i];
                for t in types_to_remove {
                    let should_remove: BOOL = msg_send![dragged_type, isEqualToString: t];
                    if should_remove == YES {
                        let () = msg_send![filtered, removeObject: dragged_type];
                    }
                }
            }

            // `registerForDraggedTypes` always add new types (avoiding duplicates), so
            // we also need to unregistered existing dragged types before adding the
            // filtered list.
            let () = msg_send![root, unregisterDraggedTypes];
            let () = msg_send![root, registerForDraggedTypes: filtered];
        }

        let subviews: id = msg_send![root, subviews];
        let len = msg_send!(subviews, count);
        for i in 0..len {
            let view: id = msg_send![subviews, objectAtIndex: i];
            disable_cef_dragged_types(view);
        }
    }
}

fn make_timer(target: id, selector: Sel, time_to_wait: Duration, repeats: bool, user_info: id) -> id {
    unsafe {
        let pool: id = msg_send![class!(NSAutoreleasePool), new];
        let timer: id = msg_send![
            class!(NSTimer),
            timerWithTimeInterval: time_to_wait.as_secs_f64()
            target: target
            selector: selector
            userInfo: user_info
            repeats: repeats
        ];
        let ns_run_loop: id = msg_send![class!(NSRunLoop), mainRunLoop];
        let () = msg_send![ns_run_loop, addTimer: timer forMode: NSRunLoopCommonModes];
        let () = msg_send![pool, release];
        timer
    }
}
