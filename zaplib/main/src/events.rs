//! Events coming from user actions, system calls, and so on.

use crate::*;
use std::collections::{BTreeSet, HashMap};

/// Modifiers that were held when a key event was fired.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct KeyModifiers {
    pub shift: bool,
    pub control: bool,
    pub alt: bool,
    pub logo: bool,
}

/// The type of input that was used to trigger a pointer event.
#[derive(Clone, Debug, PartialEq)]
pub enum PointerInputType {
    Mouse,
    Touch,
    XR,
}

impl PointerInputType {
    pub fn is_touch(&self) -> bool {
        *self == PointerInputType::Touch
    }
    pub fn is_mouse(&self) -> bool {
        *self == PointerInputType::Mouse
    }
    pub fn is_xr(&self) -> bool {
        *self == PointerInputType::XR
    }
    pub fn has_hovers(&self) -> bool {
        *self == PointerInputType::Mouse || *self == PointerInputType::XR
    }
}

impl Default for PointerInputType {
    fn default() -> Self {
        Self::Mouse
    }
}

/// The type of input that was used to trigger a pointer event.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MouseButton {
    Left,
    Right,
    Other,
}

impl Default for MouseButton {
    fn default() -> Self {
        Self::Left
    }
}

/// See [`Event::PointerDown`].
#[derive(Clone, Default, Debug, PartialEq)]
pub struct PointerDownEvent {
    pub window_id: usize,
    pub abs: Vec2,
    pub rel: Vec2,
    pub rect: Rect,
    // Digit based system is supposed to track pointer interactions, where each pointer is a digit.
    // Its doesn't exactly work in the way its supposed to work in terms of pointer tracking.
    // Our needs currently require us to have sure shot Mouse interaction events.
    // Hence we are adding `button: MouseButton` in addition to existing digits.
    // TODO(Shobhit): Refresh the digit based pointer tracking system someday.
    pub digit: usize,
    pub button: MouseButton,
    pub tap_count: u32,
    pub(crate) handled: bool,
    pub input_type: PointerInputType,
    pub modifiers: KeyModifiers,
    pub time: f64,
}

/// See [`Event::PointerMove`].
#[derive(Clone, Default, Debug, PartialEq)]
pub struct PointerMoveEvent {
    pub window_id: usize,
    pub abs: Vec2,
    pub abs_start: Vec2,
    pub rel: Vec2,
    pub rel_start: Vec2,
    pub rect: Rect,
    pub is_over: bool,
    pub digit: usize,
    pub input_type: PointerInputType,
    pub modifiers: KeyModifiers,
    pub time: f64,
}

impl PointerMoveEvent {
    pub fn move_distance(&self) -> f32 {
        ((self.abs_start.x - self.abs.x).powf(2.) + (self.abs_start.y - self.abs.y).powf(2.)).sqrt()
    }
}

/// See [`Event::PointerUp`].
#[derive(Clone, Default, Debug, PartialEq)]
pub struct PointerUpEvent {
    pub window_id: usize,
    pub abs: Vec2,
    pub abs_start: Vec2,
    pub rel: Vec2,
    pub rel_start: Vec2,
    pub rect: Rect,
    pub digit: usize,
    pub button: MouseButton,
    pub is_over: bool,
    pub input_type: PointerInputType,
    pub modifiers: KeyModifiers,
    pub time: f64,
}

/// The type of [`PointerHoverEvent`].
#[derive(Clone, Debug, PartialEq)]
pub enum HoverState {
    In,
    Over,
    Out,
}

impl Default for HoverState {
    fn default() -> HoverState {
        HoverState::Over
    }
}

/// See [`Event::PointerHover`].
#[derive(Clone, Default, Debug, PartialEq)]
pub struct PointerHoverEvent {
    pub window_id: usize,
    pub digit: usize,
    pub abs: Vec2,
    pub rel: Vec2,
    pub rect: Rect,
    pub any_down: bool,
    pub(crate) handled: bool,
    pub hover_state: HoverState,
    pub modifiers: KeyModifiers,
    pub time: f64,
}

/// See [`Event::PointerScroll`].
#[derive(Clone, Default, Debug, PartialEq)]
pub struct PointerScrollEvent {
    pub window_id: usize,
    pub digit: usize,
    pub abs: Vec2,
    pub rel: Vec2,
    pub rect: Rect,
    pub scroll: Vec2,
    pub input_type: PointerInputType,
    //pub is_wheel: bool,
    pub handled_x: bool,
    pub handled_y: bool,
    pub modifiers: KeyModifiers,
    pub time: f64,
}

