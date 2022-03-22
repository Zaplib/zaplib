use crate::button::*;
use zaplib::*;

#[derive(Clone, Copy, Default)]
#[repr(C)]
struct TabCloseIns {
    base: QuadIns,
    color: Vec4,
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
            instance color: vec4;
            instance hover: float;
            instance down: float;

            fn pixel() -> vec4 {
                let df = Df::viewport(pos * rect_size);
                let hover_max: float = (hover * 0.4 + 0.3) * 0.5;
                let hover_min: float = 1. - hover_max;
                let c: vec2 = rect_size * 0.5;
                df.circle(c, 9.6);
                df.stroke(#4000,1.);
                df.fill(mix(#3332,#555f,hover));
                df.new_path();
                df.rotate(down, c);
                df.move_to(c * hover_min);
                df.line_to(c + c * hover_max);
                df.move_to(vec2(c.x + c.x * hover_max, c.y * hover_min));
                df.line_to(vec2(c.x * hover_min, c.y + c.y * hover_max));
                return df.stroke(color, 1. + hover*0.2);
                //return df_fill(color);
            }"#
        ),
    ],
    ..Shader::DEFAULT
};

#[derive(Default)]
pub(crate) struct TabClose {
    component_id: ComponentId,
    bg_area: Area,
    animator: Animator,
}

const ANIM_DEFAULT: Anim = Anim {
    duration: 0.2,
    tracks: &[
        // TabCloseIns::color
        Track::Vec4 { key_frames: &[(1.0, vec4(0.62, 0.62, 0.62, 1.))], ease: Ease::DEFAULT },
        // TabCloseIns::hover
        Track::Float { key_frames: &[(1.0, 0.0)], ease: Ease::DEFAULT },
        // TabCloseIns::down
        Track::Float { key_frames: &[(1.0, 0.0)], ease: Ease::DEFAULT },
    ],
    ..Anim::DEFAULT
};

const ANIM_OVER: Anim = Anim {
    duration: 0.1,
    tracks: &[
        // TabCloseIns::color
        Track::Vec4 { key_frames: &[(0.0, Vec4::all(1.))], ease: Ease::DEFAULT },
        // TabCloseIns::hover
        Track::Float { key_frames: &[(1.0, 1.0)], ease: Ease::DEFAULT },
        // TabCloseIns::down
        Track::Float { key_frames: &[(1.0, 0.0)], ease: Ease::DEFAULT },
    ],
    ..Anim::DEFAULT
};

const ANIM_DOWN: Anim = Anim {
    duration: 0.2,
    tracks: &[
        // TabCloseIns::color
        Track::Vec4 { key_frames: &[(0.0, Vec4::all(1.))], ease: Ease::DEFAULT },
        // TabCloseIns::hover
        Track::Float { key_frames: &[(1.0, 1.0)], ease: Ease::DEFAULT },
        // TabCloseIns::down
        Track::Float { key_frames: &[(0.0, 0.0), (1.0, 0.0)], ease: Ease::DEFAULT },
    ],
    ..Anim::DEFAULT
};

impl TabClose {
    fn animate(&mut self, cx: &mut Cx) {
        let bg = self.bg_area.get_first_mut::<TabCloseIns>(cx);
        bg.color = self.animator.get_vec4(0);
        bg.hover = self.animator.get_float(1);
        bg.down = self.animator.get_float(2);
    }

    pub fn handle(&mut self, cx: &mut Cx, event: &mut Event) -> ButtonEvent {
        if self.animator.handle(cx, event) {
            self.animate(cx);
        }

        let rect = self.bg_area.get_rect_for_first_instance(cx).map(|rect| rect.add_padding(Padding::all(5.)));
        match event.hits_pointer(cx, self.component_id, rect) {
            Event::PointerDown(_pe) => {
                self.animator.play_anim(cx, ANIM_DOWN);
                cx.set_down_mouse_cursor(MouseCursor::Hand);
                return ButtonEvent::Down;
            }
            Event::PointerHover(pe) => {
                cx.set_hover_mouse_cursor(MouseCursor::Hand);
                match pe.hover_state {
                    HoverState::In => {
                        if pe.any_down {
                            self.animator.play_anim(cx, ANIM_DOWN)
                        } else {
                            self.animator.play_anim(cx, ANIM_OVER)
                        }
                    }
                    HoverState::Out => self.animator.play_anim(cx, ANIM_DEFAULT),
                    _ => (),
                }
            }
            Event::PointerUp(pe) => {
                if pe.is_over {
                    if pe.input_type.has_hovers() {
                        self.animator.play_anim(cx, ANIM_OVER)
                    } else {
                        self.animator.play_anim(cx, ANIM_DEFAULT)
                    }
                    return ButtonEvent::Clicked;
                } else {
                    self.animator.play_anim(cx, ANIM_DEFAULT);
                    return ButtonEvent::Up;
                }
            }
            _ => (),
        };
        ButtonEvent::None
    }

    pub fn draw(&mut self, cx: &mut Cx) {
        let rect = cx.add_box(LayoutSize::new(Width::Fix(25.0), Height::Fix(25.0)));

        self.bg_area = cx
            .add_instances(&SHADER, &[TabCloseIns { base: QuadIns::from_rect(rect).with_draw_depth(1.3), ..Default::default() }]);

        self.animator.draw(cx, ANIM_DEFAULT);
        self.animate(cx);
    }
}
