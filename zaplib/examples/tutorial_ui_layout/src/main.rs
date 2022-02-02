use zaplib::*;
use zaplib_components::*;

#[derive(Default)]
struct App {
    window: Window,
    pass: Pass,
    view: View,
    button_inc: Button,
    button_dec: Button,
    counter: i32,
    slider: FloatSlider,
    scroll_view: ScrollView,
    splitter: Splitter,
}

impl App {
    fn new(_cx: &mut Cx) -> Self {
        let mut splitter = Splitter::default();
        splitter.set_splitter_state(SplitterAlign::First, 300., Axis::Vertical);
        App { scroll_view: ScrollView::new_standard_vh(), splitter, ..Self::default() }
    }

    fn handle(&mut self, cx: &mut Cx, event: &mut Event) {
        if let ButtonEvent::Clicked = self.button_inc.handle(cx, event) {
            self.counter += 1;
            cx.request_draw();
        }
        if let ButtonEvent::Clicked = self.button_dec.handle(cx, event) {
            self.counter -= 1;
            cx.request_draw();
        }

        if let FloatSliderEvent::Change { scaled_value } = self.slider.handle(cx, event) {
            self.counter = scaled_value as i32;
            cx.request_draw();
        }

        self.scroll_view.handle(cx, event);
        self.splitter.handle(cx, event);
    }

    fn draw(&mut self, cx: &mut Cx) {
        self.window.begin_window(cx);
        self.pass.begin_pass(cx, Vec4::color("0"));
        self.view.begin_view(cx, LayoutSize::FILL);

        cx.begin_padding_box(Padding::vh(50., 50.));

        cx.begin_bottom_box();
        cx.begin_row(Width::Fill, Height::Compute);
        {
            self.button_dec.draw(cx, "Decrement");
            cx.begin_right_box();
            self.button_inc.draw(cx, "Increment");
            cx.end_right_box();
            self.slider.draw(cx, self.counter as f32, 0., 100., Some(1.0), 1.0, None);
        }
        cx.end_row();
        cx.end_bottom_box();

        cx.begin_row(Width::Fill, Height::Fill);
        {
            self.splitter.begin_draw(cx);
            {
                self.scroll_view.begin_view(cx, LayoutSize::FILL);
                cx.begin_column(Width::Compute, Height::Compute);

                for value in 0..self.counter {
                    TextIns::draw_walk(cx, &format!("row #{}", value), &TextInsProps::default());
                }
                cx.end_column();
                self.scroll_view.end_view(cx);
            }
            self.splitter.mid_draw(cx);
            {
                TextIns::draw_walk(cx, &format!("Counter: {}", self.counter), &TextInsProps::default());
            }
            self.splitter.end_draw(cx);
        }
        cx.end_row();

        cx.end_padding_box();

        self.view.end_view(cx);
        self.pass.end_pass(cx);
        self.window.end_window(cx);
    }
}

main_app!(App);