/// See [`Event::WindowGeomChange`].
#[derive(Clone, Default, Debug, PartialEq)]
pub struct WindowGeomChangeEvent {
    pub window_id: usize,
    pub old_geom: WindowGeom,
    pub new_geom: WindowGeom,
}

/// See [`Event::Timer`].
#[derive(Clone, Debug, PartialEq)]
pub struct TimerEvent {
    pub timer_id: u64,
}

/// See [`Event::Signal`].
/// TODO(JP): Is this a bit too complicated of an API? What about if we just
/// send `pub signal: u64`, or even a `Box`? Then you can use it for anything.
#[derive(Clone, Debug, PartialEq)]
pub struct SignalEvent {
    pub signals: HashMap<Signal, BTreeSet<StatusId>>,
}

/// Data for various kinds of key-based events ([`Event::KeyDown`], [`Event::KeyUp`], etc).
#[derive(Clone, Debug, PartialEq)]
pub struct KeyEvent {
    pub key_code: KeyCode,
    //pub key_char: char,
    pub is_repeat: bool,
    pub modifiers: KeyModifiers,
    pub time: f64,
}

/// See [`Event::KeyFocus`] and [`Event::KeyFocusLost`].
#[derive(Clone, Debug, PartialEq)]
pub struct KeyFocusEvent {
    pub(crate) prev: Option<ComponentId>,
    pub(crate) focus: Option<ComponentId>,
}

/// See [`Event::FileDragUpdate`].
#[derive(Clone, Debug, PartialEq)]
pub struct FileDragUpdateEvent {
    pub abs: Vec2,
}

/// See [`Event::TextInput`].
#[derive(Clone, Debug, PartialEq)]
pub struct TextInputEvent {
    pub input: String,
    pub replace_last: bool,
    pub was_paste: bool,
}

/// See [`Event::WindowCloseRequested`].
#[derive(Clone, Debug, PartialEq)]
pub struct WindowCloseRequestedEvent {
    pub window_id: usize,
    pub accept_close: bool,
}

/// See [`Event::WindowClosed`].
#[derive(Clone, Debug, PartialEq)]
pub struct WindowClosedEvent {
    pub window_id: usize,
}

/// See [`Event::WindowResizeLoop`].
#[derive(Clone, Debug, PartialEq)]
pub struct WindowResizeLoopEvent {
    pub was_started: bool,
    pub window_id: usize,
}

/// Response to operating system inquiry if a [`Window`] can be dragged.
#[derive(Clone, Debug, PartialEq)]
pub enum WindowDragQueryResponse {
    NoAnswer,
    Client,
    Caption,
    SysMenu, // windows only
}

/// See [`Event::WindowDragQuery`].
/// To respond, set `response` field to a variant of [`WindowDragQueryResponse`].
#[derive(Clone, Debug, PartialEq)]
pub struct WindowDragQueryEvent {
    pub window_id: usize,
    pub abs: Vec2,
    pub response: WindowDragQueryResponse,
}

/// See [`Event::WebSocketMessage`].
#[derive(Clone, Debug, PartialEq)]
pub struct WebSocketMessageEvent {
    pub url: String,
    pub result: Result<Vec<u8>, String>,
}

/// A file that was supplied by a user, as opposed to by the application itself (like font resources
/// and such).
#[derive(Clone, Debug)]
pub struct UserFile {
    /// Per UNIX convention, basename is the filename (including extension) part.
    /// This is the only part of the filename that is exposed on all platforms (Wasm hides the
    /// full path).
    pub basename: String,
    /// The actual file handle.
    pub file: UniversalFile,
}
impl PartialEq for UserFile {
    fn eq(&self, other: &Self) -> bool {
        self.basename == other.basename
    }
}

/// Fires when a web worker calls `callRust` to trigger a function in Rust.
#[derive(Clone, Debug)]
pub struct WebRustCallEvent {
    /// Description of the event.
    pub name: String,
    pub params: Vec<ZapParam>,
    pub callback_id: u32,
}

/// See [`Event::AppOpenFiles`].
#[derive(Clone, Debug, PartialEq)]
pub struct AppOpenFilesEvent {
    pub user_files: Vec<UserFile>,
}

