//! The main module of this crate. Contains [`Cx`], which you will see
//! used all over the place.

use crate::*;

#[cfg(feature = "cef")]
use cef_browser::MaybeCefBrowser;
use debug_log::DebugLog;
use std::{
    any::{Any, TypeId},
    collections::{BTreeSet, HashMap},
    fmt::Write,
    sync::{Arc, RwLock},
};
use zaplib_shader_compiler::generate_shader_ast::*;

/// Contains information about the platform (operating system) that we're running on.
#[derive(Clone)]
pub enum PlatformType {
    Unknown,
    Windows,
    OSX,
    Linux { custom_window_chrome: bool },
    Web { protocol: String, hostname: String, port: u16, pathname: String, search: String, hash: String },
}

impl PlatformType {
    pub fn is_desktop(&self) -> bool {
        match self {
            PlatformType::Unknown => true,
            PlatformType::Windows => true,
            PlatformType::OSX => true,
            PlatformType::Linux { .. } => true,
            PlatformType::Web { .. } => false,
        }
    }
}

pub type CallRustSyncFn = fn(name: String, params: Vec<ZapParam>) -> Vec<ZapParam>;

/// The main "context" object which contains pretty much everything we need within the framework.
pub struct Cx {
    /// See [`PlatformType`].
    pub platform_type: PlatformType,

    /// List of actual [`CxWindow`] objects. [`Window::window_id`] represents an index in this list.
    pub(crate) windows: Vec<CxWindow>,
    /// Indices in [`Cx::windows`] that have been closed and can be reused.
    ///
    /// TODO(JP): Should we be more explicit and use [`Option`] in [`Cx::windows`]?
    pub(crate) windows_free: Vec<usize>,

    /// List of actual [`CxPass`] objects. [`Pass::pass_id`] represents an index in this list.
    ///
    /// TODO(JP): We currently never remove old [`CxPass`]es.
    pub(crate) passes: Vec<CxPass>,

    /// The [`CxView`] objects that make up the draw tree. [`View::view_id`] represents an index in this list.
    ///
    /// TODO(JP): The first element is a dummy element, since we use `view_id == 0`
    /// as a sort of null pointer, which is pretty gross and can be confusing. Might
    /// be better to use `Option` wherever we need that instead..
    ///
    /// TODO(JP): We currently never remove old [`CxView`]s.
    pub(crate) views: Vec<CxView>,

    /// The compiled [`CxShader`]s. [`Shader::shader_id`] and [`Shader::shader_id`] represent an index in this list.
    pub(crate) shaders: Vec<CxShader>,
    /// Shader IDs (indices in [`Cx::shaders`]) that need to be recompiled.
    pub(crate) shader_recompile_ids: Vec<usize>,
    /// List of actual [`CxTexture`] objects. [`TextureHandle::texture_id`] represents an index in this list.
    pub(crate) textures: Vec<CxTexture>,
    /// List of actual [`CxGpuGeometry`] objects. [`GpuGeometry::gpu_geometry_id`] represents an index in this list.
    pub(crate) gpu_geometries: Vec<CxGpuGeometry>,

    /// Whether we are currently (re)drawing, ie. we called the app's `draw` function.
    pub(crate) in_redraw_cycle: bool,
    /// An auto-incrementing ID representing the current (re)draw cycle.
    ///
    /// TODO(JP): This value probably shouldn't be used when [`Cx::in_redraw_cycle`] is false, but
    /// currently it sticks around. We could combine the two variables in an [`Option<u64>`]?
    pub(crate) redraw_id: u64,
    /// Stack of [`Window::window_id`]s / indices into [`Cx::windows`], using [`Window::begin_window`]
    /// and [`Window::end_window`].
    pub(crate) window_stack: Vec<usize>,
    /// Stack of [`Pass::pass_id`]s / indices into [`Cx::passes`], using [`Pass::begin_pass`]
    /// and [`Pass::end_pass`].
    pub(crate) pass_stack: Vec<usize>,
    /// Stack of [`View::view_id`]s / indices into [`Cx::views`], using [`View::begin_view`]
    /// and [`View::end_view`].
    pub(crate) view_stack: Vec<usize>,
    /// A stack of [`CxLayoutBox`]s, using [`Cx::begin_typed_box`] and [`Cx::end_typed_box`]
    pub(crate) layout_boxes: Vec<CxLayoutBox>,

    /// The instance offsets for the different [`Shader`]s when the current "shader group" was started.
    ///
    /// Empty when there is no current "shader group". See [`Cx::begin_shader_group`].
    pub(crate) shader_group_instance_offsets: Vec<usize>,

