use crate::{axis::*, ScrollBarConfig};
use zaplib::*;

#[derive(Clone, Copy, Default)]
#[repr(C)]
struct ScrollBarIns {
    base: QuadIns,
    color: Vec4,
    is_vertical: f32,
    norm_handle: f32,
    norm_scroll: f32,
}

static SHADER: Shader = Shader {
    build_geom: Some(QuadIns::build_geom),
    code_to_concatenate: &[
        Cx::STD_SHADER,
        QuadIns::SHADER,
        code_fragment!(
            r#"
            instance color: vec4;
            instance is_vertical: float;
            instance norm_handle: float;
            instance norm_scroll: float;
            const border_radius: float = 1.5;

            fn pixel() -> vec4 {
                let df = Df::viewport(pos * rect_size);
                if is_vertical > 0.5 {
                    df.box(
                        vec2(1., rect_size.y * norm_scroll),
                        vec2(rect_size.x * 0.5, rect_size.y * norm_handle),
                        border_radius
                    );
                }
                else {
                    df.box(
                        vec2(rect_size.x * norm_scroll, 1.),
                        vec2(rect_size.x * norm_handle, rect_size.y * 0.5),
                        border_radius
                    );
                }
                return df.fill(color);
            }"#
        ),
    ],
    ..Shader::DEFAULT
};

#[derive(Debug)]
pub(crate) struct ScrollBar {
    component_id: ComponentId,
    bg_area: Area,

    config: ScrollBarConfig,

    min_handle_size: f32, //minimum size of the handle in pixels
    axis: Axis,
    animator: Animator,

    visible: bool,
    bar_side_margin: f32,
    view_area: Area,
    view_total: f32,   // the total view area
    view_visible: f32, // the visible view area
    scroll_size: f32,  // the size of the scrollbar
    scroll_pos: f32,   // scrolling position non normalised

    scroll_target: f32,
    scroll_delta: f32,

    drag_point: Option<f32>, // the point in pixels where we are dragging
}

#[derive(Clone, PartialEq, Debug)]
pub enum ScrollBarEvent {
    None,
    Scroll { scroll_pos: f32, view_total: f32, view_visible: f32 },
    ScrollDone,
}

const ANIM_DEFAULT: Anim = Anim {
    duration: 0.5,
    // ScrollBarIns::color
    tracks: &[Track::Vec4 { key_frames: &[(1.0, vec4(0.33, 0.33, 0.33, 1.))], ease: Ease::DEFAULT }],
    ..Anim::DEFAULT
};

const ANIM_OVER: Anim = Anim {
    duration: 0.05,
    // ScrollBarIns::color
    tracks: &[Track::Vec4 { key_frames: &[(1.0, vec4(0.47, 0.47, 0.47, 1.))], ease: Ease::DEFAULT }],
    ..Anim::DEFAULT
};

const ANIM_DOWN: Anim = Anim {
    duration: 0.05,
    // ScrollBarIns::color
    tracks: &[Track::Vec4 { key_frames: &[(1.0, vec4(0.6, 0.6, 0.6, 1.))], ease: Ease::DEFAULT }],
    ..Anim::DEFAULT
};

impl Default for ScrollBar {
    fn default() -> Self {
        Self {
            component_id: Default::default(),
            config: ScrollBarConfig::default(),

            min_handle_size: 30.0,
            axis: Axis::Horizontal,
            animator: Animator::default(),
            bg_area: Area::Empty,

            visible: false,

            view_area: Area::Empty,
            view_total: 0.0,
            view_visible: 0.0,
            bar_side_margin: 6.0,
            scroll_size: 0.0,
            scroll_pos: 0.0,

            scroll_target: 0.0,
            scroll_delta: 0.0,

            drag_point: None,
        }
    }
}

impl ScrollBar {
    pub(crate) fn new(config: ScrollBarConfig) -> Self {
        Self { config, ..ScrollBar::default() }
    }

    fn animate(&mut self, cx: &mut Cx) {
        let bg = self.bg_area.get_first_mut::<ScrollBarIns>(cx);
        bg.color = self.animator.get_vec4(0);
    }