/// See [`Event::System`].
#[derive(Debug, Clone)]
pub enum SystemEvent {
    /// See [`WebRustCallEvent`]. This event must have a handler registered through [`Cx::on_call_rust`].
    WebRustCall(Option<WebRustCallEvent>),
    Draw,
    /// We're going to repaint our draw tree.
    Paint,
    /// The system wants us to set a different mouse cursor.
    WindowSetHoverCursor(MouseCursor),
    /// Calls `do_message_loop_work` for CEF.
    #[cfg(feature = "cef")]
    CefDoMessageLoopWork,
}

/// Global event passed into an application's `handle` function. See contained structs for more details.
#[derive(Clone, Debug)]
pub enum Event {
    /// No event, to avoid unwrapping `Option<Event>` all over the place.
    None,
    /// App starts up; should be the very first event that gets fired.
    Construct,
    /// App gained focus.
    ///
    /// TODO(JP): Rename to `AppFocusGained` to be more symmetric with [`Event::AppFocusLost`]?
    AppFocus,
    /// App lost focus.
    AppFocusLost,
    /// New frame requested. Useful for animations; you can request this using [`Cx::request_next_frame`].
    NextFrame,
    /// The operating system inquired if a [`Window`] can be dragged.
    WindowDragQuery(WindowDragQueryEvent),
    /// The user requested to close the [`Window`].
    WindowCloseRequested(WindowCloseRequestedEvent),
    /// The [`Window`] actually closed.
    WindowClosed(WindowClosedEvent),
    /// Geometry of a [`Window`] changed (position, size, etc).
    WindowGeomChange(WindowGeomChangeEvent),
    /// The user started or ended resizing the [`Window`].
    ///
    /// TODO(JP): Mostly for internal use in Windows; we might not want to expose this
    /// to end users?
    WindowResizeLoop(WindowResizeLoopEvent),
    /// A pointer (mouse, touch, etc) was pressed down.
    ///
    /// Someone has to call [`Cx::set_key_focus`] or [`Cx::keep_key_focus`] when handling, otherwise
    /// the key focus will be reset.
    PointerDown(PointerDownEvent),
    /// A pointer (mouse, touch, etc) was moved.
    PointerMove(PointerMoveEvent),
    /// The mouse was hovered over the screen. This cannot fire from touch input.
    PointerHover(PointerHoverEvent),
    /// A pointer (mouse, touch, etc) was released.
    PointerUp(PointerUpEvent),
    /// A pointer (mouse, touch, etc) triggered a scroll.
    PointerScroll(PointerScrollEvent),
    /// A [`Timer`] was requested using [`Cx::start_timer`].
    Timer(TimerEvent),
    /// A signal was fired using [`Cx::send_signal`].
    Signal(SignalEvent),
    Command(CommandId),
    /// Keyboard focus changed between components.
    KeyFocus(KeyFocusEvent),
    /// Keyboard focus was lost by a component. Returned from [`Event::hits_keyboard`].
    KeyFocusLost(KeyFocusEvent),
    /// A key was pressed down. Rely on [`Event::TextInput`] for text input instead of this.
    KeyDown(KeyEvent),
    /// A key was released. Rely on [`Event::TextInput`] for text input instead of this.
    KeyUp(KeyEvent),
    /// Some text was inputted. Rely on this for text input instead of [`KeyEvent`].
    TextInput(TextInputEvent),
    /// Text was requested to be copied to the clipboard.
    TextCopy,
    /// A websocket message was received.
    WebSocketMessage(WebSocketMessageEvent),
    /// Intended for platforms that can register a file type to an application.
    /// Fires:
    /// - when application starts with a file
    /// - when `Window::create_add_drop_target_for_app_open_files` is set and a file is dragged and released onto the window
    /// - when the application is already started and an associated file is double-clicked
    AppOpenFiles(AppOpenFilesEvent),
    /// When `Window::create_add_drop_target_for_app_open_files` is set and a file is dragged (without being released)
    /// onto the window
    FileDragBegin,
    /// When a file is being dragged and the mouse position changes
    FileDragUpdate(FileDragUpdateEvent),
    /// When a file is being dragged and the mouse moves out of the window
    FileDragCancel,
    /// Events that are handled internally and are not propagated to an application `handle` method.
    System(SystemEvent),
}

