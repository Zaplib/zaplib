use zaplib::*;

#[derive(Clone, Copy)]
#[repr(C)]
struct RectIns {
    color: Vec4,
    rect_pos: Vec2,
    rect_size: Vec2,
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
            instance rect_pos: vec2;
            instance rect_size: vec2;
            varying pos: vec2;

            fn vertex() -> vec4 {
                let point = geom * rect_size + rect_pos;
                pos = (point - rect_pos) / rect_size;
                return camera_projection * camera_view * vec4(point.x, point.y, 0., 1.);
            }

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

        let rect1 = RectIns { color: vec4(1., 0., 0., 1.), rect_pos: vec2(50., 50.), rect_size: vec2(400., 200.) };
        let rect2 = RectIns { color: vec4(0., 0., 1., 1.), rect_pos: vec2(100., 100.), rect_size: vec2(200., 400.) };

        cx.add_instances(&SHADER, &[rect1, rect2]);

        self.view.end_view(cx);
        self.pass.end_pass(cx);
        self.window.end_window(cx);
    }
}

main_app!(App);
