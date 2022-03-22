use zaplib::*;

use crate::{internal::tabclose::TabClose, ButtonEvent};

#[derive(Clone, Copy, Default)]
#[repr(C)]
struct TabIns {
    base: QuadIns,
    color: Vec4,
    border_color: Vec4,
}

static SHADER: Shader = Shader {
    build_geom: Some(QuadIns::build_geom),
    code_to_concatenate: &[
        Cx::STD_SHADER,
        QuadIns::SHADER,
        code_fragment!(
            r#"
            instance color: vec4;
            instance border_color: vec4;
            const border_width: float = 1.0;

            fn pixel() -> vec4 {
                let df = Df::viewport(pos * rect_size);
                df.rect(vec2(-1.), rect_size + 2.);
                df.fill(color);
                df.new_path();
                df.move_to(vec2(rect_size.x, 0.));
                df.line_to(rect_size);
                df.move_to(vec2(0.));
                df.line_to(vec2(0., rect_size.y));
                return df.stroke(border_color, 1.);
            }"#
        ),
    ],
    ..Shader::DEFAULT
};

#[derive(Default)]
pub struct Tab {
    component_id: ComponentId,
    bg_area: Area,
    text_area: Area,
    pub label: String,
    animator: Animator,
    draw_depth: f32,
    pub abs_origin: Option<Vec2>,
    is_selected: bool,
    pub is_closeable: bool,
    tab_close: TabClose,
    is_focussed: bool,
    is_down: bool,
    is_drag: bool,
}

#[derive(Clone, PartialEq)]
pub enum TabEvent {
    None,
    DragMove(PointerMoveEvent),
    DragEnd(PointerUpEvent),
    Closing,
    Close,
    Select,
}

const TAB_HEIGHT: f32 = 40.;

const COLOR_BG_SELECTED: Vec4 = vec4(0.16, 0.16, 0.16, 1.0);
const COLOR_BG_NORMAL: Vec4 = vec4(0.2, 0.2, 0.2, 1.0);

const COLOR_TEXT_SELECTED_FOCUS: Vec4 = Vec4::all(1.);
const COLOR_TEXT_DESELECTED_FOCUS: Vec4 = vec4(0.62, 0.62, 0.62, 1.0);
const COLOR_TEXT_SELECTED_DEFOCUS: Vec4 = vec4(0.62, 0.62, 0.62, 1.0);
const COLOR_TEXT_DESELECTED_DEFOCUS: Vec4 = vec4(0.51, 0.51, 0.51, 1.0);

const ANIM_DESELECTED_DEFOCUS: Anim = Anim {
    duration: 0.05,
    tracks: &[
        // TabIns::color
        Track::Vec4 { ease: Ease::Lin, key_frames: &[(1.0, COLOR_BG_NORMAL)] },
        // TabIns::border_color
        Track::Vec4 { ease: Ease::Lin, key_frames: &[(1.0, COLOR_BG_SELECTED)] },
        // TextIns::color
        Track::Vec4 { ease: Ease::Lin, key_frames: &[(1.0, COLOR_TEXT_DESELECTED_DEFOCUS)] },
    ],
    ..Anim::DEFAULT
};

const ANIM_DESELECTED_FOCUS: Anim = Anim {
    duration: 0.05,
    tracks: &[
        // TabIns::color
        Track::Vec4 { ease: Ease::Lin, key_frames: &[(1.0, COLOR_BG_NORMAL)] },
        // TabIns::border_color
        Track::Vec4 { ease: Ease::Lin, key_frames: &[(1.0, COLOR_BG_SELECTED)] },
        // TextIns::color
        Track::Vec4 { ease: Ease::Lin, key_frames: &[(1.0, COLOR_TEXT_DESELECTED_FOCUS)] },
    ],
    ..Anim::DEFAULT
};

const ANIM_SELECTED_DEFOCUS: Anim = Anim {
    duration: 0.05,
    tracks: &[
        // TabIns::color
        Track::Vec4 { ease: Ease::Lin, key_frames: &[(1.0, COLOR_BG_SELECTED)] },
        // TabIns::border_color
        Track::Vec4 { ease: Ease::Lin, key_frames: &[(1.0, COLOR_BG_SELECTED)] },
        // TextIns::color
        Track::Vec4 { ease: Ease::Lin, key_frames: &[(1.0, COLOR_TEXT_SELECTED_DEFOCUS)] },
    ],
    ..Anim::DEFAULT
};

const ANIM_SELECTED_FOCUS: Anim = Anim {
    duration: 0.05,
    tracks: &[
        // TabIns::color
        Track::Vec4 { ease: Ease::Lin, key_frames: &[(1.0, COLOR_BG_SELECTED)] },
        // TabIns::border_color
        Track::Vec4 { ease: Ease::Lin, key_frames: &[(1.0, COLOR_BG_SELECTED)] },
        // TextIns::color
        Track::Vec4 { ease: Ease::Lin, key_frames: &[(1.0, COLOR_TEXT_SELECTED_FOCUS)] },
    ],
    ..Anim::DEFAULT
};

impl Tab {
    #[must_use]
    pub fn new() -> Self {
        Self { label: "Tab".to_string(), is_closeable: true, ..Default::default() }
    }

