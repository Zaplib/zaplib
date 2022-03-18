use std::time::Duration;

use zaplib::*;

#[derive(Clone, Copy)]
#[repr(C)]
struct ColorQuad {
    base: QuadIns,
    color: Vec3,
    fade: f32,
}

static SHADER: Shader = Shader {
    build_geom: Some(QuadIns::build_geom),
    code_to_concatenate: &[
        Cx::STD_SHADER,
        QuadIns::SHADER,
        code_fragment!(
            r#"
            instance color: vec3;
            instance fade: float;
            fn pixel() -> vec4 {
                return vec4(vec3(color) * fade, 1.);
            }"#
        ),
    ],
    ..Shader::DEFAULT
};

struct MultithreadExampleApp {
    window: Window,
    pass: Pass,
    main_view: View,
    quad_blue_fade: f32,
    quad_green_fade: f32,
    signal: Signal,
}

const FADE: StatusId = location_hash!();
const GREEN: StatusId = location_hash!();
const BLUE: StatusId = location_hash!();

const BLUE_DURATION: u64 = 3;
const GREEN_DURATION: u64 = 2;

impl MultithreadExampleApp {
    fn new(cx: &mut Cx) -> Self {
        Self {
            window: Window::default(),
            pass: Pass::default(),
            main_view: View::default(),
            quad_blue_fade: 1.0,
            quad_green_fade: 1.0,
            signal: cx.new_signal(),
        }
    }

    fn handle(&mut self, cx: &mut Cx, event: &mut Event) {
        if let Event::Signal(sig) = event {
            if let Some(statusses) = sig.signals.get(&self.signal) {
                for status in statusses {
                    if *status == BLUE {
                        self.quad_blue_fade = 1.;
                        let signal = self.signal;
                        universal_thread::spawn(move || {
                            universal_thread::sleep(Duration::from_secs(BLUE_DURATION));
                            Cx::post_signal(signal, BLUE);
                        });
                    }
                    if *status == GREEN {
                        self.quad_green_fade = 1.;
                        let signal = self.signal;
                        universal_thread::spawn(move || {
                            universal_thread::sleep(Duration::from_secs(GREEN_DURATION));
                            Cx::post_signal(signal, GREEN);
                        });
                    }
                    if *status == FADE {
                        self.quad_blue_fade -= 1. / (BLUE_DURATION as f32 * 10.);
                        self.quad_green_fade -= 1. / (GREEN_DURATION as f32 * 10.);

                        let signal = self.signal;
                        universal_thread::spawn(move || {
                            universal_thread::sleep(Duration::from_millis(100));
                            Cx::post_signal(signal, FADE);
                        })
                    }
                }
            }
            cx.request_draw();
        }

        match event {
            Event::Construct => {
                let signal = self.signal;

                universal_thread::spawn(move || {
                    universal_thread::sleep(Duration::from_millis(100));
                    Cx::post_signal(signal, FADE);
                });
                universal_thread::spawn(move || {
                    universal_thread::sleep(Duration::from_millis(1000));
                    Cx::post_signal(signal, GREEN);
                });
                universal_thread::spawn(move || {
                    universal_thread::sleep(Duration::from_millis(1500));
                    Cx::post_signal(signal, BLUE);
                });
            }
            _ => (),
        }
    }

    fn draw(&mut self, cx: &mut Cx) {
        self.window.begin_window(cx);
        self.pass.begin_pass(cx, Vec4::color("300"));
        self.main_view.begin_view(cx, LayoutSize::FILL);
        cx.add_instances(
            &SHADER,
            &[ColorQuad {
                base: QuadIns::from_rect(Rect { pos: vec2(100., 100.), size: vec2(100., 100.0) }),
                color: Vec3 { x: 0., y: 0., z: 1. },
                fade: self.quad_blue_fade,
            }],
        );
        cx.add_instances(
            &SHADER,
            &[ColorQuad {
                base: QuadIns::from_rect(Rect { pos: vec2(100., 300.), size: vec2(100., 100.0) }),
                color: Vec3 { x: 0., y: 1., z: 0. },
                fade: self.quad_green_fade,
            }],
        );
        self.main_view.end_view(cx);
        self.pass.end_pass(cx);
        self.window.end_window(cx);
    }
}

main_app!(MultithreadExampleApp);