impl Default for Event {
    fn default() -> Event {
        Event::None
    }
}

impl Event {
    /// Checks if an [`Event`] is a pointer-event with coordinates falling inside
    /// [`Rect`], or already has an associated [`ComponentId`] that matches the given one.
    ///
    /// For unhandled [`Event::PointerDown`] and [`Event::PointerHover`] events, the given
    /// [`ComponentId`] will be associated with that pointer (if the event falls in [`Rect`]).
    ///
    /// For [`Event::PointerUp`] (and [`Event::PointerHover`] with [`HoverState::Out`]) it's
    /// the other way around: if the pointer is associated with the given [`ComponentId`], it
    /// will be returned regardless of [`Rect`].
    ///
    /// We pass in [`Option<Rect>`] instead of [`Rect`] for convenience, since it often comes
    /// from [`Area::get_rect_for_first_instance`], which returns [`Option<Rect>`]. When passing
    /// in [`None`], we always return [`Event::None`].
    #[must_use]
    pub fn hits_pointer(&mut self, cx: &mut Cx, component_id: ComponentId, rect: Option<Rect>) -> Event {
        if let Some(rect) = rect {
            match self {
                Event::PointerScroll(pe) => {
                    if rect.contains(pe.abs) {
                        //pe.handled = true;
                        return Event::PointerScroll(PointerScrollEvent { rel: pe.abs - rect.pos, rect, ..pe.clone() });
                    }
                }
                Event::PointerHover(pe) => {
                    if cx.pointers[pe.digit]._over_last == Some(component_id) {
                        let mut any_down = false;
                        for pointer in &cx.pointers {
                            if pointer.captured == Some(component_id) {
                                any_down = true;
                                break;
                            }
                        }
                        if !pe.handled && rect.contains(pe.abs) {
                            pe.handled = true;
                            if let HoverState::Out = pe.hover_state {
                                //    cx.pointer_over_last_area = Area::Empty;
                            } else {
                                cx.pointers[pe.digit].over_last = Some(component_id);
                            }
                            return Event::PointerHover(PointerHoverEvent {
                                rel: pe.abs - rect.pos,
                                rect,
                                any_down,
                                ..pe.clone()
                            });
                        } else {
                            //self.was_over_last_call = false;
                            return Event::PointerHover(PointerHoverEvent {
                                rel: pe.abs - rect.pos,
                                rect,
                                any_down,
                                hover_state: HoverState::Out,
                                ..pe.clone()
                            });
                        }
                    } else if !pe.handled && rect.contains(pe.abs) {
                        let mut any_down = false;
                        for pointer in &cx.pointers {
                            if pointer.captured == Some(component_id) {
                                any_down = true;
                                break;
                            }
                        }
                        cx.pointers[pe.digit].over_last = Some(component_id);
                        pe.handled = true;
                        //self.was_over_last_call = true;
                        return Event::PointerHover(PointerHoverEvent {
                            rel: pe.abs - rect.pos,
                            rect,
                            any_down,
                            hover_state: HoverState::In,
                            ..pe.clone()
                        });
                    }
                }
                Event::PointerMove(pe) => {
                    // check wether our digit is captured, otherwise don't send
                    if cx.pointers[pe.digit].captured == Some(component_id) {
                        let abs_start = cx.pointers[pe.digit].down_abs_start;
                        let rel_start = cx.pointers[pe.digit].down_rel_start;
                        return Event::PointerMove(PointerMoveEvent {
                            abs_start,
                            rel: pe.abs - rect.pos,
                            rel_start,
                            rect,
                            is_over: rect.contains(pe.abs),
                            ..pe.clone()
                        });
                    }
                }
                Event::PointerDown(pe) => {
                    if !pe.handled && rect.contains(pe.abs) {
                        // Scan if any of the pointers already captured this area.
                        // TODO(JP): We might want to skip this in cases where we want to support multi-touch.
                        for pointer in &cx.pointers {
                            if pointer.captured == Some(component_id) {
                                return Event::None;
                            }
                        }
                        cx.pointers[pe.digit].captured = Some(component_id);
                        let rel = pe.abs - rect.pos;
                        cx.pointers[pe.digit].down_abs_start = pe.abs;
                        cx.pointers[pe.digit].down_rel_start = rel;
                        pe.handled = true;
                        return Event::PointerDown(PointerDownEvent { rel, rect, ..pe.clone() });
                    }
                }
                Event::PointerUp(pe) => {
                    if cx.pointers[pe.digit].captured == Some(component_id) {
                        cx.pointers[pe.digit].captured = None;
                        let abs_start = cx.pointers[pe.digit].down_abs_start;
                        let rel_start = cx.pointers[pe.digit].down_rel_start;
                        return Event::PointerUp(PointerUpEvent {
                            is_over: rect.contains(pe.abs),
                            abs_start,
                            rel_start,
                            rel: pe.abs - rect.pos,
                            rect,
                            ..pe.clone()
                        });
                    }
                }
                _ => (),
            };
        }
        Event::None
    }

