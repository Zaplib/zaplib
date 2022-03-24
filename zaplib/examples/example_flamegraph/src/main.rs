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
    spans: Vec<Span>,
}

pub struct Span {
    pub offset: f32,
    pub width: f32,
    pub level: u32,
    pub label: String,
    pub color: Vec4,
}

impl FlamegraphExampleApp {
    fn new(_cx: &mut Cx) -> Self {
        Self::default()
    }

    fn handle(&mut self, cx: &mut Cx, event: &mut Event) {
        for flame_rect in &mut self.flame_rects {
            flame_rect.handle(cx, event);
        }

        match event {
            Event::Construct => {
                // From https://personal.sron.nl/~pault/#sec:qualitative
                let colors = [
                    Vec4::color("#77AADD"),
                    Vec4::color("#EE8866"),
                    Vec4::color("#EEDD88"),
                    Vec4::color("#FFAABB"),
                    Vec4::color("#99DDFF"),
                    Vec4::color("#44BB99"),
                    Vec4::color("#BBCC33"),
                    Vec4::color("#AAAA00"),
                    Vec4::color("#DDDDDD"),
                ];

                for (y, level) in levels().iter().enumerate() {
                    let mut running_x = 0;
                    for j in (0..level.len()).step_by(4) {
                        running_x += level[j];
                        let x = running_x as f32 / (NUM_TICKS as f32);
                        let width = level[j + 1] as f32 / (NUM_TICKS as f32);
                        running_x += level[j + 1];
                        let name_id = level[j + 3] as usize;
                        let label = NAMES[name_id];
                        self.spans.push(Span {
                            offset: x,
                            width,
                            level: y as u32,
                            // TODO use offsets
                            label: label.to_string(),
                            color: colors[name_id % colors.len()],
                        })
                    }
                }
            }
            _ => (),
        }
    }

    fn draw(&mut self, cx: &mut Cx) {
        self.window.begin_window(cx);
        self.pass.begin_pass(cx, Vec4::color("300"));
        self.main_view.begin_view(cx, LayoutSize::FILL);
        cx.begin_padding_box(Padding::top(30.));

        self.flame_rects.resize_with(self.spans.len(), Default::default);
        for (i, span) in self.spans.iter().enumerate() {
            self.flame_rects[i].draw(cx, &span)
        }

        cx.end_padding_box();
        self.main_view.end_view(cx);
        self.pass.end_pass(cx);
        self.window.end_window(cx);
    }
}

main_app!(FlamegraphExampleApp);