    /// A list of [`Area`]s that we want to align later on. This is kept separate
    /// from [`Cx::layout_boxes`] (even though this is part of the box layout
    /// system) so that a parent [`CxLayoutBox`] can align a bunch of stuff that was
    /// drawn using child [`CxLayoutBox`]s.
    ///
    /// TODO(JP): This may currently only contain [`Area::InstanceRange`], so
    /// maybe we should change the type of this? It might also be nice to be able
    /// explicitly push [`Area::View`] on this list though, and then set a uniform
    /// on the entire [`CxView`], for better performance?
    pub(crate) layout_box_align_list: Vec<Area>,

    /// The system-default `dpi_factor`. See also [`PassUniforms::dpi_factor`].
    ///
    /// More commonly known as the "device pixel ratio". TODO(JP): Rename?
    pub(crate) default_dpi_factor: f32,

    /// The `dpi_factor` used during the current [`Pass`] (since you can override the system default).
    /// See also [`PassUniforms::dpi_factor`].
    ///
    /// More commonly known as the "device pixel ratio". TODO(JP): Rename?
    pub current_dpi_factor: f32,

    /// Last timestamp from when an event was fired, in seconds since the application
    /// was started. Typically you want to use this instead of making a system call.
    pub last_event_time: f64,

    /// The last [`Timer::timer_id`] that was issued.
    pub(crate) last_timer_id: u64,
    /// The last [`Signal::signal_id`] that was issued.
    pub(crate) last_signal_id: usize,

    /// The current [`ComponentId`] that has keyboard focus, so it can register key input [`Event`]s.
    ///
    /// See also [`Cx::prev_key_focus`] and [`Cx::next_key_focus`].
    pub(crate) key_focus: Option<ComponentId>,
    /// The [`ComponentId`] that previously was [`Cx::key_focus`], so you can revert it using
    /// [`Cx::revert_key_focus`].
    ///
    /// See also [`Cx::key_focus`] and [`Cx::next_key_focus`].
    prev_key_focus: Option<ComponentId>,
    /// The [`ComponentId`] that will become [`Cx::key_focus`] when the current events are handled.
    /// Gets set using [`Cx::set_key_focus`] or [`Cx::revert_key_focus`].
    ///
    /// See also [`Cx::prev_key_focus`] and [`Cx::next_key_focus`].
    ///
    /// TODO(JP): It's possible to set this during the draw cycle instead of during an
    /// event handler, and then it won't update [`Cx::key_focus`] until the next event
    /// is handled. We should probably guard against that.
    next_key_focus: Option<Option<ComponentId>>,
    pub(crate) keys_down: Vec<KeyEvent>,

    /// The cursor type that the user sees while holding the mouse down. Gets reset to [`None`] when
    /// you release the mouse button ([`Event::PointerUp`]).
    pub(crate) down_mouse_cursor: Option<MouseCursor>,

    /// The cursor type that the user sees while hovering (not holding the mouse down).
    /// Gets reset when there's a new [`Event::PointerHover`], so you have to periodically set this.
    pub(crate) hover_mouse_cursor: Option<MouseCursor>,

    /// The current state of each "pointer" that we track.
    ///
    /// TODO(JP): This seems mostly relevant for multi-touch, which we don't really support very
    /// well yet. Should we keep this?
    pub(crate) pointers: Vec<CxPerPointer>,

    /// Whether [`Cx::request_next_frame`] was called.
    pub(crate) requested_next_frame: bool,

    /// Whether [`Cx::request_draw`] was called.
    pub(crate) requested_draw: bool,

    /// The local "signals", which are like custom events that also work across threads.
    ///
    /// See also [`Signal`] and [`SignalEvent`].
    pub(crate) signals: HashMap<Signal, BTreeSet<StatusId>>,

    /// A map from profile IDs to [`UniversalInstant`], for keeping track of how long things
    /// take.
    pub(crate) profiles: HashMap<u64, UniversalInstant>,

    /// For compiling [`Shader`]s.
    pub(crate) shader_ast_generator: ShaderAstGenerator,

    /// Settings per command; see [`CommandId`] and [`CxCommandSetting`].
    pub(crate) command_settings: HashMap<CommandId, CxCommandSetting>,

    /// When set to true, will trigger a panic on the next redraw. Can be useful
    /// for debugging unwanted redraws. Can be set to true by pressing the "print
    /// screen" button on the keyboard.
    pub(crate) panic_redraw: bool,

    /// Platform-specific fields.
    pub(crate) platform: CxPlatform,

    /// The user's event handler. Storing it like this cuts the compile time of an end-user application in half.
    pub(crate) event_handler: Option<*mut dyn FnMut(&mut Cx, &mut Event)>,

    /// Fonts specific data
    /// It might be possible for fonts data to be shared between different threads, so
    /// we need to make use of locks.
    pub fonts_data: Arc<RwLock<CxFontsData>>,

