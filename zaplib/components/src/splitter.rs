use crate::axis::*;
use crate::background::*;
use zaplib::*;

pub struct Splitter {
    pub axis: Axis,
    pub align: SplitterAlign,
    pub pos: f32,

    pub component_id: ComponentId,
    pub min_size: f32,
    pub split_size: f32,
    pub bg: Background,
    pub animator: Animator,
    pub realign_dist: f32,
    pub split_view: View,
    pub _calc_pos: f32,
    pub _is_moving: bool,
    pub _drag_point: f32,
    pub _drag_pos_start: f32,
    pub _drag_max_pos: f32,
    pub _hit_state_margin: Option<Padding>,
}

#[derive(Clone, PartialEq)]
pub enum SplitterAlign {
    First,
    Last,
    Weighted,
}
impl Default for SplitterAlign {
    fn default() -> Self {
        Self::First
    }
}

#[derive(Clone, PartialEq)]
pub enum SplitterEvent {
    None,
    Moving { new_pos: f32 },
    MovingEnd { new_align: SplitterAlign, new_pos: f32 },
}

const ANIM_DEFAULT: Anim = Anim {
    duration: 0.5,
    tracks: &[
        // DrawColor::color
        Track::Vec4 { key_frames: &[(1.0, vec4(0.1, 0.1, 0.1, 1.0))], ease: Ease::DEFAULT },
    ],
    ..Anim::DEFAULT
};

const ANIM_OVER: Anim = Anim {
    duration: 0.05,
    tracks: &[
        // DrawColor::color
        Track::Vec4 { key_frames: &[(1.0, vec4(0.33, 0.33, 0.33, 1.0))], ease: Ease::DEFAULT },
    ],
    ..Anim::DEFAULT
};

const ANIM_DOWN: Anim = Anim {
    duration: 0.2,
    tracks: &[
        // DrawColor::color
        Track::Vec4 { key_frames: &[(0.0, vec4(1.0, 1.0, 1.0, 1.0)), (1.0, vec4(0.4, 0.4, 0.4, 1.0))], ease: Ease::DEFAULT },
    ],
    ..Anim::DEFAULT
};

impl Default for Splitter {
    fn default() -> Self {
        Self {
            axis: Axis::Vertical,
            align: SplitterAlign::First,
            pos: 0.0,

            component_id: Default::default(),
            _calc_pos: 0.0,
            _is_moving: false,
            _drag_point: 0.,
            _drag_pos_start: 0.,
            _drag_max_pos: 0.0,
            _hit_state_margin: None,
            realign_dist: 30.,
            split_size: 2.0,
            min_size: 25.0,
            split_view: View::default(),
            bg: Background::default().with_radius(0.5),
            animator: Animator::default(),
        }
    }
}
impl Splitter {
    fn animate(&mut self, cx: &mut Cx) {
        self.bg.set_color(cx, self.animator.get_vec4(0));
    }

    pub fn handle(&mut self, cx: &mut Cx, event: &mut Event) -> SplitterEvent {
        if self.animator.handle(cx, event) {
            self.animate(cx);
        }

        let rect = self
            .bg
            .area()
            .get_rect_for_first_instance(cx)
            .map(|rect| rect.add_padding(self._hit_state_margin.unwrap_or_default()));
        match event.hits_pointer(cx, self.component_id, rect) {
            Event::PointerDown(pe) => {
                self._is_moving = true;
                self.animator.play_anim(cx, ANIM_DOWN);
                match self.axis {
                    Axis::Horizontal => cx.set_down_mouse_cursor(MouseCursor::RowResize),
                    Axis::Vertical => cx.set_down_mouse_cursor(MouseCursor::ColResize),
                };
                self._drag_pos_start = self.pos;
                self._drag_point = match self.axis {
                    Axis::Horizontal => pe.rel.y,
                    Axis::Vertical => pe.rel.x,
                }
            }
            Event::PointerHover(pe) => {
                match self.axis {
                    Axis::Horizontal => cx.set_hover_mouse_cursor(MouseCursor::RowResize),
                    Axis::Vertical => cx.set_hover_mouse_cursor(MouseCursor::ColResize),
                };
                if !self._is_moving {
                    match pe.hover_state {
                        HoverState::In => {
                            self.animator.play_anim(cx, ANIM_OVER);
                        }
                        HoverState::Out => {
                            self.animator.play_anim(cx, ANIM_DEFAULT);
                        }
                        _ => (),
                    }
                }
            }
            Event::PointerUp(pe) => {
                self._is_moving = false;
                if pe.is_over {
                    if pe.input_type.has_hovers() {
                        self.animator.play_anim(cx, ANIM_OVER);
                    } else {
                        self.animator.play_anim(cx, ANIM_DEFAULT);
                    }
                } else {
                    self.animator.play_anim(cx, ANIM_DEFAULT);
                }
                // we should change our mode based on which edge we are closest to
                // the rule is center - 30 + 30
                let center = self._drag_max_pos * 0.5;
                if self._calc_pos > center - self.realign_dist && self._calc_pos < center + self.realign_dist {
                    self.align = SplitterAlign::Weighted;
                    self.pos = self._calc_pos / self._drag_max_pos;
                } else if self._calc_pos < center - self.realign_dist {
                    self.align = SplitterAlign::First;
                    self.pos = self._calc_pos;
                } else {
                    self.align = SplitterAlign::Last;
                    self.pos = self._drag_max_pos - self._calc_pos;
                }

                return SplitterEvent::MovingEnd { new_align: self.align.clone(), new_pos: self.pos };
            }
            Event::PointerMove(pe) => {
                let delta = match self.axis {
                    Axis::Horizontal => pe.abs_start.y - pe.abs.y,
                    Axis::Vertical => pe.abs_start.x - pe.abs.x,
                };
                let mut pos = match self.align {
                    SplitterAlign::First => self._drag_pos_start - delta,
                    SplitterAlign::Last => self._drag_pos_start + delta,
                    SplitterAlign::Weighted => self._drag_pos_start * self._drag_max_pos - delta,
                };
                if pos > self._drag_max_pos - self.min_size {
                    pos = self._drag_max_pos - self.min_size
                } else if pos < self.min_size {
                    pos = self.min_size
                };
                let calc_pos = match self.align {
                    SplitterAlign::First => {
                        self.pos = pos;
                        pos
                    }
                    SplitterAlign::Last => {
                        self.pos = pos;
                        self._drag_max_pos - pos
                    }
                    SplitterAlign::Weighted => {
                        self.pos = pos / self._drag_max_pos;
                        pos
                    }
                };
                //log_str(&format!("CALC POS {}", calc_pos));
                // pixelsnap calc_pos
                if calc_pos != self._calc_pos {
                    self._calc_pos = calc_pos;
                    cx.request_draw();
                    return SplitterEvent::Moving { new_pos: self.pos };
                }
            }
            _ => (),
        };
        SplitterEvent::None
    }

