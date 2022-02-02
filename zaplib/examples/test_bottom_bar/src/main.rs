use zaplib::*;

mod bottom_bar;
use bottom_bar::*;

struct BottomBarExampleApp {
    window: Window,
    pass: Pass,
    main_view: View,
    bottom_bar: BottomBar,
}

impl BottomBarExampleApp {
    fn new(_cx: &mut Cx) -> Self {
        Self {
            window: Window { create_inner_size: Some(Vec2 { x: 700., y: 400. }), ..Window::default() },
            pass: Pass::default(),
            bottom_bar: BottomBar::new(),
            main_view: View::default(),
        }
    }

    fn handle(&mut self, cx: &mut Cx, event: &mut Event) {
        self.bottom_bar.handle(cx, event);
    }

    fn draw(&mut self, cx: &mut Cx) {
        self.window.begin_window(cx);
        self.pass.begin_pass(cx, Vec4::color("333"));
        self.main_view.begin_view(cx, LayoutSize::FILL);

        self.bottom_bar.draw(cx);

        self.main_view.end_view(cx);
        self.pass.end_pass(cx);
        self.window.end_window(cx);
    }
}

main_app!(BottomBarExampleApp);
