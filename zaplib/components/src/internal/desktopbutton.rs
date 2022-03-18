use crate::button::*;
use crate::buttonlogic::*;
use zaplib::*;

pub enum DesktopButtonType {
    WindowsMin,
    WindowsMax,
    WindowsMaxToggled,
    WindowsClose,
    XRMode,
    Fullscreen,
}

impl DesktopButtonType {
    fn shader_float(&self) -> f32 {
        match self {
            DesktopButtonType::WindowsMin => 1.,
            DesktopButtonType::WindowsMax => 2.,
            DesktopButtonType::WindowsMaxToggled => 3.,
            DesktopButtonType::WindowsClose => 4.,
            DesktopButtonType::XRMode => 5.,
            DesktopButtonType::Fullscreen => 6.,
        }
    }
}

#[derive(Clone, Copy, Default)]
#[repr(C)]
struct DesktopButtonIns {
    base: QuadIns,
    hover: f32,
    down: f32,
    button_type: f32,
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
            instance button_type: float;

            fn pixel() -> vec4 {
                let df = Df::viewport(pos * rect_size);
                df.aa *= 3.0;
                let sz = 4.5;
                let c = rect_size * vec2(0.5, 0.5);
                // WindowsMin
                if abs(button_type - 1.) < 0.1 {
                    df.clear(mix(#3, mix(#6, #9, down), hover));
                    df.move_to(c - vec2(sz, 0.));
                    df.line_to(c + vec2(sz, 0.));
                    df.stroke(#f, 0.5 + 0.5 * dpi_dilate);
                    return df.result;
                }
                // WindowsMax
                if abs(button_type - 2.) < 0.1 {
                    df.clear(mix(#3, mix(#6, #9, down), hover));
                    df.rect(c - sz, vec2(2. * sz));
                    df.stroke(#f, 0.5 + 0.5 * dpi_dilate);
                    return df.result;
                }
                // WindowsMaxToggled
                if abs(button_type - 3.) < 0.1 {
                    let clear = mix(#3, mix(#6, #9, down), hover);
                    df.clear(clear);
                    let sz = 3.5;
                    df.rect(c - sz + vec2(1., -1.), vec2(2. * sz));
                    df.stroke(#f, 0.5 + 0.5 * dpi_dilate);
                    df.new_path();
                    df.rect(c - sz + vec2(-1., 1.), vec2(2. * sz));
                    df.fill(clear);
                    df.stroke(#f, 0.5 + 0.5 * dpi_dilate);

                    return df.result;
                }
                // WindowsClose
                if abs(button_type - 4.) < 0.1 {
                    df.clear(mix(#3, mix(#e00, #c00, down), hover));
                    df.move_to(c - sz);
                    df.line_to(c + sz);
                    df.move_to(c + vec2(-sz, sz));
                    df.line_to(c + vec2(sz, -sz));
                    df.stroke(#f, 0.5 + 0.5 * dpi_dilate);
                    return df.result;
                }
                // VRMode
                if abs(button_type - 5.) < 0.1 {
                    df.clear(mix(#3, mix(#0aa, #077, down), hover));
                    let d = vec2(12., 8.);
                    df.box(c - d, 2. * d, 2.);
                    // subtract 2 eyes
                    df.circle(vec2(c.x - 5.5, c.y), 3.5);
                    df.subtract();
                    df.circle(vec2(c.x + 5.5, c.y), 3.5);
                    df.subtract();
                    df.circle(vec2(c.x, c.y + d.y - 0.75), 2.5);
                    df.subtract();
                    df.fill(#8);

                    return df.result;
                }
                // Fullscreen
                if abs(button_type - 6.) < 0.1 {
                    sz = 8.;
                    df.clear(mix(#3, mix(#6, #9, down), hover));
                    df.rect(c - sz, vec2(2. * sz));
                    df.rect(c - sz + 1.5, vec2(2. * (sz - 1.5)));
                    df.subtract();
                    df.rect(c - sz + vec2(4., -2.), 2. * (sz + vec2(-4., 2.)));
                    df.subtract();
                    df.rect(c - sz + vec2(-2., 4.), 2. * (sz + vec2(2., -4.)));
                    df.subtract();
                    df.fill(#f); //, 0.5 + 0.5 * dpi_dilate);

                    return df.result;
                }

                return #f00;
            }"#
        ),
    ],
    ..Shader::DEFAULT
};

#[derive(Default)]
pub(crate) struct DesktopButton {
    component_id: ComponentId,
    bg_area: Area,
    animator: Animator,
}

const ANIM_DEFAULT: Anim = Anim {
    duration: 0.2,
    tracks: &[
        // DesktopButtonIns::hover
        Track::Float { key_frames: &[(1.0, 0.0)], ease: Ease::DEFAULT },
        // DesktopButtonIns::down
        Track::Float { key_frames: &[(1.0, 0.0)], ease: Ease::DEFAULT },
    ],
    ..Anim::DEFAULT
};

const ANIM_HOVER: Anim = Anim {
    duration: 0.2,
    tracks: &[
        // DesktopButtonIns::hover
        Track::Float { key_frames: &[(1.0, 1.0)], ease: Ease::DEFAULT },
        // DesktopButtonIns::down
        Track::Float { key_frames: &[(0.0, 0.0)], ease: Ease::DEFAULT },
    ],
    ..Anim::DEFAULT
};

const ANIM_DOWN: Anim = Anim {
    duration: 0.2,
    tracks: &[
        // DesktopButtonIns::hover
        Track::Float { key_frames: &[(0.0, 1.0)], ease: Ease::DEFAULT },
        // DesktopButtonIns::down
        Track::Float { key_frames: &[(1.0, 1.0)], ease: Ease::DEFAULT },
    ],
    ..Anim::DEFAULT
};

impl DesktopButton {
    fn animate(&mut self, cx: &mut Cx) {
        let bg = self.bg_area.get_first_mut::<DesktopButtonIns>(cx);
        bg.hover = self.animator.get_float(0);
        bg.down = self.animator.get_float(1);
    }

    pub fn handle(&mut self, cx: &mut Cx, event: &mut Event) -> ButtonEvent {
        if self.animator.handle(cx, event) {
            self.animate(cx);
        }
        let animator = &mut self.animator;
        let hit_event = event.hits_pointer(cx, self.component_id, self.bg_area.get_rect_for_first_instance(cx));
        handle_button_logic(cx, hit_event, |cx, logic_event| match logic_event {
            ButtonLogicEvent::Down => animator.play_anim(cx, ANIM_DOWN),
            ButtonLogicEvent::Default => animator.play_anim(cx, ANIM_DEFAULT),
            ButtonLogicEvent::Over => animator.play_anim(cx, ANIM_HOVER),
        })
    }

    pub fn draw(&mut self, cx: &mut Cx, ty: DesktopButtonType) {
        let (w, h) = match ty {
            DesktopButtonType::WindowsMin
            | DesktopButtonType::WindowsMax
            | DesktopButtonType::WindowsMaxToggled
            | DesktopButtonType::WindowsClose => (46., 29.),
            DesktopButtonType::XRMode => (50., 36.),
            DesktopButtonType::Fullscreen => (50., 36.),
        };

        let rect = cx.add_box(LayoutSize::new(Width::Fix(w), Height::Fix(h)));

        self.bg_area = cx.add_instances(
            &SHADER,
            &[DesktopButtonIns { base: QuadIns::from_rect(rect), button_type: ty.shader_float(), ..Default::default() }],
        );

        self.animator.draw(cx, ANIM_DEFAULT);
        self.animate(cx);
    }
}
