use zaplib::*;

mod flamerect;
use flamerect::*;

mod flamedata;

use flamedata::*;

#[derive(Default)]
struct FlamegraphExampleApp {
    window: Window,
    pass: Pass,
    main_view: View,
    flame_rects: Vec<FlameRect>,
}

pub struct Span {
    pub rect: Rect,
    pub label: String,
    pub color: Vec4,
}

impl FlamegraphExampleApp {
    fn new(_cx: &mut Cx) -> Self {
        Self::default()
    }

    fn handle(&mut self, cx: &mut Cx, event: &mut Event) {
        //  self.single_button.handle(cx, event);
        for flame_rect in &mut self.flame_rects {
            flame_rect.handle(cx, event);
        }
    }

    fn draw(&mut self, cx: &mut Cx) {
        /*
        let data = vec![
            Span { rect: Rect { pos: vec2(0.1, 0.), size: vec2(0.7, 0.1) }, label: "slow".to_string(), color: COLOR_RED },
            Span { rect: Rect { pos: vec2(0.1, 0.3), size: vec2(0.4, 0.1) }, label: "faster".to_string(), color: COLOR_RED },
        ];
        */
        let mut data = Vec::new();
        for (y, level) in levels().iter().enumerate() {
            let mut running_x = 0;
            for j in (0..level.len()).step_by(4) {
                running_x += level[j];
                let x = running_x as f32 / (NUM_TICKS as f32);
                let w = level[j + 1] as f32 / (NUM_TICKS as f32);
                running_x += level[j + 1];
                // TODO use offsets
                let label = NAMES[level[j + 3] as usize];
                data.push(Span {
                    rect: Rect { pos: vec2(x, y as f32 * 0.07), size: vec2(w, 0.06) },
                    label: label.to_string(),
                    color: COLOR_RED,
                })
            }
        }

        self.window.begin_window(cx);
        self.pass.begin_pass(cx, Vec4::color("300"));
        self.main_view.begin_view(cx, LayoutSize::FILL);
        cx.begin_padding_box(Padding::top(30.));

        self.flame_rects.resize_with(data.len(), Default::default);
        for (i, span) in data.iter().enumerate() {
            self.flame_rects[i].draw(cx, &span)
        }
        //self.single_button.draw(cx);

        cx.end_padding_box();
        self.main_view.end_view(cx);
        self.pass.end_pass(cx);
        self.window.end_window(cx);
    }
}

main_app!(FlamegraphExampleApp);
