use zaplib::*;
use zaplib_components::*;

mod chart_list;
use chart_list::*;

mod lines_basic;
use lines_basic::*;
mod tooltip_custom;
use tooltip_custom::*;

pub(crate) trait ChartExample {
    fn handle(&mut self, cx: &mut Cx, event: &mut Event) -> ChartEvent;
    fn draw(&mut self, cx: &mut Cx);
}

pub struct ChartsExampleApp {
    window: Window,
    pass: Pass,
    main_view: View,
    splitter: Splitter,
    chart_list: ChartList,
    chart: Box<dyn ChartExample>,
}

impl ChartsExampleApp {
    pub fn new(_: &mut Cx) -> Self {
        let mut splitter = Splitter::default();
        splitter.set_splitter_state(SplitterAlign::First, 300., Axis::Vertical);
        Self {
            window: Window { create_inner_size: Some(vec2(1000., 700.)), ..Window::default() },
            pass: Pass::default(),
            main_view: View::default(),
            splitter,
            chart_list: ChartList::with_items(vec![
                "Lines",
                "Lines - Dark",
                "Lines - Styling (TODO)",
                "Tooltip - Custom",
                "Interaction - Zoom",
                "Interaction - Pan",
                "Interaction - Zoom & Pan",
            ]),
            chart: Box::new(LinesBasic::default()),
        }
    }

    pub fn handle(&mut self, cx: &mut Cx, event: &mut Event) {
        match self.splitter.handle(cx, event) {
            SplitterEvent::Moving { .. } => {
                cx.request_draw();
            }
            _ => (),
        }

        if let ChartListEvent::ChartSelected(selected) = self.chart_list.handle(cx, event) {
            match selected {
                "Lines" => self.chart = Box::new(LinesBasic::default()),
                "Lines - Dark" => self.chart = Box::new(LinesBasic::with_dark_style()),
                "Tooltip - Custom" => self.chart = Box::new(TooltipCustomExample::default()),
                "Interaction - Zoom" => self.chart = Box::new(LinesBasic::with_zoom()),
                "Interaction - Pan" => self.chart = Box::new(LinesBasic::with_pan()),
                "Interaction - Zoom & Pan" => self.chart = Box::new(LinesBasic::with_zoom_and_pan()),
                _ => (),
            }
            cx.request_draw();
        }

        self.chart.handle(cx, event);
    }

    pub fn draw(&mut self, cx: &mut Cx) {
        self.window.begin_window(cx);
        self.pass.begin_pass(cx, COLOR_WHITE);
        self.main_view.begin_view(cx, LayoutSize::FILL);

        cx.begin_row(Width::Fill, Height::Fill);
        {
            self.splitter.begin_draw(cx);
            self.chart_list.draw(cx);
            self.splitter.mid_draw(cx);
            self.chart.draw(cx);
            self.splitter.end_draw(cx);
        }
        cx.end_row();

        self.main_view.end_view(cx);
        self.pass.end_pass(cx);
        self.window.end_window(cx);
        cx.request_draw();
    }
}

main_app!(ChartsExampleApp);
