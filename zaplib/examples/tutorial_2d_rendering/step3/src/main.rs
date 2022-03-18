use zaplib::*;

#[derive(Clone, Copy)]
#[repr(C)]
struct RectIns {
    quad: QuadIns,
    color: Vec4,
}

static SHADER: Shader = Shader {
    build_geom: Some(QuadIns::build_geom),
    code_to_concatenate: &[
        Cx::STD_SHADER,
        QuadIns::SHADER,
        code_fragment!(
            r#"
            instance color: vec4;

            fn pixel() -> vec4 {
                let border = 10.;
                let pt = pos * rect_size;
                if pt.x < border || pt.y < border || pt.x > rect_size.x - border || pt.y > rect_size.y - border {
                    return vec4(1., 1., 0., 1.0);
                }
                return color;
            }
            "#
        ),
    ],
    ..Shader::DEFAULT
};

#[derive(Default)]
struct App {
    window: Window,
    pass: Pass,
    view: View,
}

impl App {
    fn new(_cx: &mut Cx) -> Self {
        Self::default()
    }

    fn handle(&mut self, _cx: &mut Cx, _event: &mut Event) {}

    fn draw(&mut self, cx: &mut Cx) {
        self.window.begin_window(cx);
        self.pass.begin_pass(cx, Vec4::color("0"));
        self.view.begin_view(cx, LayoutSize::FILL);

        let rect1 = RectIns {
            quad: QuadIns { rect_pos: vec2(50., 50.), rect_size: vec2(400., 200.), draw_depth: 0. },
            color: vec4(1., 0., 0., 1.),
        };
        let rect2 = RectIns {
            quad: QuadIns { rect_pos: vec2(100., 100.), rect_size: vec2(200., 400.), draw_depth: 0. },
            color: vec4(0., 0., 1., 1.),
        };

        cx.add_instances(&SHADER, &[rect1, rect2]);

        self.view.end_view(cx);
        self.pass.end_pass(cx);
        self.window.end_window(cx);
    }
}

main_app!(App);
