use crate::axis::*;
use crate::scrollbar::*;
use zaplib::*;

#[derive(Debug)]
pub struct ScrollBarConfig {
    pub(crate) smoothing: Option<f32>,
    pub(crate) bar_size: f32,
    pub(crate) use_vertical_pointer_scroll: bool,
}

impl Default for ScrollBarConfig {
    fn default() -> Self {
        Self { bar_size: 12.0, smoothing: None, use_vertical_pointer_scroll: false }
    }
}

impl ScrollBarConfig {
    #[must_use]
    pub fn with_bar_size(self, bar_size: f32) -> Self {
        Self { bar_size, ..self }
    }
    #[must_use]
    pub fn with_smoothing(self, s: f32) -> Self {
        Self { smoothing: Some(s), ..self }
    }
    #[must_use]
    pub fn with_use_vertical_pointer_scroll(self, use_vertical_pointer_scroll: bool) -> Self {
        Self { use_vertical_pointer_scroll, ..self }
    }
}

#[derive(Default)]
pub struct ScrollView {
    view: View,
    scroll_h: Option<ScrollBar>,
    scroll_v: Option<ScrollBar>,
}

impl ScrollView {
    #[must_use]
    pub fn new_standard_vh() -> Self {
        Self {
            scroll_h: Some(ScrollBar::default()),
            scroll_v: Some(ScrollBar::new(ScrollBarConfig::default().with_smoothing(0.15))),
            ..Self::default()
        }
    }

    #[must_use]
    pub fn with_scroll_h(self, config: ScrollBarConfig) -> Self {
        Self { scroll_h: Some(ScrollBar::new(config)), ..self }
    }

    #[must_use]
    pub fn with_scroll_v(self, config: ScrollBarConfig) -> Self {
        Self { scroll_v: Some(ScrollBar::new(config)), ..self }
    }

    pub fn begin_view(&mut self, cx: &mut Cx, layout_size: LayoutSize) {
        self.view.begin_view(cx, layout_size);
    }

    pub fn handle(&mut self, cx: &mut Cx, event: &mut Event) -> bool {
        let mut ret_h = ScrollBarEvent::None;
        let mut ret_v = ScrollBarEvent::None;

        if let Some(scroll_h) = &mut self.scroll_h {
            ret_h = scroll_h.handle(cx, event);
        }
        if let Some(scroll_v) = &mut self.scroll_v {
            ret_v = scroll_v.handle(cx, event);
        }
        if let Some(view_id) = self.view.view_id {
            match ret_h {
                ScrollBarEvent::None => (),
                ScrollBarEvent::Scroll { scroll_pos, .. } => {
                    cx.set_view_scroll_x(view_id, scroll_pos);
                }
                _ => (),
            };
            match ret_v {
                ScrollBarEvent::None => (),
                ScrollBarEvent::Scroll { scroll_pos, .. } => {
                    cx.set_view_scroll_y(view_id, scroll_pos);
                }
                _ => (),
            };
            ret_h != ScrollBarEvent::None || ret_v != ScrollBarEvent::None
        } else {
            false
        }
    }

    pub fn get_scroll_pos(&self, cx: &Cx) -> Vec2 {
        self.view.get_scroll_pos(cx)
    }

    pub fn set_scroll_pos(&mut self, cx: &mut Cx, pos: Vec2) -> bool {
        let view_id = self.view.view_id.unwrap();
        //let view_area = Area::DrawList(DrawListArea{draw_list_id:draw_list_id, redraw_id:cx.redraw_id});
        let mut changed = false;
        if let Some(scroll_h) = &mut self.scroll_h {
            if scroll_h.set_scroll_pos(cx, pos.x) {
                let scroll_pos = scroll_h.get_scroll_pos();
                cx.set_view_scroll_x(view_id, scroll_pos);
                changed = true;
            }
        }
        if let Some(scroll_v) = &mut self.scroll_v {
            if scroll_v.set_scroll_pos(cx, pos.y) {
                let scroll_pos = scroll_v.get_scroll_pos();
                cx.set_view_scroll_y(view_id, scroll_pos);
                changed = true;
            }
        }
        changed
    }