    /// A buffer with temporary data used in [`Area::get_first`] and [`Area::get_first_mut`].
    ///
    /// Shouldn't be used excessively. Gets cleared out after a draw cycle.
    ///
    /// TODO(JP): It would be nice if we can eliminate this altogether, e.g. by guaranteeing
    /// that handle functions are only called after a component has been drawn, or by making
    /// shader animations a more integral part of the framework.
    pub(crate) temp_default_data: Vec<Box<dyn Any>>,

    /// See [`CxDebugFlags`].
    pub(crate) debug_flags: CxDebugFlags,

    #[cfg(feature = "cef")]
    pub(crate) cef_browser: MaybeCefBrowser,

    /// Various debug logs that are getting appended during draw cycle.
    /// See [`DebugLog`] for more information on supported types
    pub(crate) debug_logs: Vec<DebugLog>,

    /// Function registered through [`Cx::on_call_rust_async`]
    pub call_rust_async_fn: Option<usize>,

    /// Reference to the main_app type
    pub app_type_id: TypeId,

    /// `false` when we're in the `new` function of the app. This means that we can do thread-unsafe
    /// initialization, since we're still guaranteed that there are no other threads running.
    pub(crate) finished_app_new: bool,
}

/// Flags that can be set that enable debug functionality. See [`Cx::debug_flags_mut`] for an example.
#[derive(Copy, Clone, Default)]
pub struct CxDebugFlags {
    /// See [`CxDebugDrawTree`].
    pub draw_tree: CxDebugDrawTree,

    /// Makes it so every call to `Cx::add_instances` gets a fresh [`DrawCall`]. This is useful for debugging,
    /// since the batching of draw calls can be confusing sometimes (and you should never rely on it happening).
    pub disable_draw_call_batching: bool,

    /// Enables overlay with borders of CxLayoutBox rects
    pub enable_layout_debugger: bool,
}

/// What kind of debug information should be printed about the draw tree.
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum CxDebugDrawTree {
    /// No draw tree debugging.
    None,
    /// Print the draw tree.
    DrawTree,
    /// Print the draw tree and also information on the individual instances.
    Instances,
    /// Print geometries.
    Geometries,
}
impl Default for CxDebugDrawTree {
    fn default() -> Self {
        Self::None
    }
}

/// Settings for "commands"; see [`CommandId`].
///
/// Only supported on OSX for now.
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
#[derive(Clone, Copy, Default)]
pub(crate) struct CxCommandSetting {
    pub(crate) shift: bool,
    pub(crate) key_code: KeyCode,
    pub(crate) enabled: bool,
}

#[derive(Default, Clone)]
pub(crate) struct CxPerPointer {
    pub(crate) captured: Option<ComponentId>,
    pub(crate) tap_count: (Vec2, f64, u32),
    pub(crate) down_abs_start: Vec2,
    pub(crate) down_rel_start: Vec2,
    pub(crate) over_last: Option<ComponentId>,
    pub(crate) _over_last: Option<ComponentId>,
}

pub(crate) const NUM_POINTERS: usize = 10;

impl Cx {
    pub fn new(app_type_id: TypeId) -> Self {
        let mut pointers = Vec::new();
        pointers.resize(NUM_POINTERS, CxPerPointer::default());

        let textures = vec![CxTexture {
            desc: TextureDesc { format: TextureFormat::ImageRGBA, width: Some(4), height: Some(4), multisample: None },
            image_u32: vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            update_image: true,
            platform: CxPlatformTexture::default(),
        }];

        // We try to initialize Vecs with some reasonable capacity, to prevent reallocations.
        Self {
            platform_type: PlatformType::Unknown,

            windows: Vec::new(),
            windows_free: Vec::new(),
            passes: Vec::with_capacity(10),
            // TODO(JP): See my note up at [`Cx::views`].
            views: vec![CxView::default()],
            fonts_data: Arc::new(RwLock::new(CxFontsData::default())),
            textures,
            shaders: Vec::with_capacity(50),
            shader_recompile_ids: Vec::with_capacity(50),
            gpu_geometries: Vec::new(),

            default_dpi_factor: 1.0,
            current_dpi_factor: 1.0,
            in_redraw_cycle: false,
            window_stack: Vec::new(),
            pass_stack: Vec::with_capacity(10),
            view_stack: Vec::with_capacity(50),
            layout_boxes: Vec::with_capacity(100),
            layout_box_align_list: Vec::with_capacity(100),
            shader_group_instance_offsets: Vec::with_capacity(10),

            last_event_time: 0.0,

            redraw_id: 1,
            last_timer_id: 1,
            last_signal_id: 1,

            next_key_focus: None,
            prev_key_focus: None,
            key_focus: None,
            keys_down: Vec::new(),

            down_mouse_cursor: None,
            hover_mouse_cursor: None,
            pointers,

            shader_ast_generator: ShaderAstGenerator::new(),

            command_settings: HashMap::new(),

            requested_next_frame: false,
            requested_draw: false,

            profiles: HashMap::new(),

            signals: HashMap::new(),

            panic_redraw: false,

            platform: CxPlatform::default(),

            event_handler: None,

            temp_default_data: Vec::with_capacity(1000),

            debug_flags: Default::default(),

            #[cfg(feature = "cef")]
            cef_browser: MaybeCefBrowser::new(),

            debug_logs: Vec::new(),

            call_rust_async_fn: None,
            app_type_id,
            finished_app_new: false,
        }
    }

