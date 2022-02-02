use zaplib::*;
use zaplib_components::*;

#[derive(Default)]
struct App {
    window: Window,
    pass: Pass,
    view: View,
    button: Button,
    counter: i32,
    slider: FloatSlider,
    scroll_view: ScrollView,
}

impl App {
    fn new(_cx: &mut Cx) -> Self {
        App { scroll_view: ScrollView::new_standard_vh(), ..Self::default() }
    }

    fn handle(&mut self, cx: &mut Cx, event: &mut Event) {
        if let ButtonEvent::Clicked = self.button.handle(cx, event) {
            self.counter += 1;
            cx.request_draw();
        }

        if let FloatSliderEvent::Change { scaled_value } = self.slider.handle(cx, event) {
            self.counter = scaled_value as i32;
            cx.request_draw();
        }

        self.scroll_view.handle(cx, event);
    }

    fn draw(&mut self, cx: &mut Cx) {
        self.window.begin_window(cx);
        self.pass.begin_pass(cx, Vec4::color("0"));
        self.view.begin_view(cx, LayoutSize::FILL);

        cx.begin_padding_box(Padding::vh(50., 50.));
        self.button.draw(cx, "Increment Counter");
        TextIns::draw_walk(cx, &format!("Counter: {}", self.counter), &TextInsProps::default());

        self.slider.draw(cx, self.counter as f32, 0., 100., Some(1.0), 1.0, None);

        self.scroll_view.begin_view(cx, LayoutSize::FILL);
        for value in 0..self.counter {
            TextIns::draw_walk(cx, &format!("row #{}", value), &TextInsProps::default());
        }
        self.scroll_view.end_view(cx);

        cx.end_padding_box();

        self.view.end_view(cx);
        self.pass.end_pass(cx);
        self.window.end_window(cx);
    }
}

main_app!(App);
