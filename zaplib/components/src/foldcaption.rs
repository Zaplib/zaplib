use crate::button::*;
use crate::buttonlogic::*;
use zaplib::*;

#[derive(Clone)]
pub enum FoldOpenState {
    Open,
    Opening(f32),
    Closed,
    Closing(f32),
}

impl Default for FoldOpenState {
    fn default() -> Self {
        FoldOpenState::Open
    }
}

impl FoldOpenState {
    fn get_value(&self) -> f32 {
        match self {
            FoldOpenState::Opening(fac) => 1.0 - *fac,
            FoldOpenState::Closing(fac) => *fac,
            FoldOpenState::Open => 1.0,
            FoldOpenState::Closed => 0.0,
        }
    }
    pub fn is_open(&self) -> bool {
        match self {
            FoldOpenState::Opening(_) => true,
            FoldOpenState::Closing(_) => false,
            FoldOpenState::Open => true,
            FoldOpenState::Closed => false,
        }
    }
    pub fn toggle(&mut self) {
        *self = match self {
            FoldOpenState::Opening(fac) => FoldOpenState::Closing(1.0 - *fac),
            FoldOpenState::Closing(fac) => FoldOpenState::Opening(1.0 - *fac),
            FoldOpenState::Open => FoldOpenState::Closing(1.0),
            FoldOpenState::Closed => FoldOpenState::Opening(1.0),
        };
    }
    pub fn do_open(&mut self) {
        *self = match self {
            FoldOpenState::Opening(fac) => FoldOpenState::Opening(*fac),
            FoldOpenState::Closing(fac) => FoldOpenState::Opening(1.0 - *fac),
            FoldOpenState::Open => FoldOpenState::Open,
            FoldOpenState::Closed => FoldOpenState::Opening(1.0),
        };
    }
    pub fn do_close(&mut self) {
        *self = match self {
            FoldOpenState::Opening(fac) => FoldOpenState::Closing(1.0 - *fac),
            FoldOpenState::Closing(fac) => FoldOpenState::Closing(*fac),
            FoldOpenState::Open => FoldOpenState::Closing(1.0),
            FoldOpenState::Closed => FoldOpenState::Closed,
        };
    }
    pub fn do_time_step(&mut self, mul: f32) -> bool {
        let mut redraw = false;
        *self = match self {
            FoldOpenState::Opening(fac) => {
                redraw = true;
                if *fac < 0.001 {
                    FoldOpenState::Open
                } else {
                    FoldOpenState::Opening(*fac * mul)
                }
            }
            FoldOpenState::Closing(fac) => {
                redraw = true;
                if *fac < 0.001 {
                    FoldOpenState::Closed
                } else {
                    FoldOpenState::Closing(*fac * mul)
                }
            }
            FoldOpenState::Open => FoldOpenState::Open,
            FoldOpenState::Closed => FoldOpenState::Closed,
        };
        redraw
    }
}

#[derive(Clone, Copy, Default)]
#[repr(C)]
struct FoldCaptionIns {
    base: QuadIns,
    hover: f32,
    down: f32,
    open: f32,
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
            instance open: float;

            const shadow: float = 3.0;
            const border_radius: float = 2.5;