    pub(crate) fn process_pre_event(&mut self, event: &mut Event) {
        match event {
            Event::PointerHover(pe) => {
                self.pointers[pe.digit].over_last = None;
                self.hover_mouse_cursor = None;
            }
            Event::PointerUp(_pe) => {
                self.down_mouse_cursor = None;
            }
            Event::PointerDown(pe) => {
                // lets set the pointer tap count
                pe.tap_count = self.process_tap_count(pe.digit, pe.abs, pe.time);
            }
            Event::KeyDown(ke) => {
                self.process_key_down(ke.clone());
                if ke.key_code == KeyCode::PrintScreen {
                    self.panic_redraw = true;
                }

                // shortcuts: ctrl + option + cmd + ...
                if ke.modifiers.control && ke.modifiers.alt && ke.modifiers.logo {
                    match ke.key_code {
                        KeyCode::Key1 => {
                            self.debug_flags.enable_layout_debugger = !self.debug_flags.enable_layout_debugger;
                            log!("Set enable_layout_debugger to {}", self.debug_flags.enable_layout_debugger);
                            self.request_draw();
                        }
                        KeyCode::Key2 => {
                            self.debug_flags.disable_draw_call_batching = !self.debug_flags.disable_draw_call_batching;
                            log!("Set disable_draw_call_batching to {}", self.debug_flags.disable_draw_call_batching);
                            self.request_draw();
                        }
                        KeyCode::Key3 => {
                            // cycle through options:
                            match self.debug_flags.draw_tree {
                                CxDebugDrawTree::None => self.debug_flags.draw_tree = CxDebugDrawTree::DrawTree,
                                CxDebugDrawTree::DrawTree => self.debug_flags.draw_tree = CxDebugDrawTree::Instances,
                                CxDebugDrawTree::Instances => self.debug_flags.draw_tree = CxDebugDrawTree::Geometries,
                                CxDebugDrawTree::Geometries => self.debug_flags.draw_tree = CxDebugDrawTree::None,
                            }
                            log!("Set draw_tree to {:?}", self.debug_flags.draw_tree);
                            self.request_draw();
                        }
                        _ => {}
                    }
                }
            }
            Event::KeyUp(ke) => {
                self.process_key_up(ke);
            }
            Event::AppFocusLost => {
                self.call_all_keys_up();
            }
            _ => (),
        };
    }

    pub(crate) fn process_post_event(&mut self, event: &mut Event) {
        match event {
            Event::PointerUp(pe) => {
                // decapture automatically
                self.pointers[pe.digit].captured = None;
            }
            Event::PointerHover(pe) => {
                // new last area pointer over
                self.pointers[pe.digit]._over_last = self.pointers[pe.digit].over_last;
                //if pe.hover_state == HoverState::Out{
                //    self.hover_mouse_cursor = None;
                //}
            }
            Event::PointerScroll(_) => {
                // check for anything being paint or draw dirty
                #[cfg(not(target_arch = "wasm32"))]
                if self.requested_draw {
                    self.platform.desktop.repaint_via_scroll_event = true;
                }
            }
            _ => {}
        }
    }

    pub(crate) fn process_tap_count(&mut self, digit: usize, pos: Vec2, time: f64) -> u32 {
        if digit >= self.pointers.len() {
            return 0;
        };
        let (last_pos, last_time, count) = self.pointers[digit].tap_count;

        if (time - last_time) < 0.5 && pos.distance(&last_pos) < 10. {
            self.pointers[digit].tap_count = (pos, time, count + 1);
            count + 1
        } else {
            self.pointers[digit].tap_count = (pos, time, 1);
            1
        }
    }

    pub fn get_dpi_factor_of(&mut self, area: &Area) -> f32 {
        match area {
            Area::InstanceRange(ia) => {
                let pass_id = self.views[ia.view_id].pass_id;
                return self.get_delegated_dpi_factor(pass_id);
            }
            Area::View(va) => {
                let pass_id = self.views[va.view_id].pass_id;
                return self.get_delegated_dpi_factor(pass_id);
            }
            _ => (),
        }
        1.0
    }

    pub(crate) fn get_delegated_dpi_factor(&mut self, pass_id: usize) -> f32 {
        let mut dpi_factor = 1.0;
        let mut pass_id_walk = pass_id;
        for _ in 0..25 {
            match self.passes[pass_id_walk].dep_of {
                CxPassDepOf::Window(window_id) => {
                    dpi_factor = match self.windows[window_id].window_state {
                        CxWindowState::Create { .. } => self.default_dpi_factor,
                        CxWindowState::Created => self.windows[window_id].window_geom.dpi_factor,
                        _ => 1.0,
                    };
                    break;
                }
                CxPassDepOf::Pass(next_pass_id) => {
                    pass_id_walk = next_pass_id;
                }
                _ => {
                    break;
                }
            }
        }
        dpi_factor
    }

