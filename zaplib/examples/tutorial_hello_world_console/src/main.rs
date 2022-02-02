use zaplib::*;

#[derive(Default)]
struct App {}

impl App {
    fn new(_cx: &mut Cx) -> Self {
        Self::default()
    }

    fn handle(&mut self, _cx: &mut Cx, event: &mut Event) {
        if let Event::Construct = event {
            log!("Hello, world!");
        }
    }
    fn draw(&mut self, _cx: &mut Cx) {}
}

main_app!(App);