    // reads back normalized scroll position info
    fn get_normalized_scroll_pos(&self) -> (f32, f32) {
        // computed handle size normalized
        let vy = self.view_visible / self.view_total;
        if !self.visible {
            return (0.0, 0.0);
        }
        let norm_handle = vy.max(self.min_handle_size / self.scroll_size);
        let norm_scroll = (1. - norm_handle) * ((self.scroll_pos / self.view_total) / (1. - vy));
        (norm_scroll, norm_handle)
    }

    // sets the scroll pos from pointer position
    fn set_scroll_pos_from_pointer(&mut self, cx: &mut Cx, pointer: f32) -> ScrollBarEvent {
        let vy = self.view_visible / self.view_total;
        let norm_handle = vy.max(self.min_handle_size / self.scroll_size);

        let new_scroll_pos = ((self.view_total * (1. - vy) * (pointer / self.scroll_size)) / (1. - norm_handle))
            .max(0.)
            .min(self.view_total - self.view_visible);

        // lets snap new_scroll_pos
        let changed = self.scroll_pos != new_scroll_pos;
        self.scroll_pos = new_scroll_pos;
        self.scroll_target = new_scroll_pos;
        if changed {
            self.update_shader_scroll_pos(cx);
            return self.make_scroll_event();
        }
        ScrollBarEvent::None
    }

    // writes the norm_scroll value into the shader
    fn update_shader_scroll_pos(&mut self, cx: &mut Cx) {
        let bg = self.bg_area.get_first_mut::<ScrollBarIns>(cx);
        bg.norm_scroll = self.get_normalized_scroll_pos().0;
    }

    // turns scroll_pos into an event on this.event
    fn make_scroll_event(&mut self) -> ScrollBarEvent {
        ScrollBarEvent::Scroll { scroll_pos: self.scroll_pos, view_total: self.view_total, view_visible: self.view_visible }
    }

    fn move_towards_scroll_target(&mut self, cx: &mut Cx) -> bool {
        if self.config.smoothing.is_none() {
            return false;
        }
        if (self.scroll_target - self.scroll_pos).abs() < 0.01 {
            return false;
        }
        if self.scroll_pos > self.scroll_target {
            // go back
            self.scroll_pos += (self.config.smoothing.unwrap() * self.scroll_delta).min(-1.);
            if self.scroll_pos <= self.scroll_target {
                // hit the target
                self.scroll_pos = self.scroll_target;
                self.update_shader_scroll_pos(cx);
                return false;
            }
        } else {
            // go forward
            self.scroll_pos += (self.config.smoothing.unwrap() * self.scroll_delta).max(1.);
            if self.scroll_pos > self.scroll_target {
                // hit the target
                self.scroll_pos = self.scroll_target;
                self.update_shader_scroll_pos(cx);
                return false;
            }
        }
        self.update_shader_scroll_pos(cx);
        true
    }

    pub(crate) fn get_scroll_pos(&self) -> f32 {
        self.scroll_pos
    }

    pub(crate) fn set_scroll_pos(&mut self, cx: &mut Cx, scroll_pos: f32) -> bool {
        // clamp scroll_pos to
        let scroll_pos = scroll_pos.min(self.view_total - self.view_visible).max(0.);

        if self.scroll_pos != scroll_pos {
            self.scroll_pos = scroll_pos;
            self.scroll_target = scroll_pos;
            self.update_shader_scroll_pos(cx);
            cx.request_next_frame();
            return true;
        };
        false
    }

    pub(crate) fn set_scroll_view_total(&mut self, _cx: &mut Cx, view_total: f32) {
        self.view_total = view_total;
    }

    pub(crate) fn get_scroll_view_total(&self) -> f32 {
        self.view_total
    }

    pub(crate) fn set_scroll_target(&mut self, cx: &mut Cx, scroll_pos_target: f32) -> bool {
        // clamp scroll_pos to

        let new_target = scroll_pos_target.min(self.view_total - self.view_visible).max(0.);
        if self.scroll_target != new_target {
            self.scroll_target = new_target;
            self.scroll_delta = new_target - self.scroll_pos;
            cx.request_next_frame();
            return true;
        };
        false
    }

