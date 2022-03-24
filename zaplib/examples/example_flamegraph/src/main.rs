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
    zoom_pan: ZoomPan,
    pointer_start_x_offset: Option<f32>,
    component_id: ComponentId,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ZoomPan {
    x_offset: f32,
    width: f32,
}

impl Default for ZoomPan {
    fn default() -> Self {
        Self { x_offset: 0.0, width: 1.0 }
    }
}

pub struct Span {
    /// Fraction of the width of the container.
    pub offset: f32,
    /// Fraction of the width of the container.
    pub width: f32,
    /// Absolute level number; top level is 0.
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

        let view_rect = self.main_view.get_rect(cx);
        match event.hits_pointer(cx, self.component_id, Some(view_rect)) {
            Event::PointerScroll(pse) => {
                self.zoom_pan.width = (self.zoom_pan.width + pse.scroll.y / 300.0).clamp(0.001, 1.0);
                cx.request_draw();
            }
            Event::PointerDown(pd) => {
                if pd.button == MouseButton::Left {
                    self.pointer_start_x_offset = Some(self.zoom_pan.x_offset);
                }
            }
            Event::PointerUp(_pd) => {
                self.pointer_start_x_offset = None;
            }
            Event::PointerMove(pm) => {
                if let Some(pointer_start_x_offset) = self.pointer_start_x_offset {
                    self.zoom_pan.x_offset = pointer_start_x_offset + (pm.abs.x - pm.abs_start.x) * self.zoom_pan.width / view_rect.size.x;
                    cx.request_draw();
                }
                cx.request_draw();
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
            self.flame_rects[i].draw(cx, &span, self.zoom_pan);
        }

        cx.end_padding_box();
        self.main_view.end_view(cx);
        self.pass.end_pass(cx);
        self.window.end_window(cx);
    }
}

main_app!(FlamegraphExampleApp);