    pub fn set_splitter_state(&mut self, align: SplitterAlign, pos: f32, axis: Axis) {
        self.axis = axis;
        self.align = align;
        self.pos = pos;
        match self.axis {
            Axis::Horizontal => self._hit_state_margin = Some(Padding { l: 0., t: 3., r: 0., b: 7. }),
            Axis::Vertical => self._hit_state_margin = Some(Padding { l: 3., t: 0., r: 7., b: 0. }),
        }
    }

    pub fn begin_draw(&mut self, cx: &mut Cx) {
        let rect = cx.get_box_rect();
        self._calc_pos = match self.align {
            SplitterAlign::First => self.pos,
            SplitterAlign::Last => match self.axis {
                Axis::Horizontal => rect.size.y - self.pos,
                Axis::Vertical => rect.size.x - self.pos,
            },
            SplitterAlign::Weighted => {
                self.pos
                    * match self.axis {
                        Axis::Horizontal => rect.size.y,
                        Axis::Vertical => rect.size.x,
                    }
            }
        };
        let dpi_factor = cx.get_dpi_factor_of(&self.bg.area());
        self._calc_pos -= self._calc_pos % (1.0 / dpi_factor);
        match self.axis {
            Axis::Horizontal => cx.begin_row(Width::Fill, Height::Fix(self._calc_pos)),
            Axis::Vertical => cx.begin_row(Width::Fix(self._calc_pos), Height::Fill),
        };
    }

    pub fn mid_draw(&mut self, cx: &mut Cx) {
        cx.end_row();
        let rect = cx.get_box_rect();
        let origin = cx.get_box_origin();

        match self.axis {
            Axis::Horizontal => {
                cx.set_draw_pos(Vec2 { x: origin.x, y: origin.y + self._calc_pos });
                self.split_view.begin_view(cx, LayoutSize::new(Width::Fix(rect.size.x), Height::Fix(self.split_size)));
                self.bg.draw(
                    cx,
                    Rect { pos: vec2(0., 0.), size: vec2(rect.size.x, self.split_size) }.translate(cx.get_box_origin()),
                    Vec4::default(),
                );
                self.split_view.end_view(cx);
                cx.set_draw_pos(Vec2 { x: origin.x, y: origin.y + self._calc_pos + self.split_size });
            }
            Axis::Vertical => {
                cx.set_draw_pos(Vec2 { x: origin.x + self._calc_pos, y: origin.y });
                self.split_view.begin_view(cx, LayoutSize::new(Width::Fix(self.split_size), Height::Fix(rect.size.y)));
                self.bg.draw(
                    cx,
                    Rect { pos: vec2(0., 0.), size: vec2(self.split_size, rect.size.y) }.translate(cx.get_box_origin()),
                    Vec4::default(),
                );
                self.split_view.end_view(cx);
                cx.set_draw_pos(Vec2 { x: origin.x + self._calc_pos + self.split_size, y: origin.y });
            }
        };
        cx.begin_row(Width::Fill, Height::Fill);
    }

    pub fn end_draw(&mut self, cx: &mut Cx) {
        cx.end_row();
        // draw the splitter in the middle of the box
        let rect = cx.get_box_rect();

        self.animator.draw(cx, ANIM_DEFAULT);
        self.animate(cx);

        match self.axis {
            Axis::Horizontal => {
                self._drag_max_pos = rect.size.y;
            }
            Axis::Vertical => {
                self._drag_max_pos = rect.size.x;
            }
        };
    }
}