    pub fn set_scroll_view_total(&mut self, cx: &mut Cx, view_total: Vec2) {
        if let Some(scroll_h) = &mut self.scroll_h {
            scroll_h.set_scroll_view_total(cx, view_total.x)
        }
        if let Some(scroll_v) = &mut self.scroll_v {
            scroll_v.set_scroll_view_total(cx, view_total.y)
        }
    }

    pub fn get_scroll_view_total(&mut self) -> Vec2 {
        Vec2 {
            x: if let Some(scroll_h) = &mut self.scroll_h { scroll_h.get_scroll_view_total() } else { 0. },
            y: if let Some(scroll_v) = &mut self.scroll_v { scroll_v.get_scroll_view_total() } else { 0. },
        }
    }

    pub fn scroll_into_view(&mut self, cx: &mut Cx, rect: Rect) {
        if let Some(scroll_h) = &mut self.scroll_h {
            scroll_h.scroll_into_view(cx, rect.pos.x, rect.size.x, true);
        }
        if let Some(scroll_v) = &mut self.scroll_v {
            scroll_v.scroll_into_view(cx, rect.pos.y, rect.size.y, true);
        }
    }

    pub fn scroll_into_view_no_smooth(&mut self, cx: &mut Cx, rect: Rect) {
        if let Some(scroll_h) = &mut self.scroll_h {
            scroll_h.scroll_into_view(cx, rect.pos.x, rect.size.x, false);
        }
        if let Some(scroll_v) = &mut self.scroll_v {
            scroll_v.scroll_into_view(cx, rect.pos.y, rect.size.y, false);
        }
    }

    pub fn scroll_into_view_abs(&mut self, cx: &mut Cx, rect: Rect) {
        let self_rect = self.get_rect(cx);
        if let Some(scroll_h) = &mut self.scroll_h {
            scroll_h.scroll_into_view(cx, rect.pos.x - self_rect.pos.x, rect.size.x, true);
        }
        if let Some(scroll_v) = &mut self.scroll_v {
            scroll_v.scroll_into_view(cx, rect.pos.y - self_rect.pos.y, rect.size.y, true);
        }
    }

    pub fn set_scroll_target(&mut self, cx: &mut Cx, pos: Vec2) {
        if let Some(scroll_h) = &mut self.scroll_h {
            scroll_h.set_scroll_target(cx, pos.x);
        }
        if let Some(scroll_v) = &mut self.scroll_v {
            scroll_v.set_scroll_target(cx, pos.y);
        }
    }

    pub fn end_view(&mut self, cx: &mut Cx) -> Area {
        // lets ask the box our actual bounds
        let view_total = cx.get_box_bounds();
        let mut rect_now = cx.get_box_rect();
        if rect_now.size.y.is_nan() {
            rect_now.size.y = view_total.y;
        }
        if rect_now.size.x.is_nan() {
            rect_now.size.x = view_total.x;
        }

        let view_id = self.view.view_id.unwrap();
        if let Some(scroll_h) = &mut self.scroll_h {
            let scroll_pos = scroll_h.draw(cx, Axis::Horizontal, self.view.area(), rect_now, view_total);
            cx.set_view_scroll_x(view_id, scroll_pos);
        }
        if let Some(scroll_v) = &mut self.scroll_v {
            //println!("SET SCROLLBAR {} {}", rect_now.h, view_total.y);
            let scroll_pos = scroll_v.draw(cx, Axis::Vertical, self.view.area(), rect_now, view_total);
            cx.set_view_scroll_y(view_id, scroll_pos);
        }

        self.view.end_view(cx)
    }

    pub fn get_rect(&mut self, cx: &Cx) -> Rect {
        self.view.get_rect(cx)
    }

    pub fn area(&self) -> Area {
        self.view.area()
    }
}
