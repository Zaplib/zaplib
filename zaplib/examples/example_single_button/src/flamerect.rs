use zaplib::*;

use crate::Span;

#[derive(Clone, PartialEq)]
pub enum FlameRectEvent {
    None,
    Clicked,
    Down,
    Up,
}

#[derive(Clone, Default)]
#[repr(C)]
struct BgIns {
    base: QuadIns,
    hover: f32,
    down: f32,
}

static SHADER: Shader = Shader {
    build_geom: Some(QuadIns::build_geom),
    code_to_concatenate: &[
        Cx::STD_SHADER,
        QuadIns::SHADER,
        code_fragment!(
            r#"
            instance hover: float;
            instance down: float;

            const shadow: float = 3.0;
            const border_radius: float = 2.5;

            fn pixel() -> vec4 {
                let df = Df::viewport(pos * rect_size);
                df.box(vec2(shadow), rect_size - shadow * (1. + down), border_radius);
                df.blur = 6.0;
                df.fill(mix(#0007, #0, hover));
                df.new_path();
                df.blur = 0.001;
                df.box(vec2(shadow), rect_size - shadow * 2., border_radius);
                return df.fill(mix(mix(#3, #4, hover), #2a, down));
                //return #a00;
            }"#
        ),
    ],
    ..Shader::DEFAULT
};

#[derive(Default)]
pub struct FlameRect {
    component_id: ComponentId,
    bg_area: Area,
    text_area: Area,
    animator: Animator,
}

const ANIM_DEFAULT: Anim = Anim {
    duration: 0.05,
    chain: true,
    tracks: &[
        // BgIns::hover
        Track::Float { key_frames: &[(1.0, 0.0)], ease: Ease::DEFAULT },
        // BgIns::down
        Track::Float { key_frames: &[(1.0, 0.0)], ease: Ease::DEFAULT },
        // TextIns::color
        Track::Vec4 { key_frames: &[(1.0, vec4(0.6, 0.6, 0.6, 1.))], ease: Ease::DEFAULT },
    ],
};

const ANIM_HOVER: Anim = Anim {
    duration: 0.05,
    chain: true,
    tracks: &[
        // BgIns::hover
        Track::Float { key_frames: &[(0.0, 1.0)], ease: Ease::DEFAULT },
        // BgIns::down
        Track::Float { key_frames: &[(1.0, 0.0)], ease: Ease::DEFAULT },
        // TextIns::color
        Track::Vec4 { key_frames: &[(1.0, vec4(1., 1., 1., 1.))], ease: Ease::DEFAULT },
    ],
};

const ANIM_DOWN: Anim = Anim {
    duration: 0.1,
    tracks: &[
        // BgIns::hover
        Track::Float { key_frames: &[(0.0, 1.0)], ease: Ease::DEFAULT },
        // BgIns::down
        Track::Float { key_frames: &[(0.0, 0.0), (1.0, 1.0)], ease: Ease::DEFAULT },
        // TextIns::color
        Track::Vec4 { key_frames: &[(1.0, vec4(0.8, 0.8, 0.8, 1.))], ease: Ease::DEFAULT },
    ],
    ..Anim::DEFAULT
};

impl FlameRect {
    fn animate(&mut self, cx: &mut Cx) {
        let draw_bg = self.bg_area.get_first_mut::<BgIns>(cx);
        draw_bg.hover = self.animator.get_float(0);
        draw_bg.down = self.animator.get_float(1);
        TextIns::set_color(cx, self.text_area, self.animator.get_vec4(2));
    }

    pub fn handle(&mut self, cx: &mut Cx, event: &mut Event) -> FlameRectEvent {
        if self.animator.handle(cx, event) {
            self.animate(cx);
        }
        let animator = &mut self.animator;
        let hit_event = event.hits_pointer(cx, self.component_id, self.bg_area.get_rect_for_first_instance(cx));

        match hit_event {
            Event::PointerDown(_pe) => {
                animator.play_anim(cx, ANIM_DOWN);
                return FlameRectEvent::Down;
            }
            Event::PointerHover(pe) => {
                cx.set_hover_mouse_cursor(MouseCursor::Hand);
                match pe.hover_state {
                    HoverState::In => {
                        if pe.any_down {
                            animator.play_anim(cx, ANIM_DOWN);
                        } else {
                            animator.play_anim(cx, ANIM_HOVER);
                        }
                    }
                    HoverState::Out => animator.play_anim(cx, ANIM_DEFAULT),
                    _ => (),
                }
            }
            Event::PointerUp(pe) => {
                if pe.is_over {
                    if pe.input_type.has_hovers() {
                        animator.play_anim(cx, ANIM_HOVER);
                    } else {
                        animator.play_anim(cx, ANIM_DEFAULT);
                    }
                    return FlameRectEvent::Clicked;
                } else {
                    animator.play_anim(cx, ANIM_DEFAULT);
                    return FlameRectEvent::Up;
                }
            }
            _ => (),
        };
        FlameRectEvent::None
    }

    pub fn draw(&mut self, cx: &mut Cx, span: &Span) {
        cx.begin_shader_group(&[&SHADER, &TEXT_INS_SHADER]);

        let rect = Rect {
            pos: cx.get_draw_pos() + cx.get_box_rect().size * span.rect.pos,
            size: cx.get_box_rect().size * span.rect.size,
        };

        self.bg_area = cx.add_instances(&SHADER, &[BgIns { base: QuadIns::from_rect(rect), ..Default::default() }]);
        self.text_area = TextIns::draw_str(cx, &span.label, rect.pos, &Default::default());

        self.animator.draw(cx, ANIM_DEFAULT);
        self.animate(cx);
        cx.end_shader_group();
    }
}
