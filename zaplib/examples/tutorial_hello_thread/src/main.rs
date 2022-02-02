use std::io::Read;

use zaplib::*;

#[derive(Default)]
struct App {
    window: Window,
}

impl App {
    fn new(_cx: &mut Cx) -> Self {
        Self { window: Window { create_add_drop_target_for_app_open_files: true, ..Window::default() } }
    }

    fn handle(&mut self, _cx: &mut Cx, event: &mut Event) {
        if let Event::AppOpenFiles(aof) = event {
            // Get a copy of the file handle for use in the thread.
            let mut file = aof.user_files[0].file.clone();

            universal_thread::spawn(move || {
                let mut contents = String::new();
                file.read_to_string(&mut contents).unwrap();
                log!("Contents of dropped file: {contents}");
            });
        }
    }

    fn draw(&mut self, cx: &mut Cx) {
        self.window.begin_window(cx);
        self.window.end_window(cx);
    }
}

main_app!(App);