    pub(crate) fn compute_passes_to_repaint(&mut self, passes_todo: &mut Vec<usize>, windows_need_repaint: &mut usize) {
        passes_todo.truncate(0);

        loop {
            let mut altered = false; // yes this is horrible but im tired and i dont know why recursion fails
            for pass_id in 0..self.passes.len() {
                if self.passes[pass_id].paint_dirty {
                    let other = match self.passes[pass_id].dep_of {
                        CxPassDepOf::Pass(dep_of_pass_id) => Some(dep_of_pass_id),
                        _ => None,
                    };
                    if let Some(other) = other {
                        if !self.passes[other].paint_dirty {
                            self.passes[other].paint_dirty = true;
                            altered = true;
                        }
                    }
                }
            }
            if !altered {
                break;
            }
        }

        for (pass_id, cxpass) in self.passes.iter().enumerate() {
            if cxpass.paint_dirty {
                let mut inserted = false;
                match cxpass.dep_of {
                    CxPassDepOf::Window(_) => *windows_need_repaint += 1,
                    CxPassDepOf::Pass(dep_of_pass_id) => {
                        if pass_id == dep_of_pass_id {
                            eprintln!("WHAAAT");
                        }
                        for insert_before in 0..passes_todo.len() {
                            if passes_todo[insert_before] == dep_of_pass_id {
                                passes_todo.insert(insert_before, pass_id);
                                inserted = true;
                                break;
                            }
                        }
                    }
                    CxPassDepOf::None => {
                        // we need to be first
                        passes_todo.insert(0, pass_id);
                        inserted = true;
                    }
                }
                if !inserted {
                    passes_todo.push(pass_id);
                }
            }
        }
    }

    /// Request a new redraw of the application.
    pub fn request_draw(&mut self) {
        if self.panic_redraw {
            #[cfg(debug_assertions)]
            panic!("Panic Redraw triggered")
        }
        self.requested_draw = true;
    }

    /// Sets a [`ComponentId`] that will become [`Cx::key_focus`] when the current events are handled.
    ///
    /// TODO(JP): It's possible to set this during the draw cycle instead of during an
    /// event handler, and then it won't update [`Cx::key_focus`] until the next event
    /// is handled. We should probably guard against that.
    pub fn set_key_focus(&mut self, focus_component_id: Option<ComponentId>) {
        self.next_key_focus = Some(focus_component_id);
    }

    /// Keep the existing key focus during an [`Event::PointerDown`], because otherwise we'll reset
    /// it back to [`Area::Empty`].
    pub fn keep_key_focus(&mut self) {
        self.next_key_focus = Some(self.key_focus);
    }

    /// Reverts back to the previous [`Cx::key_focus`] value.
    ///
    /// TODO(JP): It's possible to set this during the draw cycle instead of during an
    /// event handler, and then it won't update [`Cx::key_focus`] until the next event
    /// is handled. We should probably guard against that.
    pub fn revert_key_focus(&mut self) {
        self.next_key_focus = Some(self.prev_key_focus);
    }

    /// Check if a [`ComponentId`] currently has keyboard focus.
    pub fn has_key_focus(&self, component_id: Option<ComponentId>) -> bool {
        self.key_focus == component_id
    }

    pub(crate) fn process_key_down(&mut self, key_event: KeyEvent) {
        if self.keys_down.iter().any(|k| k.key_code == key_event.key_code) {
            return;
        }
        self.keys_down.push(key_event);
    }

    pub(crate) fn process_key_up(&mut self, key_event: &KeyEvent) {
        for i in 0..self.keys_down.len() {
            if self.keys_down[i].key_code == key_event.key_code {
                self.keys_down.remove(i);
                return;
            }
        }
    }

    pub(crate) fn call_all_keys_up(&mut self) {
        let mut keys_down = Vec::new();
        std::mem::swap(&mut keys_down, &mut self.keys_down);
        for key_event in keys_down {
            self.call_event_handler(&mut Event::KeyUp(key_event))
        }
    }

    pub(crate) fn call_event_handler(&mut self, event: &mut Event) {
        let event_handler = self.event_handler.unwrap();

        unsafe {
            (*event_handler)(self, event);
        }

        // Someone has to call `set_key_focus` or `keep_key_focus` when handling `PointerDown`, otherwise
        // the key focus will be reset.
        if let Event::PointerDown(_) = event {
            if self.next_key_focus.is_none() {
                self.next_key_focus = Some(None);
            }
        }

        if let Some(next_key_focus) = self.next_key_focus {
            if next_key_focus != self.key_focus {
                self.prev_key_focus = self.key_focus;
                self.key_focus = next_key_focus;
                unsafe {
                    (*event_handler)(
                        self,
                        &mut Event::KeyFocus(KeyFocusEvent { prev: self.prev_key_focus, focus: self.key_focus }),
                    );
                }
            }
            self.next_key_focus = None;
        }

        self.temp_default_data.clear();
    }