            fn pixel() -> vec4 {
                let sz = 3.;
                let c = vec2(5.0,0.5*rect_size.y);
                let df = Df::viewport(pos * rect_size);
                df.clear(#2);
                // we have 3 points, and need to rotate around its center
                df.rotate(open*0.5*PI+0.5*PI, c);
                df.move_to(c + vec2(-sz, sz));
                df.line_to(c + vec2(0, -sz));
                df.line_to(c + sz);
                df.close_path();
                df.fill(mix(#a,#f,hover));

                return df.result;
            }"#
        ),
    ],
    ..Shader::DEFAULT
};

#[derive(Default)]
pub struct FoldCaption {
    component_id: ComponentId,
    bg_area: Area,
    text_area: Area,
    animator: Animator,
    open_state: FoldOpenState,
}

const ANIM_DEFAULT: Anim = Anim {
    duration: 0.1,
    tracks: &[
        // FoldCaptionIns::hover
        Track::Float { key_frames: &[(1.0, 0.0)], ease: Ease::DEFAULT },
        // FoldCaptionIns::down
        Track::Float { key_frames: &[(1.0, 0.0)], ease: Ease::DEFAULT },
        // TextIns::color
        Track::Vec4 { key_frames: &[(1.0, vec4(0.6, 0.6, 0.6, 1.0))], ease: Ease::DEFAULT },
    ],
    ..Anim::DEFAULT
};

const ANIM_OVER: Anim = Anim {
    duration: 0.1,
    tracks: &[
        // FoldCaptionIns::hover
        Track::Float { key_frames: &[(0.0, 1.0), (1.0, 1.0)], ease: Ease::DEFAULT },
        // FoldCaptionIns::down
        Track::Float { key_frames: &[(1.0, 0.0)], ease: Ease::DEFAULT },
        // TextIns::color
        Track::Vec4 { key_frames: &[(0.0, Vec4::all(1.))], ease: Ease::DEFAULT },
    ],
    ..Anim::DEFAULT
};

const ANIM_DOWN: Anim = Anim {
    duration: 0.2,
    tracks: &[
        // FoldCaptionIns::hover
        Track::Float { key_frames: &[(0.0, 1.0), (1.0, 1.0)], ease: Ease::DEFAULT },
        // FoldCaptionIns::down
        Track::Float { key_frames: &[(1.0, 1.0)], ease: Ease::DEFAULT },
        // TextIns::color
        Track::Vec4 { key_frames: &[(0.0, vec4(0.8, 0.8, 0.8, 1.0))], ease: Ease::DEFAULT },
    ],
    ..Anim::DEFAULT
};

impl FoldCaption {
    fn animate(&mut self, cx: &mut Cx) {
        let bg = self.bg_area.get_first_mut::<FoldCaptionIns>(cx);
        bg.hover = self.animator.get_float(0);
        bg.down = self.animator.get_float(1);
        TextIns::set_color(cx, self.text_area, self.animator.get_vec4(2));
    }

    pub fn handle_fold_caption(&mut self, cx: &mut Cx, event: &mut Event) -> ButtonEvent {
        if self.animator.handle(cx, event) {
            self.animate(cx);
        }

        let animator = &mut self.animator;
        let open_state = &mut self.open_state;
        let hit_event = event.hits_pointer(cx, self.component_id, self.bg_area.get_rect_for_first_instance(cx));
        handle_button_logic(cx, hit_event, |cx, logic_event| match logic_event {
            ButtonLogicEvent::Down => {
                // lets toggle our anim state
                open_state.toggle();
                cx.request_draw();
                animator.play_anim(cx, ANIM_DOWN);
            }
            ButtonLogicEvent::Default => animator.play_anim(cx, ANIM_DEFAULT),
            ButtonLogicEvent::Over => animator.play_anim(cx, ANIM_OVER),
        })
    }

    pub fn draw_fold_caption(&mut self, cx: &mut Cx, label: &str) -> f32 {
        let open_value = self.open_state.get_value();

        self.bg_area = cx.add_instances(&SHADER, &[FoldCaptionIns { open: open_value, ..Default::default() }]);

        cx.begin_padding_box(Padding::all(1.0));
        cx.begin_row(Width::Fill, Height::Compute); // fold
        cx.begin_padding_box(Padding::vh(8., 14.)); // fold content
        {
            if self.open_state.do_time_step(0.6) {
                cx.request_draw();
            }
            cx.begin_right_box();
            let draw_str_props =
                TextInsProps { wrapping: Wrapping::Ellipsis(cx.get_width_left() - 10.), ..TextInsProps::DEFAULT };
            TextIns::draw_walk(cx, label, &draw_str_props);
            cx.end_right_box();
        }
        cx.end_padding_box(); // fold content
        let rect = cx.end_row(); // fold
        cx.end_padding_box();

        let bg = self.bg_area.get_first_mut::<FoldCaptionIns>(cx);
        bg.base = QuadIns::from_rect(rect);

        self.animator.draw(cx, ANIM_DEFAULT);
        self.animate(cx);

        open_value
    }
}
