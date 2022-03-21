use std::{
    collections::HashMap,
    f32::{EPSILON, INFINITY, NEG_INFINITY},
    sync::{Arc, RwLock},
};

use crate::*;
use zaplib::*;

#[derive(Default)]
pub struct ChartTooltip {
    background: Background,
    target_pos: Vec2,
    value: f32,
    axis: f32,
    dataset: usize,
}

impl ChartTooltip {
    fn update(&mut self, current_element: &ChartCurrentElement) {
        self.target_pos = current_element.normalized_data_point;
        self.value = current_element.data_point.y;
        self.axis = current_element.data_point.x;
        self.dataset = current_element.dataset_index;
    }

    fn draw(&mut self, cx: &mut Cx, config: &ChartConfig) {
        let bounds = cx.get_box_rect();

        // Simple coloring for the tooltip: set the text color to be the same as the
        // chart's background, while using the invert of that color as the background
        // of the tooltip.
        let text_color = config.style.background_color;
        let mut background_color = vec4(1., 1., 1., 1.) - text_color;
        background_color.w = 1.; // Keep a valid alpha color.

        let size = config.tooltip.size.max(&vec2(130., 50.));
        let arrow_pointer_size = vec2(10., 10.);

        // Center the tooltip horizontally, always on top of the current element
        let mut pos = self.target_pos - vec2(0.5 * size.x, size.y);

        // Keep the tooltip inside the chart's horizontal bounds
        pos.x = pos.x.clamp(bounds.pos.x, bounds.pos.x + bounds.size.x - size.x);

        // Make sure the tooltip is snapped to the top border if
        // the current element is too close to it.
        if pos.y < 0.5 * size.y {
            pos.y += size.y;
        }

        // Shift the tooltip a bit up/down based on the current element position,
        // so the pointer is visible. Also, since the tooltip is mostly on top
        // of the current element, the pointer is inverted by default but the
        // tip of the arrow must always point to the current element
        let arrow_pointer_direction = if pos.y < self.target_pos.y {
            pos.y -= arrow_pointer_size.y - 1.;
            ArrowPointerDirection::Down
        } else {
            pos.y += arrow_pointer_size.y - 1.;
            ArrowPointerDirection::Up
        };

        // The pointer is always drawn at the current element's position.
        ArrowPointerIns::draw(cx, self.target_pos, background_color, arrow_pointer_direction, arrow_pointer_size);

        self.background.draw(cx, Rect { pos, size }, background_color);

        if let Some(renderer) = &config.tooltip.renderer {
            renderer.read().unwrap().draw_tooltip(cx, config, pos);
        } else {
            let text_props = TextInsProps { text_style: TEXT_STYLE_MONO, color: text_color, ..TextInsProps::DEFAULT };

            let path = {
                if (self.axis as usize) < config.labels.len() {
                    config.labels[self.axis as usize].to_string()
                } else {
                    format!("{}", self.axis)
                }
            };

            TextIns::draw_str(cx, &path, pos + vec2(10., 10.), &text_props);
            TextIns::draw_str(cx, &format!("Dataset {}: {:.3}", self.dataset, self.value), pos + vec2(10., 25.), &text_props);
        }
    }
}

/// TODO(hernan): Implement other chart types like bar, pie, etc...
pub enum ChartType {
    Line,
}

