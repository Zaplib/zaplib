use zaplib::*;
use zaplib_components::*;

#[derive(Default)]
#[repr(C)]
struct ExampleQuad {
    base: Background,
}
impl ExampleQuad {
    fn draw(&mut self, cx: &mut Cx, label: &str) {
        cx.begin_padding_box(Padding::all(1.0));
        self.base.begin_draw(cx, Width::Compute, Height::Compute, vec4(0.8, 0.2, 0.4, 1.));
        cx.begin_padding_box(Padding::vh(12., 16.));
        TextIns::draw_walk(cx, label, &TextInsProps::DEFAULT);
        cx.end_padding_box();
        self.base.end_draw(cx);
        cx.end_padding_box();
    }
}

struct LayoutExampleApp {
    window: Window,
    pass: Pass,
    main_view: View,
    quad: ExampleQuad,
    token_input: TextInput,
    slider: FloatSlider,
    padding_value: f32,
}

impl LayoutExampleApp {
    fn new(_cx: &mut Cx) -> Self {
        Self {
            window: Window { create_inner_size: Some(Vec2 { x: 800., y: 600. }), ..Window::default() },
            pass: Pass::default(),
            quad: ExampleQuad::default(),
            main_view: View::default(),
            token_input: TextInput::new(TextInputOptions {
                empty_message: "Enter text".to_string(),
                ..TextInputOptions::default()
            }),
            slider: FloatSlider::default(),
            padding_value: 15.,
        }
    }

    fn handle(&mut self, cx: &mut Cx, event: &mut Event) {
        self.token_input.handle(cx, event);
        if let FloatSliderEvent::Change { scaled_value } = self.slider.handle(cx, event) {
            self.padding_value = scaled_value;
            cx.request_draw();
        }
    }

    fn draw_padding_slider(&mut self, cx: &mut Cx) {
        // This is the important non-trivial case of Compute box (padding_box) enclosing Fill one (slider)
        // It is tricky because Fill box doesn't have outer bounds (width/height) passed to it as
        // the outer box Compute is unbounded
        cx.begin_row(Width::Fill, Height::Compute);
        {
            cx.begin_padding_box(Padding::all(self.padding_value));
            self.quad.draw(cx, &format!("{:.0} padding", self.padding_value));
            cx.end_padding_box();

            cx.begin_padding_box(Padding::all(self.padding_value));
            self.slider.draw(cx, self.padding_value, 0.0, 30.0, Some(1.0), 1.0, None);
            cx.end_padding_box();
        }
        cx.end_row();
    }

    fn draw_alignment_examples(&mut self, cx: &mut Cx) {
        // This is the example of applying various alignment techniques
        {
            // First we cut the row with quads being on both side (left / right) and the middle one spanning the remaining
            cx.begin_row(Width::Fill, Height::Compute);
            {
                self.quad.draw(cx, "Row 1");
                self.quad.draw(cx, "Row 2");
                self.quad.draw(cx, "3");
                self.quad.draw(cx, "4");
            }
            {
                cx.begin_right_box();
                self.quad.draw(cx, "Row 5");
                self.quad.draw(cx, "Row 6");
                cx.end_right_box();
            }
            {
                cx.begin_center_x_align();
                self.quad.draw(cx, "Row mid");
                cx.end_center_x_align();
            }
            cx.end_row();
        }
        {
            // Cut fixed height row
            cx.begin_row(Width::Fill, Height::Fix(80.));
            self.quad.draw(cx, "Fixed Row Top");
            {
                cx.begin_column(Width::Compute, Height::Fill);
                cx.begin_center_y_align();
                self.quad.draw(cx, "Fixed Row Center");
                cx.end_center_y_align();
                cx.end_column();
            }
            {
                cx.begin_column(Width::Compute, Height::Fill);
                cx.begin_bottom_box();
                self.quad.draw(cx, "Fixed Row Bottom");
                cx.end_bottom_box();
                cx.end_column();
            }
            cx.end_row();
        }
        {
            // Now split the remaining space
            cx.begin_row(Width::Fill, Height::Fill);
            {
                // Cut the column aligned on the right
                cx.begin_right_box();
                cx.begin_column(Width::Compute, Height::Fill);
                {
                    self.quad.draw(cx, "Col 1");
                    self.quad.draw(cx, "some very long text");
                }
                {
                    cx.begin_center_y_align();
                    self.quad.draw(cx, "Col mid");
                    cx.end_center_y_align();
                }
                cx.end_column();
                cx.end_right_box();
            }
            {
                // Finally the remaining block has the quad centered by both x and y axis
                cx.begin_center_x_and_y_align();
                {
                    self.quad.draw(cx, "Mid 1");
                    self.token_input.draw(cx);
                    self.quad.draw(cx, "Mid 2");
                }
                cx.end_center_x_and_y_align();
            }
            cx.end_row();
        }
    }

    fn draw(&mut self, cx: &mut Cx) {
        self.window.begin_window(cx);
        self.pass.begin_pass(cx, Vec4::color("500"));
        self.main_view.begin_view(cx, LayoutSize::FILL);

        // cut fixed size top bar
        cx.begin_row(Width::Fill, Height::Fix(27.));
        cx.end_row();

        self.draw_padding_slider(cx);
        self.draw_alignment_examples(cx);

        self.main_view.end_view(cx);
        self.pass.end_pass(cx);
        self.window.end_window(cx);
    }
}

main_app!(LayoutExampleApp);