    pub(crate) fn call_draw_event(&mut self) {
        // self.profile();
        self.in_redraw_cycle = true;
        self.redraw_id += 1;
        self.layout_box_align_list.clear();
        self.debug_logs.clear();

        // TODO(Paras): Terrible hack.
        //
        // For some reason our wasm builds always have an incorrect initial draw, but are immediately
        // fixed on the second draw call. So here, we make sure we have drawn twice before clearing
        // `requested_draw`.
        // Interestingly, native builds do not strictly need this hack, because they draw twice
        // anyways unlike WASM.
        if self.redraw_id > 2 {
            self.requested_draw = false;
        }

        self.call_event_handler(&mut Event::System(SystemEvent::Draw));
        self.in_redraw_cycle = false;
        if !self.view_stack.is_empty() {
            panic!("View stack disaligned, forgot an end_view(cx)");
        }
        if !self.pass_stack.is_empty() {
            panic!("Pass stack disaligned, forgot an end_pass(cx)");
        }
        if !self.window_stack.is_empty() {
            panic!("Window stack disaligned, forgot an end_window(cx)");
        }
        if !self.layout_boxes.is_empty() {
            panic!("LayoutBoxes stack disaligned, forgot an end_box(cx)");
        }
        if !self.shader_group_instance_offsets.is_empty() {
            panic!("Shader group stack disaligned, forgot an end_shader_group()");
        }
        //self.profile();
    }

    pub(crate) fn call_next_frame_event(&mut self) {
        self.requested_next_frame = false;
        self.call_event_handler(&mut Event::NextFrame);
    }

    /// Request an [`Event::NextFrame`].
    pub fn request_next_frame(&mut self) {
        self.requested_next_frame = true;
    }

    /// Create a new [`Signal`], which is used to send and capture custom
    /// events.
    ///
    /// See also [`SignalEvent`].
    pub fn new_signal(&mut self) -> Signal {
        self.last_signal_id += 1;
        Signal { signal_id: self.last_signal_id }
    }

    /// Triggers a new [`SignalEvent`] with the same ID as [`Signal`]. You can
    /// post custom data with it using `status`.
    ///
    /// If you want to fire a [`SignalEvent`] from a thread, use [`Cx::post_signal`]
    /// instead.
    ///
    /// See also [`SignalEvent`].
    pub fn send_signal(&mut self, signal: Signal, status: StatusId) {
        if signal.signal_id == 0 {
            return;
        }
        if let Some(statusses) = self.signals.get_mut(&signal) {
            if !statusses.contains(&status) {
                statusses.insert(status);
            }
        } else {
            let mut new_set = BTreeSet::new();
            new_set.insert(status);
            self.signals.insert(signal, new_set);
        }
    }

    pub(crate) fn call_signals(&mut self) {
        let mut counter = 0;
        while !self.signals.is_empty() {
            counter += 1;
            let mut signals = HashMap::new();
            std::mem::swap(&mut self.signals, &mut signals);

            self.call_event_handler(&mut Event::Signal(SignalEvent { signals }));

            if counter > 100 {
                println!("Signal feedback loop detected");
                break;
            }
        }
    }

    pub const STATUS_HTTP_SEND_OK: StatusId = location_hash!();
    pub const STATUS_HTTP_SEND_FAIL: StatusId = location_hash!();

    /// Change the debug flags, which control various debug functionality.
    /// See [`CxDebugFlags`] for more information on the individual flags.
    ///
    /// Example:
    /// ```ignore
    /// let flags = cx.debug_flags_mut();
    /// flags.draw_tree = CxDebugDrawTree::Instances;
    /// flags.disable_draw_call_batching = true;
    /// ```
    pub fn debug_flags_mut(&mut self) -> &mut CxDebugFlags {
        &mut self.debug_flags
    }