    pub(crate) fn scroll_into_view(&mut self, cx: &mut Cx, pos: f32, size: f32, smooth: bool) {
        if pos < self.scroll_pos {
            // scroll up
            let scroll_to = pos;
            if !smooth || self.config.smoothing.is_none() {
                self.set_scroll_pos(cx, scroll_to);
            } else {
                self.set_scroll_target(cx, scroll_to);
            }
        } else if pos + size > self.scroll_pos + self.view_visible {
            // scroll down
            let scroll_to = (pos + size) - self.view_visible;
            if pos + size > self.view_total {
                // resize _view_total if need be
                self.view_total = pos + size;
            }
            if !smooth || self.config.smoothing.is_none() {
                self.set_scroll_pos(cx, scroll_to);
            } else {
                self.set_scroll_target(cx, scroll_to);
            }
        }
    }

    pub(crate) fn handle(&mut self, cx: &mut Cx, event: &mut Event) -> ScrollBarEvent {
        // lets check if our view-area gets a mouse-scroll.
        match event {
            Event::PointerScroll(pe) => {
                //if !pe.handled {
                if !match self.axis {
                    Axis::Horizontal => pe.handled_x,
                    Axis::Vertical => pe.handled_y,
                } {
                    let rect = self.view_area.get_rect_for_first_instance(cx).unwrap_or_default();
                    if rect.contains(pe.abs) {
                        // handle mousewheel
                        // we should scroll in either x or y
                        let scroll = match self.axis {
                            Axis::Horizontal => {
                                if self.config.use_vertical_pointer_scroll {
                                    pe.scroll.y
                                } else {
                                    pe.scroll.x
                                }
                            }
                            Axis::Vertical => pe.scroll.y,
                        };
                        // TODO(Dmitry): there used to be special handling for the
                        // self.smoothing.is_some() && pe.input_type.is_mouse() case to allow smoother scrolling
                        // but this seemed to be buggy and causing scrolled posiition and internal state going
                        // out of sync, so we are removing it in favor of general approach.
                        // We can re-introduce smooth scrolling later in the future
                        let scroll_pos = self.get_scroll_pos();
                        if self.set_scroll_pos(cx, scroll_pos + scroll) {
                            match self.axis {
                                Axis::Horizontal => pe.handled_x = true,
                                Axis::Vertical => pe.handled_y = true,
                            }
                        }
                        return self.make_scroll_event();
                    }
                }
            }

            _ => (),
        };
        if self.visible {
            if self.animator.handle(cx, event) {
                self.animate(cx);
            }
            if let Event::NextFrame = event {
                if self.move_towards_scroll_target(cx) {
                    cx.request_next_frame();
                    return self.make_scroll_event();
                }
            }

            match event.hits_pointer(cx, self.component_id, self.bg_area.get_rect_for_first_instance(cx)) {
                Event::PointerDown(pe) => {
                    cx.keep_key_focus();
                    self.animator.play_anim(cx, ANIM_DOWN);
                    let rel = match self.axis {
                        Axis::Horizontal => pe.rel.x,
                        Axis::Vertical => pe.rel.y,
                    };
                    cx.set_down_mouse_cursor(MouseCursor::Default);
                    let (norm_scroll, norm_handle) = self.get_normalized_scroll_pos();
                    let bar_start = norm_scroll * self.scroll_size;
                    let bar_size = norm_handle * self.scroll_size;
                    if rel < bar_start || rel > bar_start + bar_size {
                        // clicked outside
                        self.drag_point = Some(bar_size * 0.5);
                        return self.set_scroll_pos_from_pointer(cx, rel - self.drag_point.unwrap());
                    } else {
                        // clicked on
                        self.drag_point = Some(rel - bar_start); // store the drag delta
                    }
                }
                Event::PointerHover(pe) => {
                    if self.drag_point.is_none() {
                        cx.set_hover_mouse_cursor(MouseCursor::Default);
                        match pe.hover_state {
                            HoverState::In => {
                                self.animator.play_anim(cx, ANIM_OVER);
                            }
                            HoverState::Out => {
                                self.animator.play_anim(cx, ANIM_DEFAULT);
                            }
                            _ => (),
                        }
                    }
                }
                Event::PointerUp(pe) => {
                    self.drag_point = None;
                    if pe.is_over {
                        if pe.input_type.has_hovers() {
                            self.animator.play_anim(cx, ANIM_OVER);
                        } else {
                            self.animator.play_anim(cx, ANIM_DEFAULT);
                        }
                    } else {
                        self.animator.play_anim(cx, ANIM_DEFAULT);
                    }
                    return ScrollBarEvent::ScrollDone;
                }
                Event::PointerMove(pe) => {
                    // helper called by event code to scroll from a pointer
                    if self.drag_point.is_none() {
                        // state should never occur.
                        //println!("Invalid state in scrollbar, pointerMove whilst drag_point is none")
                    } else {
                        match self.axis {
                            Axis::Horizontal => {
                                return self.set_scroll_pos_from_pointer(cx, pe.rel.x - self.drag_point.unwrap());
                            }
                            Axis::Vertical => {
                                return self.set_scroll_pos_from_pointer(cx, pe.rel.y - self.drag_point.unwrap());
                            }
                        }
                    }
                }
                _ => (),
            };
        }

        ScrollBarEvent::None
    }