    /// Process a keyboard/text-related event, if the given [`ComponentId`] has key focus ([`Cx::key_focus`]).
    #[must_use]
    pub fn hits_keyboard(&mut self, cx: &mut Cx, component_id: ComponentId) -> Event {
        match self {
            Event::KeyFocus(kf) => {
                if kf.prev == Some(component_id) {
                    return Event::KeyFocusLost(kf.clone());
                } else if kf.focus == Some(component_id) {
                    return Event::KeyFocus(kf.clone());
                }
            }
            Event::KeyDown(_) => {
                if cx.key_focus == Some(component_id) {
                    return self.clone();
                }
            }
            Event::KeyUp(_) => {
                if cx.key_focus == Some(component_id) {
                    return self.clone();
                }
            }
            Event::TextInput(_) => {
                if cx.key_focus == Some(component_id) {
                    return self.clone();
                }
            }
            Event::TextCopy => {
                if cx.key_focus == Some(component_id) {
                    return Event::TextCopy;
                }
            }
            _ => (),
        }
        Event::None
    }
}

/// For firing and capturing custom events. Can even be fired from different
/// threads using [`Cx::post_signal`].
#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug, Default)]
pub struct Signal {
    pub signal_id: usize,
}

/// Status field to send with a [`Signal`].
///
/// An alias over LocationHash so we have a semantic type
/// but can change the underlying implementation whenever.
pub type StatusId = LocationHash;

/// Created using [`Cx::start_timer`].
#[derive(Clone, Debug, Default)]
pub struct Timer {
    pub timer_id: u64,
}

impl Timer {
    pub fn empty() -> Timer {
        Timer { timer_id: 0 }
    }

    pub fn is_timer(&mut self, te: &TimerEvent) -> bool {
        te.timer_id == self.timer_id
    }
}

/// Lowest common denominator keymap between desktop and web.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum KeyCode {
    Escape,

    Backtick,
    Key0,
    Key1,
    Key2,
    Key3,
    Key4,
    Key5,
    Key6,
    Key7,
    Key8,
    Key9,
    Minus,
    Equals,

    Backspace,
    Tab,

    KeyQ,
    KeyW,
    KeyE,
    KeyR,
    KeyT,
    KeyY,
    KeyU,
    KeyI,
    KeyO,
    KeyP,
    LBracket,
    RBracket,
    Return,

    KeyA,
    KeyS,
    KeyD,
    KeyF,
    KeyG,
    KeyH,
    KeyJ,
    KeyK,
    KeyL,
    Semicolon,
    Quote,
    Backslash,

    KeyZ,
    KeyX,
    KeyC,
    KeyV,
    KeyB,
    KeyN,
    KeyM,
    Comma,
    Period,
    Slash,

    Control,
    Alt,
    Shift,
    Logo,

    //RightControl,
    //RightShift,
    //RightAlt,
    //RightLogo,
    Space,
    Capslock,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,

    PrintScreen,
    Scrolllock,
    Pause,

    Insert,
    Delete,
    Home,
    End,
    PageUp,
    PageDown,

    Numpad0,
    Numpad1,
    Numpad2,
    Numpad3,
    Numpad4,
    Numpad5,
    Numpad6,
    Numpad7,
    Numpad8,
    Numpad9,

    NumpadEquals,
    NumpadSubtract,
    NumpadAdd,
    NumpadDecimal,
    NumpadMultiply,
    NumpadDivide,
    Numlock,
    NumpadEnter,

    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,

    Unknown,
}

impl Default for KeyCode {
    fn default() -> Self {
        KeyCode::Unknown
    }
}
