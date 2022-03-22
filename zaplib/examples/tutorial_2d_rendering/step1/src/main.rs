use zaplib::*;

#[derive(Clone, Copy)]
#[repr(C)]
struct RectIns {
    color: Vec4,
}

fn build_geom() -> Geometry {
    let vertex_attributes = vec![vec2(0., 0.), vec2(1., 0.), vec2(1., 1.), vec2(0., 1.)];
    let indices = vec![[0, 1, 2], [2, 3, 0]];
    Geometry::new(vertex_attributes, indices)
}

static SHADER: Shader = Shader {
    build_geom: Some(build_geom),
    code_to_concatenate: &[
        Cx::STD_SHADER,
        code_fragment!(
            r#"
            geometry geom: vec2;
            instance color: vec4;

            fn vertex() -> vec4 {
                return vec4(geom.x, geom.y, 0., 1.);
            }

            fn pixel() -> vec4 {
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

        let color = vec4(1., 0., 0., 1.);
        cx.add_instances(&SHADER, &[RectIns { color }]);

        self.view.end_view(cx);
        self.pass.end_pass(cx);
        self.window.end_window(cx);
    }
}

main_app!(App);
