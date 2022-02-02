use std::sync::{Arc, RwLock};

use zaplib::*;

use crate::*;

#[derive(Default)]
struct Tooltip {
    value: f32,
    path: String,
}

impl ChartTooltipRenderer for Tooltip {
    fn draw_tooltip(&self, cx: &mut Cx, config: &ChartConfig, pos: Vec2) {
        let text_props =
            TextInsProps { text_style: TEXT_STYLE_MONO, color: config.style.background_color, ..TextInsProps::DEFAULT };

        TextIns::draw_str(cx, "This is a custom tooltip", pos + vec2(10., 10.), &text_props);
        TextIns::draw_str(cx, &format!("Value: {}", self.value), pos + vec2(10., 25.), &text_props);
        TextIns::draw_str(cx, &format!("Path: {}", self.path), pos + vec2(10., 40.), &text_props);
    }
}

pub(crate) struct TooltipCustomExample {
    base: LinesBasic,
    tooltip: Arc<RwLock<Tooltip>>,
}

impl Default for TooltipCustomExample {
    fn default() -> Self {
        let tooltip = Arc::new(RwLock::new(Tooltip::default()));
        Self {
            base: LinesBasic {
                tooltip: ChartTooltipConfig { size: vec2(200., 60.), renderer: Some(tooltip.clone()) },
                ..LinesBasic::default()
            },
            tooltip,
        }
    }
}

impl ChartExample for TooltipCustomExample {
    fn handle(&mut self, cx: &mut Cx, event: &mut Event) -> ChartEvent {
        if let ChartEvent::PointerHover { current_element: Some(current_element), .. } = self.base.handle(cx, event) {
            let mut write = self.tooltip.write().unwrap();
            write.value = current_element.data_point.y;
            write.path = format!("{:.0}", current_element.data_point.x);
        }

        ChartEvent::None
    }

    fn draw(&mut self, cx: &mut Cx) {
        self.base.draw(cx);
    }
}