/// Contains the data is going to be used to render the chart
///
/// Input data can be represented in different formats and we
/// use references to avoid an extra copy
#[derive(Debug, Clone)]
pub enum ChartData<'a> {
    Empty,
    Values(&'a [f32]),
    Pairs(&'a [Vec2]),
}

impl<'a> ChartData<'a> {
    pub fn from_values(data: &'a [f32]) -> ChartData<'a> {
        ChartData::Values(data)
    }

    pub fn from_pairs(data: &'a [Vec2]) -> ChartData<'a> {
        ChartData::Pairs(data)
    }

    pub fn len(&self) -> usize {
        match self {
            ChartData::Values(data) => data.len(),
            ChartData::Pairs(data) => data.len(),
            ChartData::Empty => 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            ChartData::Values(data) => data.is_empty(),
            ChartData::Pairs(data) => data.is_empty(),
            ChartData::Empty => true,
        }
    }

    // TODO(hernan): There's probably a better way to do this
    pub fn min_max(&self, lo: Vec2, hi: Vec2) -> (Vec2, Vec2) {
        match self {
            ChartData::Values(data) => {
                let mut min = lo;
                let mut max = hi;
                for i in 0..data.len() {
                    let p = vec2(i as f32, data[i]);
                    min = min.min(&p);
                    max = max.max(&p);
                }
                (min, max)
            }
            ChartData::Pairs(data) => {
                let mut min = lo;
                let mut max = hi;
                for p in *data {
                    min = min.min(p);
                    max = max.max(p);
                }
                (min, max)
            }
            ChartData::Empty => (lo, hi),
        }
    }

    pub fn value_at(&self, i: usize) -> Vec2 {
        match self {
            ChartData::Values(data) => vec2(i as f32, data[i]),
            ChartData::Pairs(data) => data[i],
            ChartData::Empty => vec2(INFINITY, INFINITY),
        }
    }

    // TODO(hernan): Prevent copying data
    pub fn points(&self) -> Vec<Vec2> {
        match self {
            ChartData::Values(data) => data.iter().enumerate().map(|(x, y)| vec2(x as f32, *y)).collect(),
            ChartData::Pairs(data) => data.to_vec(),
            ChartData::Empty => vec![],
        }
    }
}

#[derive(Debug, Clone)]
pub struct ChartDataset<'a> {
    pub label: String,
    pub data: ChartData<'a>,
    pub point_background_color: Vec4,
    pub point_radius: f32,
    pub point_style: DrawPoints3dStyle,
    pub border_color: Vec4,
    pub border_width: f32,
    pub show_line: bool,
}

impl<'a> Default for ChartDataset<'a> {
    fn default() -> Self {
        Self {
            label: String::new(),
            data: ChartData::Empty,
            point_background_color: COLOR_WHITE,
            point_radius: 10.,
            point_style: DrawPoints3dStyle::Circle,
            border_color: COLOR_WHITE,
            border_width: 2.,
            show_line: true,
        }
    }
}

pub struct ChartScale {
    pub min: f32,
    pub max: f32,
}

#[derive(Clone)]
pub struct ChartStyle {
    pub background_color: Vec4,
    pub grid_color: Vec4,
    pub label_color: Vec4,
}

pub const CHART_STYLE_LIGHT: ChartStyle =
    ChartStyle { background_color: COLOR_WHITE, grid_color: COLOR_LIGHTGRAY, label_color: COLOR_DARKGRAY };

pub const CHART_STYLE_DARK: ChartStyle =
    ChartStyle { background_color: COLOR_BLACK, grid_color: vec4(0.25, 0.25, 0.25, 1.), label_color: COLOR_WHITE };

/// Renders a tooltip's content
///
/// Implement this trait to render the elements that are shown inside a tooltip. The
/// tooltip's background and current element indicator (small triangle) are not rendered
/// by this function.
///
/// See [`ChartTooltipConfig`] for comments about size and other settings for tooltips.
pub trait ChartTooltipRenderer {
    /// Draw the tooltip's content
    ///
    /// Use `pos` to position the elements that need to be rendered.
    ///
    /// TODO(Hernan): Add support for boxes and other layout mechanisms.
    fn draw_tooltip(&self, cx: &mut Cx, config: &ChartConfig, pos: Vec2);
}

/// An extension machanism for charts
///
/// Implement this trait to render custom elements on top of the chart, but
/// behind tooltips, if any.
///
/// Note that any new element added by this function will not be considered for
/// selection and will not affect selection of existing elements drawn by the
/// chart.
pub trait ChartPlugin {
    fn draw(&mut self, cx: &mut Cx, config: &ChartConfig, chart_bounds: &Rect);
}

/// Tooltip configuration
///
/// TODO(Hernan): Provide customization options for background, current element
/// indicator and whether or not the tooltip should be visible and animated.
#[derive(Default, Clone)]
pub struct ChartTooltipConfig {
    /// Tooltip content's size
    ///
    /// This value does not take into account the size of the current element
    /// indicator, which is drawn outside of the tooltip's content rectangle.
    pub size: Vec2,
    /// Optional custom renderer for tooltips.
    ///
    /// If none is provided, the tooltip will be rendered with the default content
    /// and style.
    pub renderer: Option<Arc<RwLock<dyn ChartTooltipRenderer>>>,
}

/// These options are based on the ones provided by ChartJS
pub struct ChartConfig<'a> {
    pub chart_type: ChartType,
    /// If the [`ChartConfig::labels] property of the main data property is used,
    /// it has to contain the same amount of elements as the dataset with the most values.
    pub labels: Vec<String>,
    pub datasets: Vec<ChartDataset<'a>>,
    pub scales: HashMap<String, ChartScale>,
    pub style: ChartStyle,
    pub tooltip: ChartTooltipConfig,
    pub zoom_enabled: bool,
    pub pan_enabled: bool,
}

impl<'a> Default for ChartConfig<'a> {
    fn default() -> Self {
        Self {
            chart_type: ChartType::Line,
            labels: Vec::<String>::default(),
            datasets: vec![],
            scales: HashMap::new(),
            style: CHART_STYLE_DARK,
            tooltip: ChartTooltipConfig::default(),
            pan_enabled: false,
            zoom_enabled: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ChartCurrentElement {
    pub dataset_index: usize,
    pub datum_index: usize,
    pub data_point: Vec2,
    pub normalized_data_point: Vec2,
}

#[derive(Debug, Clone)]
pub enum ChartEvent {
    None,
    PointerOut,
    PointerHover {
        /// Current mouse position relative to the chart boundaries.
        /// Useful for positioning tooltips.
        cursor: Vec2,
        /// The value at the current mouse position
        /// This value might not be part of the input data. Instead, it's interpolated
        /// based on normalized values.
        cursor_value: Vec2,
        /// If exists, we also retreive the element in the input data that is closest
        /// to the current mouse position
        current_element: Option<ChartCurrentElement>,
    },
}

#[derive(Default)]
pub struct Chart {
    bounds: Rect,
    view: View,
    chart_view: View,
    component_id: ComponentId,
    texture_area: Area,
    pass: Pass,
    color_texture: Texture,
    min: Vec2,
    max: Vec2,
    background: Background,
    areas: Vec<Area>,
    tooltip: ChartTooltip,
    tooltip_visible: bool,
    // Keep an Arc<RwLock> to plugins since most of them are maybe owned
    // by other components. But we also need to call them here.
    pub plugins: Vec<Arc<RwLock<dyn ChartPlugin>>>,
    last_pointer_pos: Vec2,
    zoom_enabled: bool,
    pan_enabled: bool,
    panning: bool,

    /// Set to `None` when we're viewing the whole chart (e.g. initial state,
    /// and when clicking the "reset view" button) which means that the axes
    /// can automatically get resized as new data comes in; and set to `Some`
    /// the moment you drag or scroll.
    ///
    /// `pos` and `size` are in units of the horizontal and vertical axes, with
    // `pos` representing the top-right point in the chart (initially often 0,0).
    zoom_pan: Option<Rect>,
}

impl Chart {
    pub fn handle(&mut self, cx: &mut Cx, event: &mut Event) -> ChartEvent {
        match event.hits_pointer(cx, self.component_id, self.texture_area.get_rect_for_first_instance(cx)) {
            Event::PointerDown(pd) => {
                if self.pan_enabled {
                    self.last_pointer_pos = pd.rel;
                    self.panning = true;
                }
            }
            Event::PointerMove(pm) => {
                if self.pan_enabled {
                    let delta_pan = pm.rel - self.last_pointer_pos;
                    self.last_pointer_pos = pm.rel;

                    if self.zoom_pan.is_none() {
                        self.zoom_pan = Some(self.bounds);
                    }

                    if let Some(zoom_pan) = &mut self.zoom_pan {
                        zoom_pan.pos += delta_pan;
                    }

                    cx.request_draw();
                }
            }
            Event::PointerUp(_) => {
                if self.pan_enabled {
                    self.panning = false;
                }
            }
            Event::PointerScroll(ps) => {
                if self.zoom_enabled {
                    if self.zoom_pan.is_none() {
                        self.zoom_pan = Some(self.bounds);
                    }

                    if let Some(zoom_pan) = &mut self.zoom_pan {
                        let old_size = zoom_pan.size;
                        let new_size = old_size - vec2(ps.scroll.y, ps.scroll.y);
                        let zoom_factor = (new_size / old_size).y;

                        // Compute new offset
                        // See: https://stackoverflow.com/a/38302057
                        let old_pos = zoom_pan.pos;
                        let new_pos = ps.rel - zoom_factor * (ps.rel - old_pos);

                        zoom_pan.size = new_size;
                        zoom_pan.pos = new_pos;
                    }

                    cx.request_draw();
                }
            }
            Event::PointerHover(pe) => {
                if self.panning || pe.hover_state == HoverState::Out {
                    self.tooltip_visible = false;
                    return ChartEvent::PointerOut;
                }

                let mouse_pos_rel = pe.rel;
                let cursor = mouse_pos_rel.clamp(&self.bounds.pos, &(self.bounds.pos + self.bounds.size));
                let cursor_value = self.denormalize_data_point(cursor);
                let current_element = self.get_element_at(cx, cursor_value);
                if let Some(current_element) = &current_element {
                    self.tooltip.update(current_element);
                    self.tooltip_visible = true;
                } else {
                    self.tooltip_visible = false;
                }

                cx.request_draw();

                return ChartEvent::PointerHover { cursor, cursor_value, current_element };
            }
            _ => (),
        }

        ChartEvent::None
    }

    /// Gets the nearest element for a given value
    /// `stride` is used to jump between elements, in case they are represented by
    /// more than one value.
    fn get_nearest_element_index(data: &[DrawPoints3dInstance], value: f32) -> Option<usize> {
        if data.is_empty() {
            return None;
        }

        let partition_index = {
            let mut i = 0;
            // find the first element higher than the given value.
            while i < data.len() && data[i].user_info.x <= value {
                i += 1;
            }
            i
        };

        if partition_index >= data.len() {
            // The dataset cannot be partitioned, meaning the reference time is beyond
            // the available data (which might happen because the data is being loaded).
            // In this case, just return the last element, which should be the nearest
            // one since data is assumed to be sorted.
            return Some(data.len() - 1);
        }

        if partition_index == 0 {
            // All values are greater than the reference time, so just return the first
            // element.
            return Some(0);
        }

        // Compare values with the previous one
        if (data[partition_index].user_info.x - value).abs() >= (data[partition_index - 1].user_info.x - value).abs() {
            return Some(partition_index - 1);
        }

        Some(partition_index)
    }

    fn get_element_at(&self, cx: &mut Cx, cursor: Vec2) -> Option<ChartCurrentElement> {
        let mut ret: Option<ChartCurrentElement> = None;
        let mut min_distance = INFINITY;

        for dataset_index in 0..self.areas.len() {
            let area = &self.areas[dataset_index];
            let points = area.get_slice::<DrawPoints3dInstance>(cx);
            if let Some(datum_index) = Self::get_nearest_element_index(points, cursor.x) {
                let data_point = points[datum_index].user_info;
                let distance = (data_point.y - cursor.y).abs();
                if distance < min_distance {
                    ret = Some(ChartCurrentElement {
                        dataset_index,
                        datum_index,
                        data_point,
                        normalized_data_point: self.normalize_data_point(data_point),
                    });
                    min_distance = distance;
                }
            }
        }

        ret
    }

    fn remap(value: f32, lo0: f32, hi0: f32, lo1: f32, hi1: f32) -> f32 {
        lo1 + (value - lo0) / (hi0 - lo0) * (hi1 - lo1)
    }

    fn draw_grid(&mut self, cx: &mut Cx, config: &ChartConfig) {
        let min_x = self.bounds.pos.x;
        let max_x = min_x + self.bounds.size.x;
        let min_y = self.bounds.pos.y;
        let max_y = min_y + self.bounds.size.y;

        // Minimum size of a cell, in pixels.
        let min_cell_size = 50.;

        let data_min = self.min;
        let data_max = self.denormalize_data_point(vec2(max_x, max_y));

        let first = self.normalize_data_point(data_min);
        let last = self.normalize_data_point(data_max);

        let max_lines = (data_max.x - data_min.x).abs();
        let step_size = (last.x - first.x).abs() / max_lines;

        let mut lines = vec![];

        let mut draw_vertical_line = |x, round_op: &dyn Fn(f32) -> f32| {
            lines.push(DrawLines3dInstance::from_segment(
                vec3(x, min_y, 0.),
                vec3(x, max_y + 10., 0.),
                config.style.grid_color,
                1.,
            ));

            // TODO(hernan): Render text labels if provided in config
            let label = {
                let col_value = round_op(self.denormalize_data_point(vec2(x, min_y)).x);
                format!("{:.0}", col_value)
            };

            TextIns::draw_str(
                cx,
                &label,
                Vec2 { x, y: max_y + 10. },
                &TextInsProps {
                    position_anchoring: TEXT_ANCHOR_CENTER_H,
                    color: config.style.label_color,
                    ..TextInsProps::DEFAULT
                },
            );
        };

        // Lines are rendered in reversed order first
        // This prevents jumping when panning and zooming
        let mut x = first.x;
        let mut last_x = x + min_cell_size;
        while x > min_x {
            // Skip some lines in order to ensure there is enough
            // space between them
            if x < max_x && (last_x - x >= min_cell_size) {
                draw_vertical_line(x, &|x| x);
                last_x = x;
            }
            x -= step_size;
        }

        let mut x = first.x;
        let mut last_x = x - min_cell_size;
        while x < max_x {
            if min_x < x && (x - last_x >= min_cell_size) {
                draw_vertical_line(x, &|x| x);
                last_x = x;
            }
            x += step_size;
        }

        draw_vertical_line(min_x, &|x| x.floor());
        draw_vertical_line(max_x, &|x| x.ceil());

        let mut draw_horizontal_line = |y, round_op: &dyn Fn(f32) -> f32| {
            lines.push(DrawLines3dInstance::from_segment(
                vec3(min_x - 10., y, 0.),
                vec3(max_x, y, 0.),
                config.style.grid_color,
                1.,
            ));

            // Flip min_y/max_y since y coordinate is inverted
            let row_value = round_op(self.denormalize_data_point(vec2(min_x, y)).y);

            TextIns::draw_str(
                cx,
                &format!("{:.0}", row_value),
                Vec2 { x: min_x - 15., y },
                &TextInsProps {
                    position_anchoring: TEXT_ANCHOR_RIGHT + TEXT_ANCHOR_CENTER_V,
                    color: config.style.label_color,
                    ..TextInsProps::DEFAULT
                },
            );
        };

        // See comments above for rendering vertical lines
        let mut y = first.y;
        let mut last_y = y + min_cell_size;
        while y > min_y {
            if y < max_y && (last_y - y >= min_cell_size) {
                draw_horizontal_line(y, &|y| y);
                last_y = y;
            }
            y -= step_size;
        }

        let mut y = first.y;
        let mut last_y = y - min_cell_size;
        while y < max_y {
            if min_y < y && (y - last_y >= min_cell_size) {
                draw_horizontal_line(y, &|y| y);
                last_y = y;
            }
            y += step_size;
        }

        draw_horizontal_line(min_y, &|y| y.floor());
        draw_horizontal_line(max_y, &|y| y.ceil());

        // Draw axes a bit brighter than columns/rows
        let axis_color = vec4(0.5, 0.5, 0.5, 1.);
        lines.push(DrawLines3dInstance::from_segment(vec3(min_x, max_y, 0.), vec3(max_x, max_y, 0.), axis_color, 1.));
        lines.push(DrawLines3dInstance::from_segment(vec3(min_x, min_y, 0.), vec3(min_x, max_y, 0.), axis_color, 1.));

        DrawLines3d::draw(cx, &lines, Default::default());
    }

    /// Use Liang-Barsky algorithm to clip line its points are both inside
    /// the chart boundaries (see: <https://en.wikipedia.org/wiki/Liang%E2%80%93Barsky_algorithm>)
    fn draw_lines(&mut self, cx: &mut Cx, data: &[Vec2], color: Vec4, scale: f32) {
        let min_x = self.bounds.pos.x;
        let max_x = min_x + self.bounds.size.x;
        let min_y = self.bounds.pos.y;
        let max_y = min_y + self.bounds.size.y;

        let clip_line = |a: Vec2, b: Vec2| {
            let dx = b.x - a.x;
            let dy = b.y - a.y;

            let mut u1: f32 = 0.;
            let mut u2: f32 = 1.;

            let pk = [-dx, dx, -dy, dy];
            let qk = [a.x - min_x, max_x - a.x, a.y - min_y, max_y - a.y];

            for i in 0..4 {
                if pk[i].abs() < EPSILON {
                    if qk[i] < 0. {
                        return None;
                    }
                } else {
                    // Calculate the intersection point for the line and the window edge
                    let r = qk[i] / pk[i];
                    if pk[i] < 0. {
                        // Lines going outside to inside
                        u1 = u1.max(r);
                    } else if pk[i] > 0. {
                        // Lines going inside to outside
                        u2 = u2.min(r);
                    }
                }
            }

            if u1 > u2 {
                // The line is completely outside of the clipping window
                return None;
            }

            if u1 < 0. && 1. < u2 {
                // The line is completely inside of the clipping window
                return Some((a, b));
            }

            let a2 = vec2(a.x + u1 * dx, a.y + u1 * dy);
            let b2 = vec2(a.x + u2 * dx, a.y + u2 * dy);
            Some((a2, b2))
        };

        let mut lines = vec![];
        for i in 0..(data.len() - 1) {
            let a = data[i];
            let b = data[i + 1];
            if let Some((a, b)) = clip_line(a, b) {
                lines.push(DrawLines3dInstance::from_segment(a.to_vec3(), b.to_vec3(), color, scale));
            }
        }

        DrawLines3d::draw(cx, &lines, Default::default());
    }

    fn draw_points(
        &mut self,
        cx: &mut Cx,
        normalized_data: &[Vec2],
        original_data: &[Vec2],
        color: Vec4,
        scale: f32,
        point_style: DrawPoints3dStyle,
    ) -> Area {
        let min_x = self.bounds.pos.x;
        let max_x = min_x + self.bounds.size.x;
        let min_y = self.bounds.pos.y;
        let max_y = min_y + self.bounds.size.y;

        let color = color.to_vec3();
        let size = scale;
        let mut points = Vec::<DrawPoints3dInstance>::with_capacity(normalized_data.len());
        for i in 0..normalized_data.len() {
            let p = normalized_data[i];

            // Check if point is inside the chart boundaries before drawing
            if min_x <= p.x && p.x <= max_x && min_y <= p.y && p.y <= max_y {
                points.push(DrawPoints3dInstance { position: p.to_vec3(), color, size, user_info: original_data[i] });
            }
        }

        DrawPoints3d::draw(
            cx,
            &points,
            DrawPoints3dOptions { use_screen_space: true, point_style, ..DrawPoints3dOptions::default() },
        )
    }

    /// Compute offset and scaling based on zoom/pan values
    fn get_offset_scale(&self) -> (Vec2, Vec2) {
        if let Some(zoom_pan) = self.zoom_pan {
            let offset = zoom_pan.pos - self.bounds.pos;
            let scale = (zoom_pan.size / self.bounds.size).max(&vec2(0.1, 0.1));
            (offset, scale)
        } else {
            (vec2(0., 0.), vec2(1., 1.))
        }
    }

    /// Transform a data point from data coordinates to normalized screen coordinates
    fn normalize_data_point(&self, data_point: Vec2) -> Vec2 {
        let (offset, scale) = self.get_offset_scale();
        offset
            + scale
                * vec2(
                    // For x axis, we want values to be in the range [bounds.pos.x, bounds.pos.x + bounds.size.x],
                    // using (p.x - min.x) / (max.x - min.x) for interpolation.
                    self.bounds.pos.x + (data_point.x - self.min.x) / (self.max.x - self.min.x) * self.bounds.size.x,
                    // For y axis, it's a similar process except that we want charts to start at the bottom instead.
                    // Then, we add bounds.size.y and subtract the interpolated value.
                    (self.bounds.pos.y + self.bounds.size.y)
                        - (data_point.y - self.min.y) / (self.max.y - self.min.y) * self.bounds.size.y,
                )
    }

    /// Transform a normalized data point from screen coordinates to data coordinates
    fn denormalize_data_point(&self, normalized_data_point: Vec2) -> Vec2 {
        let (offset, scale) = self.get_offset_scale();
        let normalized_data_point = (normalized_data_point - offset) / scale;
        vec2(
            Self::remap(
                normalized_data_point.x,
                self.bounds.pos.x,
                self.bounds.pos.x + self.bounds.size.x,
                self.min.x,
                self.max.x,
            ),
            Self::remap(
                normalized_data_point.y,
                self.bounds.pos.y,
                self.bounds.pos.y + self.bounds.size.y,
                self.max.y,
                self.min.y,
            ),
        )
    }

    fn normalize(&self, data: &[Vec2]) -> Vec<Vec2> {
        data.iter().map(|p| self.normalize_data_point(*p)).collect()
    }

    /// Rounds a number to the closest power of 10, rounded up
    fn round_up_to_10s(value: f32) -> f32 {
        let exp = value.log10().ceil();
        10_f32.powf(exp)
    }

    /// Rounds a number to the closest power of 10, rounded down
    fn round_down_to_10s(value: f32) -> f32 {
        let exp = value.log10().floor();
        10_f32.powf(exp)
    }

    fn round_up(value: Vec2) -> Vec2 {
        vec2(value.x, if value.y < 0. { -Self::round_down_to_10s(value.y.abs()) } else { Self::round_up_to_10s(value.y) })
    }

    fn round_down(value: Vec2) -> Vec2 {
        vec2(value.x, if value.y < 0. { -Self::round_up_to_10s(value.y.abs()) } else { Self::round_down_to_10s(value.y) })
    }

    fn get_min_max(config: &ChartConfig) -> (Vec2, Vec2) {
        let mut min = vec2(INFINITY, INFINITY);
        let mut max = vec2(NEG_INFINITY, NEG_INFINITY);

        if let Some(x_scale) = config.scales.get("x") {
            min.x = x_scale.min;
            max.x = x_scale.max;
        }

        for dataset in &config.datasets {
            let (lo, hi) = dataset.data.min_max(min, max);
            min = lo;
            max = hi;
        }

        // Force either bound to be zero (but not both)
        if max.y < 0. {
            max.y = 0.;
        } else if min.y > 0. {
            min.y = 0.;
        }

        (Self::round_down(min), Self::round_up(max))
    }

    pub fn reset_zoom_pan(&mut self) {
        self.zoom_pan = None;
    }

    fn draw_chart(&mut self, cx: &mut Cx, config: &ChartConfig) {
        self.chart_view.begin_view(cx, LayoutSize::FILL);

        let rect = cx.get_box_rect();

        let current_dpi = cx.current_dpi_factor;

        self.background.draw(cx, rect, config.style.background_color);

        self.zoom_enabled = config.zoom_enabled;
        self.pan_enabled = config.pan_enabled;

        // Compute the rect where the chart will be rendered. The offsets below
        // add some marging so we can also render labels for each axis.
        // TODO(Hernan): should this be customizable?
        self.bounds = Rect { pos: rect.pos + vec2(60., 5.), size: rect.size - vec2(80., 40.) };

        if self.zoom_pan.is_none() {
            // Compute min/max for all datasets before rendering
            // Only update min/max values if we're not panning/zooming
            let (data_min, data_max) = Self::get_min_max(config);
            self.min = data_min;
            self.max = data_max;
        }

        self.areas = vec![];

        self.draw_grid(cx, config);

        for dataset in &config.datasets {
            let points = dataset.data.points();
            let normalized_data = self.normalize(&points);
            if !normalized_data.is_empty() {
                self.draw_lines(cx, &normalized_data, dataset.border_color, dataset.border_width * current_dpi);
                let area = self.draw_points(
                    cx,
                    &normalized_data,
                    &points,
                    dataset.point_background_color,
                    dataset.point_radius * current_dpi,
                    dataset.point_style,
                );
                self.areas.push(area);
            }
        }

        for plugin in &mut self.plugins {
            plugin.write().unwrap().draw(cx, config, &self.bounds)
        }

        if self.tooltip_visible {
            self.tooltip.draw(cx, config);
        }

        self.chart_view.end_view(cx);
    }

    fn draw_view(&mut self, cx: &mut Cx) {
        self.view.begin_view(cx, LayoutSize::FILL);
        let rect = cx.get_box_rect();
        let color_texture_handle = self.color_texture.get_color(cx);
        self.texture_area = ImageIns::draw(cx, rect, color_texture_handle);
        self.view.end_view(cx);
    }

    pub fn draw(&mut self, cx: &mut Cx, config: &ChartConfig) {
        self.draw_view(cx);

        self.pass.begin_pass_without_textures(cx);
        let rect = cx.get_box_rect();
        let color_texture_handle = self.color_texture.get_color(cx);
        self.pass.set_size(cx, rect.size);
        self.pass.add_color_texture(cx, color_texture_handle, ClearColor::default());

        self.draw_chart(cx, config);

        self.pass.end_pass(cx);
    }
}

#[cfg(test)]
mod tests {
    use zaplib::vec2;

    use crate::Chart;

    #[test]
    fn it_rounds_up() {
        assert_eq!(Chart::round_up(vec2(10., 943.)), vec2(10., 1000.));
        assert_eq!(Chart::round_up(vec2(10., -943.)), vec2(10., -100.));
        assert_eq!(Chart::round_up(vec2(10., 478.)), vec2(10., 1000.));
        assert_eq!(Chart::round_up(vec2(10., 5623.)), vec2(10., 10000.));
        assert_eq!(Chart::round_up(vec2(10., -876.)), vec2(10., -100.));
        assert_eq!(Chart::round_up(vec2(10., 33.)), vec2(10., 100.));
        assert_eq!(Chart::round_up(vec2(10., 7.)), vec2(10., 10.));
        assert_eq!(Chart::round_up(vec2(10., -7.)), vec2(10., -1.));
        assert_eq!(Chart::round_up(vec2(10., 99.)), vec2(10., 100.));
        assert_eq!(Chart::round_up(vec2(10., -99.)), vec2(10., -10.));
        assert_eq!(Chart::round_up(vec2(10., 100.)), vec2(10., 100.));
        assert_eq!(Chart::round_up(vec2(10., -100.)), vec2(10., -100.));
        assert_eq!(Chart::round_up(vec2(10., 1001.)), vec2(10., 10000.));
        assert_eq!(Chart::round_up(vec2(10., -1001.)), vec2(10., -1000.));
    }

    #[test]
    fn it_rounds_down() {
        assert_eq!(Chart::round_down(vec2(10., 943.)), vec2(10., 100.));
        assert_eq!(Chart::round_down(vec2(10., -943.)), vec2(10., -1000.));
        assert_eq!(Chart::round_down(vec2(10., 478.)), vec2(10., 100.));
        assert_eq!(Chart::round_down(vec2(10., 5623.)), vec2(10., 1000.));
        assert_eq!(Chart::round_down(vec2(10., -876.)), vec2(10., -1000.));
        assert_eq!(Chart::round_down(vec2(10., 33.)), vec2(10., 10.));
        assert_eq!(Chart::round_down(vec2(10., 7.)), vec2(10., 1.));
        assert_eq!(Chart::round_down(vec2(10., -7.)), vec2(10., -10.));
        assert_eq!(Chart::round_down(vec2(10., 99.)), vec2(10., 10.));
        assert_eq!(Chart::round_down(vec2(10., -99.)), vec2(10., -100.));
        assert_eq!(Chart::round_down(vec2(10., 100.)), vec2(10., 100.));
        assert_eq!(Chart::round_down(vec2(10., -100.)), vec2(10., -100.));
        assert_eq!(Chart::round_down(vec2(10., 1001.)), vec2(10., 1000.));
        assert_eq!(Chart::round_down(vec2(10., -1001.)), vec2(10., -10000.));
    }
}