    pub(crate) fn debug_draw_tree(&mut self, view_id: usize) {
        if self.debug_flags.draw_tree == CxDebugDrawTree::DrawTree || self.debug_flags.draw_tree == CxDebugDrawTree::Instances {
            let mut s = String::new();
            let dump_instances = self.debug_flags.draw_tree == CxDebugDrawTree::Instances;
            self.debug_draw_tree_recur(dump_instances, &mut s, view_id, 0);
            crate::log!("{}", &s);
        }
        if self.debug_flags.draw_tree == CxDebugDrawTree::Geometries {
            crate::log!("--------------- Geometries for redraw_id: {} ---------------", self.redraw_id);
            for (index, gpu_geometry) in self.gpu_geometries.iter().enumerate() {
                crate::log!(
                    "{}: vertex data: {} bytes, index data: {} bytes / {} triangles, dirty: {}, usage_count: {}",
                    index,
                    gpu_geometry.geometry.vertices_f32_slice().len() * std::mem::size_of::<f32>(),
                    gpu_geometry.geometry.indices_u32_slice().len() * std::mem::size_of::<f32>(),
                    gpu_geometry.geometry.indices_u32_slice().len() / 3,
                    gpu_geometry.dirty,
                    gpu_geometry.usage_count()
                );
            }
        }
    }

    fn debug_draw_tree_recur(&mut self, dump_instances: bool, s: &mut String, view_id: usize, depth: usize) {
        if view_id >= self.views.len() {
            writeln!(s, "---------- Drawlist still empty ---------").unwrap();
            return;
        }
        let mut indent = String::new();
        for _i in 0..depth {
            indent.push_str("  ");
        }
        let draw_calls_len = self.views[view_id].draw_calls_len;
        if view_id == 0 {
            writeln!(s, "---------- Begin Debug draw tree for redraw_id: {} ---------", self.redraw_id).unwrap();
        }
        writeln!(
            s,
            "{}view {}: len:{} rect:{:?} scroll:{:?}",
            indent, view_id, draw_calls_len, self.views[view_id].rect, self.views[view_id].snapped_scroll
        )
        .unwrap();
        indent.push_str("  ");
        for draw_call_id in 0..draw_calls_len {
            let sub_view_id = self.views[view_id].draw_calls[draw_call_id].sub_view_id;
            if sub_view_id != 0 {
                self.debug_draw_tree_recur(dump_instances, s, sub_view_id, depth + 1);
            } else {
                let cxview = &mut self.views[view_id];
                let draw_call = &mut cxview.draw_calls[draw_call_id];
                let sh = &self.shaders[draw_call.shader_id];
                let slots = sh.mapping.instance_props.total_slots;
                let instances = draw_call.instances.len() / slots;
                writeln!(
                    s,
                    "{}call {}: {}({}) *:{} scroll: {} draw_local_scroll: {}",
                    indent,
                    draw_call_id,
                    sh.name,
                    draw_call.shader_id,
                    instances,
                    vec2(draw_call.draw_uniforms.draw_scroll_x, draw_call.draw_uniforms.draw_scroll_y),
                    vec2(draw_call.draw_uniforms.draw_local_scroll_x, draw_call.draw_uniforms.draw_local_scroll_y)
                )
                .unwrap();
                if dump_instances {
                    for inst in 0..instances.min(1) {
                        let mut out = String::new();
                        let mut off = 0;
                        for prop in &sh.mapping.instance_props.props {
                            match prop.slots {
                                1 => out.push_str(&format!("{}:{} ", prop.name, draw_call.instances[inst * slots + off])),
                                2 => out.push_str(&format!(
                                    "{}:v2({},{}) ",
                                    prop.name,
                                    draw_call.instances[inst * slots + off],
                                    draw_call.instances[inst * slots + 1 + off]
                                )),
                                3 => out.push_str(&format!(
                                    "{}:v3({},{},{}) ",
                                    prop.name,
                                    draw_call.instances[inst * slots + off],
                                    draw_call.instances[inst * slots + 1 + off],
                                    draw_call.instances[inst * slots + 1 + off]
                                )),
                                4 => out.push_str(&format!(
                                    "{}:v4({},{},{},{}) ",
                                    prop.name,
                                    draw_call.instances[inst * slots + off],
                                    draw_call.instances[inst * slots + 1 + off],
                                    draw_call.instances[inst * slots + 2 + off],
                                    draw_call.instances[inst * slots + 3 + off]
                                )),
                                _ => {}
                            }
                            off += prop.slots;
                        }
                        writeln!(s, "  {}instance {}: {}", indent, inst, out).unwrap();
                    }
                }
            }
        }
        if view_id == 0 {
            writeln!(s, "---------- End Debug draw tree for redraw_id: {} ---------", self.redraw_id).unwrap();
        }
    }