    pub(crate) fn draw(&mut self, cx: &mut Cx, axis: Axis, view_area: Area, view_rect: Rect, view_total: Vec2) -> f32 {
        self.view_area = view_area;
        self.axis = axis;

        match self.axis {
            Axis::Horizontal => {
                self.visible = view_total.x > view_rect.size.x + 0.1;
                self.scroll_size = if view_total.y > view_rect.size.y + 0.1 {
                    view_rect.size.x - self.config.bar_size
                } else {
                    view_rect.size.x
                } - self.bar_side_margin * 2.;
                self.view_total = view_total.x;
                self.view_visible = view_rect.size.x;
                self.scroll_pos = self.scroll_pos.min(self.view_total - self.view_visible).max(0.);

                if self.visible {
                    let (norm_scroll, norm_handle) = self.get_normalized_scroll_pos();
                    self.bg_area = cx.add_instances_with_scroll_sticky(
                        &SHADER,
                        &[ScrollBarIns {
                            base: QuadIns::from_rect(Rect {
                                pos: cx.get_box_origin() + vec2(self.bar_side_margin, view_rect.size.y - self.config.bar_size),
                                size: vec2(self.scroll_size, self.config.bar_size),
                            })
                            .with_draw_depth(2.5),
                            is_vertical: 0.0,
                            norm_handle,
                            norm_scroll,
                            ..Default::default()
                        }],
                        true,
                        true,
                    );
                }
            }
            Axis::Vertical => {
                self.visible = view_total.y > view_rect.size.y + 0.1;
                self.scroll_size = if view_total.x > view_rect.size.x + 0.1 {
                    view_rect.size.y - self.config.bar_size
                } else {
                    view_rect.size.y
                } - self.bar_side_margin * 2.;
                self.view_total = view_total.y;
                self.view_visible = view_rect.size.y;
                self.scroll_pos = self.scroll_pos.min(self.view_total - self.view_visible).max(0.);

                if self.visible {
                    let (norm_scroll, norm_handle) = self.get_normalized_scroll_pos();
                    self.bg_area = cx.add_instances_with_scroll_sticky(
                        &SHADER,
                        &[ScrollBarIns {
                            base: QuadIns::from_rect(Rect {
                                pos: cx.get_box_origin() + vec2(view_rect.size.x - self.config.bar_size, self.bar_side_margin),
                                size: vec2(self.config.bar_size, self.scroll_size),
                            })
                            .with_draw_depth(2.5),
                            is_vertical: 1.0,
                            norm_handle,
                            norm_scroll,
                            ..Default::default()
                        }],
                        true,
                        true,
                    );
                }
            }
        }

        // see if we need to clamp
        let clamped_pos = self.scroll_pos.min(self.view_total - self.view_visible).max(0.);
        if clamped_pos != self.scroll_pos {
            self.scroll_pos = clamped_pos;
            self.scroll_target = clamped_pos;
            // ok so this means we 'scrolled' this can give a problem for virtual viewport widgets
            cx.request_next_frame();
        }

        if self.visible {
            self.animator.draw(cx, ANIM_DEFAULT);
            self.animate(cx);
        }

        self.scroll_pos
    }
}
