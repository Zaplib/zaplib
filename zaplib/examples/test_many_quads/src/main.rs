use zaplib::*;

static SHADER: Shader = Shader {
    build_geom: Some(QuadIns::build_geom),
    code_to_concatenate: &[
        Cx::STD_SHADER,
        QuadIns::SHADER,
        code_fragment!(
            r#"
            instance counter: float;
            fn pixel() -> vec4 {
                return mix(#f00, #0f0, abs(sin(counter)));
            }"#
        ),
    ],
    ..Shader::DEFAULT
};

#[derive(Clone, Copy)]
#[repr(C)]
struct CounterQuad {
    quad: QuadIns,
    counter: f32,
}

#[derive(Default)]
struct BareExampleApp {
    window: Window,
    pass: Pass,
    main_view: View,
    count: f32,
}

impl BareExampleApp {
    fn new(_cx: &mut Cx) -> Self {
        Self::default()
    }

    fn handle(&mut self, _cx: &mut Cx, event: &mut Event) {
        match event {
            Event::Construct => {}
            Event::PointerMove(pm) => {
                self.count = pm.abs.x * 0.01;
            }
            _ => (),
        }
    }

    fn draw(&mut self, cx: &mut Cx) {
        self.window.begin_window(cx);
        self.pass.begin_pass(cx, Vec4::color("300"));
        self.main_view.begin_view(cx, LayoutSize::FILL);
        cx.profile_start(1);
        let data: Vec<CounterQuad> = (0..1000000)
            .map(|i| {
                let v = 0.3 * (i as f32);
                let x = 400. + (v + self.count).sin() * 400.;
                let y = 400. + (v * 1.12 + self.count * 18.).cos() * 400.;
                CounterQuad {
                    quad: QuadIns::from_rect(Rect { pos: vec2(x, y), size: vec2(10., 10.0) }),
                    counter: (i as f32).sin(),
                }
            })
            .collect();
        cx.add_instances(&SHADER, &data);
        self.count += 0.001;

        self.main_view.end_view(cx);
        cx.profile_end(1);

        self.pass.end_pass(cx);
        self.window.end_window(cx);
        cx.request_draw();
    }
}

main_app!(BareExampleApp);
