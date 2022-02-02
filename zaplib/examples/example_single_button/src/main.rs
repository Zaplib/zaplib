use zaplib::*;

mod single_button;
use single_button::*;

#[derive(Default)]
struct SingleButtonExampleApp {
    window: Window,
    pass: Pass,
    main_view: View,
    single_button: SingleButton,
}

impl SingleButtonExampleApp {
    fn new(_cx: &mut Cx) -> Self {
        Self::default()
    }

    fn handle(&mut self, cx: &mut Cx, event: &mut Event) {
        self.single_button.handle(cx, event);
    }

    fn draw(&mut self, cx: &mut Cx) {
        self.window.begin_window(cx);
        self.pass.begin_pass(cx, Vec4::color("300"));
        self.main_view.begin_view(cx, LayoutSize::FILL);
        cx.begin_padding_box(Padding::top(30.));

        self.single_button.draw(cx);

        cx.end_padding_box();
        self.main_view.end_view(cx);
        self.pass.end_pass(cx);
        self.window.end_window(cx);
    }
}

main_app!(SingleButtonExampleApp);
