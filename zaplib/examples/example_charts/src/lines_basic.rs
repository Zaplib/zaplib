use zaplib::*;
use zaplib_components::*;

use crate::ChartExample;

pub(crate) struct LinesBasic {
    pub(crate) chart: Chart,
    pub(crate) datasets: Vec<Vec<f32>>,
    pub(crate) randomize_btn: Button,
    pub(crate) add_dataset_btn: Button,
    pub(crate) add_data_btn: Button,
    pub(crate) remove_dataset_btn: Button,
    pub(crate) remove_data_btn: Button,
    pub(crate) reset_view_btn: Button,
    pub(crate) style: ChartStyle,
    pub(crate) tooltip: ChartTooltipConfig,
    pub(crate) pan_enabled: bool,
    pub(crate) zoom_enabled: bool,
}

impl Default for LinesBasic {
    fn default() -> Self {
        let mut ret = Self {
            chart: Chart::default(),
            datasets: vec![],
            randomize_btn: Button::default(),
            add_dataset_btn: Button::default(),
            add_data_btn: Button::default(),
            remove_dataset_btn: Button::default(),
            remove_data_btn: Button::default(),
            reset_view_btn: Button::default(),
            style: CHART_STYLE_LIGHT,
            tooltip: Default::default(),
            pan_enabled: false,
            zoom_enabled: false,
        };

        // Add two initial datasets
        ret.add_dataset();
        ret.add_dataset();

        ret
    }
}

impl LinesBasic {
    pub(crate) fn with_dark_style() -> Self {
        Self { style: CHART_STYLE_DARK, ..Self::default() }
    }

    pub(crate) fn with_zoom() -> Self {
        Self { zoom_enabled: true, ..Self::default() }
    }

    pub(crate) fn with_pan() -> Self {
        Self { pan_enabled: true, ..Self::default() }
    }

    pub(crate) fn with_zoom_and_pan() -> Self {
        Self { zoom_enabled: true, pan_enabled: true, ..Self::default() }
    }

    fn get_random_data(count: usize) -> Vec<f32> {
        if count == 0 {
            vec![]
        } else {
            (0..count).into_iter().map(|_| -100. + 200. * (universal_rand::random_128() as f32 / f32::MAX)).collect()
        }
    }

    fn randomize(&mut self) {
        if self.datasets.is_empty() {
            return;
        }

        let data_count = self.datasets[0].len();
        for data in &mut self.datasets {
            *data = Self::get_random_data(data_count);
        }
    }

    fn add_dataset(&mut self) {
        let data_count = {
            if self.datasets.is_empty() {
                7 // some arbitrary size
            } else {
                self.datasets[0].len()
            }
        };
        self.datasets.push(Self::get_random_data(data_count));
    }

    fn add_data(&mut self) {
        for data in &mut self.datasets {
            data.push(Self::get_random_data(1)[0])
        }
    }

    fn remove_dataset(&mut self) {
        if self.datasets.is_empty() {
            return;
        }

        self.datasets.pop();
    }

    fn remove_data(&mut self) {
        for data in &mut self.datasets {
            data.pop();
        }
    }

    fn draw_chart(&mut self, cx: &mut Cx) {
        cx.begin_row(Width::Fill, Height::Fix(cx.get_height_left() - 70.));
        cx.begin_padding_box(Padding::top(20.));

        let colors = vec![COLOR_RED, COLOR_ORANGE, COLOR_YELLOW, COLOR_GREEN, COLOR_BLUE, COLOR_PURPLE, COLOR_GRAY];
        let months = vec![
            "January",
            "February",
            "March",
            "April",
            "May",
            "June",
            "July",
            "August",
            "September",
            "October",
            "November",
            "December",
        ];

        let datasets: Vec<ChartDataset> = self
            .datasets
            .iter()
            .enumerate()
            .map(|(i, data)| ChartDataset {
                label: format!("Dataset {}", i),
                data: ChartData::from_values(data),
                point_background_color: colors[i % colors.len()],
                point_radius: 4.,
                border_color: colors[i % colors.len()],
                border_width: 2.,
                ..ChartDataset::default()
            })
            .collect();

        // Generate labels
        let mut labels = vec![];
        if let Some(data_count) = datasets.iter().map(|ds| ds.data.len()).max() {
            for i in 0..data_count {
                labels.push(months[i % months.len()].to_string());
            }
        }

        let config = ChartConfig {
            labels,
            chart_type: ChartType::Line,
            datasets,
            style: self.style.clone(),
            tooltip: self.tooltip.clone(),
            zoom_enabled: self.zoom_enabled,
            pan_enabled: self.pan_enabled,
            ..ChartConfig::default()
        };

        self.chart.draw(cx, &config);

        cx.end_padding_box();
        cx.end_row();
    }

    pub fn draw_bottom_bar(&mut self, cx: &mut Cx) {
        cx.begin_row(Width::Fill, Height::Fix(50.));
        self.randomize_btn.draw(cx, "Randomize");
        self.add_dataset_btn.draw(cx, "Add Dataset");
        self.add_data_btn.draw(cx, "Add Data");
        self.remove_dataset_btn.draw(cx, "Remove Dataset");
        self.remove_data_btn.draw(cx, "Remove Data");

        if self.zoom_enabled || self.pan_enabled {
            self.reset_view_btn.draw(cx, "Reset View");
        }
        cx.end_row();
    }
}

impl ChartExample for LinesBasic {
    fn handle(&mut self, cx: &mut Cx, event: &mut Event) -> ChartEvent {
        if let ButtonEvent::Clicked = self.randomize_btn.handle(cx, event) {
            self.randomize();
        }

        if let ButtonEvent::Clicked = self.add_dataset_btn.handle(cx, event) {
            self.add_dataset();
        }

        if let ButtonEvent::Clicked = self.add_data_btn.handle(cx, event) {
            self.add_data();
        }

        if let ButtonEvent::Clicked = self.remove_dataset_btn.handle(cx, event) {
            self.remove_dataset();
        }

        if let ButtonEvent::Clicked = self.remove_data_btn.handle(cx, event) {
            self.remove_data();
        }

        if let ButtonEvent::Clicked = self.reset_view_btn.handle(cx, event) {
            self.chart.reset_zoom_pan();
        }

        self.chart.handle(cx, event)
    }

    fn draw(&mut self, cx: &mut Cx) {
        cx.begin_column(Width::Fill, Height::Fill);

        self.draw_chart(cx);
        self.draw_bottom_bar(cx);

        cx.end_column();
    }
}