    #[must_use]
    pub fn with_draw_depth(self, draw_depth: f32) -> Self {
        Self { draw_depth, ..self }
    }

    fn animate(&mut self, cx: &mut Cx) {
        let bg = self.bg_area.get_first_mut::<TabIns>(cx);
        bg.color = self.animator.get_vec4(0);
        bg.border_color = self.animator.get_vec4(1);
        TextIns::set_color(cx, self.text_area, self.animator.get_vec4(2));
    }

    fn play_anim(&mut self, cx: &mut Cx) {
        if self.is_selected {
            if self.is_focussed {
                self.animator.play_anim(cx, ANIM_SELECTED_FOCUS);
            } else {
                self.animator.play_anim(cx, ANIM_SELECTED_DEFOCUS);
            }
        } else if self.is_focussed {
            self.animator.play_anim(cx, ANIM_DESELECTED_FOCUS);
        } else {
            self.animator.play_anim(cx, ANIM_DESELECTED_DEFOCUS);
        }
    }

    pub fn set_tab_focus(&mut self, cx: &mut Cx, focus: bool) {
        if focus != self.is_focussed {
            self.is_focussed = focus;
            self.play_anim(cx);
        }
    }

    pub fn set_tab_selected(&mut self, cx: &mut Cx, selected: bool) {
        if selected != self.is_selected {
            self.is_selected = selected;
            self.play_anim(cx);
        }
    }

    pub fn set_tab_state(&mut self, cx: &mut Cx, selected: bool, focus: bool) {
        self.is_selected = selected;
        self.is_focussed = focus;
        self.play_anim(cx);
    }

    pub fn handle(&mut self, cx: &mut Cx, event: &mut Event) -> TabEvent {
        if self.animator.handle(cx, event) {
            self.animate(cx);
        }

        match self.tab_close.handle(cx, event) {
            ButtonEvent::Down => {
                return TabEvent::Close;
            }
            _ => (),
        }

        match event.hits_pointer(cx, self.component_id, self.bg_area.get_rect_for_first_instance(cx)) {
            Event::PointerDown(_pe) => {
                cx.set_down_mouse_cursor(MouseCursor::Hand);
                self.is_down = true;
                self.is_drag = false;
                self.is_selected = true;
                self.is_focussed = true;
                self.play_anim(cx);
                return TabEvent::Select;
            }
            Event::PointerHover(_pe) => {
                cx.set_hover_mouse_cursor(MouseCursor::Hand);
            }
            Event::PointerUp(pe) => {
                self.is_down = false;

                if self.is_drag {
                    self.is_drag = false;
                    return TabEvent::DragEnd(pe);
                }
            }
            Event::PointerMove(pe) => {
                if !self.is_drag && pe.move_distance() > 50. {
                    //cx.set_down_mouse_cursor(MouseCursor::Hidden);
                    self.is_drag = true;
                }
                if self.is_drag {
                    return TabEvent::DragMove(pe);
                }
            }
            _ => (),
        };
        TabEvent::None
    }

    pub fn get_tab_rect(&self, cx: &mut Cx) -> Rect {
        let bg = self.bg_area.get_first::<TabIns>(cx);
        bg.base.rect()
    }

    pub fn begin_tab(&mut self, cx: &mut Cx) {
        if let Some(abs_origin) = self.abs_origin {
            // Set tab position by absolute coordinates
            cx.begin_absolute_box();
            cx.begin_padding_box(Padding { l: abs_origin.x, t: abs_origin.y, r: 0., b: 0. });
        };

        self.bg_area = cx.add_instances(&SHADER, &[TabIns::default()]);

        cx.begin_row(Width::Compute, Height::Fix(TAB_HEIGHT)); // tab
        cx.begin_padding_box(Padding { l: 16.0, t: 1.0, r: 16.0, b: 0.0 }); // tab content

        cx.begin_row(Width::Compute, Height::Fix(TAB_HEIGHT));
        cx.begin_center_y_align();
        let draw_str_props = TextInsProps { draw_depth: self.draw_depth, ..TextInsProps::DEFAULT };
        self.text_area = TextIns::draw_walk(cx, &self.label, &draw_str_props);
        cx.end_center_y_align();
        cx.end_row();

        if self.is_closeable {
            cx.begin_row(Width::Fix(10.), Height::Fix(TAB_HEIGHT));
            cx.begin_center_y_align();
            self.tab_close.draw(cx);
            cx.end_center_y_align();
            cx.end_row();
        }
    }

    pub fn end_tab(&mut self, cx: &mut Cx) {
        cx.end_padding_box(); // tab content
        let rect = cx.end_row(); // tab

        // We need to close corresponding boxes which we opened in absolute mode
        if self.abs_origin.is_some() {
            cx.end_padding_box(); // close padding_box for absolute box
            cx.end_absolute_box();
        }

        let bg = self.bg_area.get_first_mut::<TabIns>(cx);
        bg.base = QuadIns::from_rect(rect).with_draw_depth(self.draw_depth);
        self.animator.draw(cx, ANIM_DESELECTED_DEFOCUS);
        self.animate(cx);
    }

    pub fn draw_tab(&mut self, cx: &mut Cx) {
        self.begin_tab(cx);
        self.end_tab(cx);
    }

    pub fn selected(&self) -> bool {
        self.is_selected
    }
}