    /// Register function to handle `callRustSync` from JavaScript. Registered function must be a method on the main app.
    pub fn on_call_rust_async<T: 'static>(
        &mut self,
        func: fn(this: &mut T, cx: &mut Cx, name: String, params: Vec<ZapParam>) -> Vec<ZapParam>,
    ) {
        if self.call_rust_async_fn.is_some() {
            panic!("Attempting to call on_call_rust_async twice.");
        }

        if self.app_type_id != TypeId::of::<T>() {
            panic!("Error in on_call_rust_async: Function must be a method on the main_app.");
        }
        self.call_rust_async_fn = Some(Box::into_raw(Box::new(func)) as usize);
    }

    /// Set the callback for `zaplib.callRustSync` calls.
    ///
    /// Can only be called in the `new` function of your app, since we do some thread-unsafe
    /// operations in this, and during `new` there are no threads yet. We make the assertion
    /// here for consistency, but it's primarily for `on_call_rust_sync_internal`
    /// in `cx_wasm32.rs`.
    #[allow(unused_variables)] // `func` is unused when not matching the `cfg` below.
    pub fn on_call_rust_sync(&mut self, func: CallRustSyncFn) {
        assert!(!self.finished_app_new, "Can only call cx.on_call_rust_sync in `new`");

        #[cfg(any(target_arch = "wasm32", feature = "cef"))]
        self.on_call_rust_sync_internal(func);
    }

    pub fn call_rust_sync_dispatch(func: CallRustSyncFn, name: String, mut params: Vec<ZapParam>) -> Vec<ZapParam> {
        if name == "__zaplibCreateMutableBuffer" {
            let param_type = params[0].as_str().parse::<usize>().unwrap();
            let size = params[1].as_str().parse::<usize>().unwrap();

            if param_type == 1 || param_type == 2 {
                vec![vec![0u8; size].into_param()]
            } else if param_type == 3 || param_type == 4 {
                vec![vec![0f32; size].into_param()]
            } else {
                panic!("Unknown param type {}", param_type);
            }
        } else if name == "__zaplibMakeBufferReadOnly" {
            match params.remove(0) {
                ZapParam::MutableU8Buffer(v) => vec![Arc::new(v).into_param()],
                ZapParam::MutableF32Buffer(v) => vec![Arc::new(v).into_param()],
                _ => panic!("Unknown param type"),
            }
        } else {
            func(name, params)
        }
    }

    /// Mark that the `new` function of the main app has been called. Automatically called by the `main_app!` macro;
    /// don't call this in user code.
    pub fn set_finished_app_new(&mut self) {
        self.finished_app_new = true;
    }
}

/// A bunch of traits that are common between the native platforms and the WebAssembly platform. This trait makes sure
/// that there is consistency in the interface, and provides one place for documentation.
pub trait CxDesktopVsWasmCommon {
    /// Get a default window size for new windows.
    /// TODO(JP): This doesn't make too much sense for Wasm; maybe just omit this method there?
    fn get_default_window_size(&self) -> Vec2;

    /// Write `data` to `path`.
    fn file_write(&mut self, path: &str, data: &[u8]);

    /// Send data over a Websocket.
    fn websocket_send(&mut self, url: &str, data: &[u8]);

    /// Make an HTTP request. When done, you get a [`SignalEvent`] corresponding to the provided
    /// [`Signal`] with [`Cx::STATUS_HTTP_SEND_OK`] or [`Cx::STATUS_HTTP_SEND_FAIL`] as the status.
    fn http_send(
        &mut self,
        verb: &str,
        path: &str,
        _proto: &str,
        domain: &str,
        port: u16,
        content_type: &str,
        body: &[u8],
        signal: Signal,
    );

    /// Call JS function from Rust. Must be called on main thread and call a function already
    /// registered using `register_call_js_callbacks`. `params` is an arbitrary string.
    /// `buffers` is an array of reference-counted byte buffers.
    #[cfg(any(target_arch = "wasm32", feature = "cef"))]
    fn call_js(&mut self, name: &str, params: Vec<ZapParam>);

    /// Mechanism to communicate back returns values from `callRustAsync` functions.
    fn return_to_js(&mut self, callback_id: u32, params: Vec<ZapParam>);
}

/// A bunch of traits that are common between the different target platforms. This trait makes sure
/// that there is consistency in the interface, and provides one place for documentation.
pub trait CxPlatformCommon {
    /// Show an [Input Method Editor (IME)](https://en.wikipedia.org/wiki/Input_method) at a particular
    /// location, typically with everything but the cursor hidden.
    fn show_text_ime(&mut self, x: f32, y: f32);
    /// Hide the IME shown by [`CxPlatformCommon::show_text_ime`].
    fn hide_text_ime(&mut self);
    /// Start a new [`Timer`] with the given `interval`, and which may `repeat` if required.
    fn start_timer(&mut self, interval: f64, repeats: bool) -> Timer;
    /// Stop zap`Timer`] given by [`CxPlatformCommon::start_timer`].
    fn stop_timer(&mut self, timer: &mut Timer);
    /// Post a [`Signal`] from any thread. If you don't need to use this from a thread, you may
    /// instead use [`Cx::send_signal`], which might be faster.
    fn post_signal(signal: Signal, status: StatusId);
    /// Set a [`Menu`].
    fn update_menu(&mut self, menu: &Menu);
    /// Copy the given text to the clipboard, if possible.
    fn copy_text_to_clipboard(&mut self, text: &str);
    /// Send zaplib Event for processing from any thread
    fn send_event_from_any_thread(event: Event);
}
