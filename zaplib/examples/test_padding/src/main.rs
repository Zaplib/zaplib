use zaplib::*;
use zaplib_components::*;

const COLOR_INNER: Vec4 = vec4(0.8, 0.2, 0.4, 1.);

#[derive(Default)]
#[repr(C)]
struct ExampleQuad {
    base: Background,
}
impl ExampleQuad {
    fn begin_fill(&mut self, cx: &mut Cx, label: &str, color: Vec4) {
        self.base.begin_draw(cx, Width::Fill, Height::Fill, color);
        TextIns::draw_walk(cx, label, &TextInsProps::DEFAULT);
    }

    fn end_fill(&mut self, cx: &mut Cx) {
        self.base.end_draw(cx);
    }
}

struct PaddingExampleApp {
    window: Window,
    pass: Pass,
    main_view: View,
    quad: ExampleQuad,
}

impl PaddingExampleApp {
    fn new(_cx: &mut Cx) -> Self {
        Self {
            window: Window { create_inner_size: Some(Vec2 { x: 800., y: 600. }), ..Window::default() },
            pass: Pass::default(),
            quad: ExampleQuad::default(),
            main_view: View::default(),
        }
    }

    fn handle(&mut self, _cx: &mut Cx, _event: &mut Event) {}

    fn draw(&mut self, cx: &mut Cx) {
        self.window.begin_window(cx);
        self.pass.begin_pass(cx, Vec4::color("500"));
        self.main_view.begin_view(cx, LayoutSize::FILL);

        // cut fixed size top bar
        cx.begin_row(Width::Fill, Height::Fix(27.));
        cx.end_row();

        cx.begin_row(Width::Fill, Height::Fill);
        {
            cx.begin_padding_box(Padding::all(30.));
            self.quad.begin_fill(cx, "inner", COLOR_INNER);
            self.quad.end_fill(cx);
            cx.end_padding_box();
        }
        cx.end_row();

        self.main_view.end_view(cx);
        self.pass.end_pass(cx);
        self.window.end_window(cx);
    }
}

main_app!(PaddingExampleApp);
