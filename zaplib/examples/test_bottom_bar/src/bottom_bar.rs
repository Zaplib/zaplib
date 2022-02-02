use zaplib::*;
use zaplib_components::*;

pub struct BottomBar {
    switch_button: Button,
    token_input: TextInput,
    play_speed_button: Button,
    slider: FloatSlider,

    play_speed: String,
    position: f32,
}

impl BottomBar {
    pub fn new() -> Self {
        Self {
            switch_button: Button::default(),
            token_input: TextInput::new(TextInputOptions {
                empty_message: "Enter token".to_string(),
                ..TextInputOptions::default()
            }),
            play_speed_button: Button::default(),
            slider: FloatSlider::default(),
            play_speed: String::from("1x"),
            position: 0.5,
        }
    }

    pub fn handle(&mut self, cx: &mut Cx, event: &mut Event) {
        if let ButtonEvent::Clicked = self.switch_button.handle(cx, event) {
            log!("button clicked!");
        }
        if let ButtonEvent::Clicked = self.play_speed_button.handle(cx, event) {
            if self.play_speed == "1x" {
                self.play_speed = String::from("Fixed: 33ms");
            } else {
                self.play_speed = String::from("1x");
            }
            cx.request_draw();
        }
        if let FloatSliderEvent::Change { scaled_value } = self.slider.handle(cx, event) {
            self.position = scaled_value;
            cx.request_draw();
        }

        self.token_input.handle(cx, event);
    }

    pub fn draw(&mut self, cx: &mut Cx) {
        cx.begin_column(Width::Fill, Height::Fill); // outer_box
        {
            cx.begin_bottom_box();
            cx.begin_row(Width::Fill, Height::Compute); // bottom bar itself
            {
                self.switch_button.draw(cx, "*");

                self.token_input.draw(cx);

                cx.begin_right_box();
                self.play_speed_button.draw(cx, &self.play_speed);
                cx.end_right_box();

                self.slider.draw(cx, self.position, 0.0, 1.0, None, 1.0, None);
            }
            cx.end_row(); // bottom bar itself
            cx.end_bottom_box();
        }
        cx.end_column(); // outer_box;
    }
}
