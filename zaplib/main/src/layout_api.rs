//! This is a public API for layouting: laying out different widgets on the screen.
//!
//! At the core you can imagine the box as having a position ([`CxLayoutBox::pos`]), a "sandbox" that
//! it can move in (delineated by [`CxLayoutBox::origin`] and [`CxLayoutBox::width`] and [`CxLayoutBox::height`]).
//!
//! Its movement is determined primarily by the [`Layout`] that you pass in, and you can modify it
//! ad-hoc by calling various functions.
//!
//! Boxes can be nested, so we have a stack of boxes in [`Cx::layout_boxes`]. The last [`CxLayoutBox`] on
//! the stack is the "current" or "active" box. When you call [`Cx::end_typed_box`], the last box's
//! "sandbox" [`Rect`] will be used to move the draw position of the parent box.
//!
//! A core philosophy of the box model is its simplicity and speed, by having only a single pass
//! to do layouting. Contrast this with systems like [CSS Flexbox](https://en.wikipedia.org/wiki/CSS_Flexible_Box_Layout),
//! which use a constraint satisfaction system to lay out your widgets. Instead, we make a single
//! pass, but do sometimes shift over individual elements after the fact, typically using
//! [`Cx::layout_box_align_list`]. When doing this we can regard it as a "1.5-pass" rendering. Currently
//! we have to go through every individual element if we want to move it, but in the future we could
//! exploit groupings of elements in [`View`]s and [`DrawCall`]s, and set uniforms on them.
//!
//! TODO(JP): The way the boxes move around is quite confusing in a lot of cases! This model
//! probably requires a complete rework. We can take inspiration from other layouting systems (e.g.
//! the [CSS box model](https://developer.mozilla.org/en-US/docs/Learn/CSS/Building_blocks/The_box_model))

use crate::*;

impl Cx {
    /// Starts the box that has it elements layed out horizontally (as a row)
    pub fn begin_row(&mut self, width: Width, height: Height) {
        self.begin_typed_box(
            CxBoxType::Row,
            Layout { direction: Direction::Right, layout_size: LayoutSize { width, height }, ..Layout::default() },
        );
    }

    /// Ends the current block that was opened by [`Cx::begin_row`].
    /// Returns a [`Rect`] representing the overall area of that row
    pub fn end_row(&mut self) -> Rect {
        self.end_typed_box(CxBoxType::Row)
    }

    /// Starts the box that has it elements layed out vertically (as a column)
    pub fn begin_column(&mut self, width: Width, height: Height) {
        self.begin_typed_box(
            CxBoxType::Column,
            Layout { direction: Direction::Down, layout_size: LayoutSize { width, height }, ..Layout::default() },
        );
    }

    /// Ends the current block that was opened by [`Cx::begin_column`].
    /// Returns a [`Rect`] representing the overall area of that column
    pub fn end_column(&mut self) -> Rect {
        self.end_typed_box(CxBoxType::Column)
    }

    /// Starts alignment element that fills all remaining space by y axis and centers content by it
    pub fn begin_center_y_align(&mut self) {
        let parent = self.layout_boxes.last().unwrap();
        let layout_box = CxLayoutBox {
            align_list_x_start_index: self.layout_box_align_list.len(),
            align_list_y_start_index: self.layout_box_align_list.len(),
            origin: parent.pos,
            pos: parent.pos,
            // fills out all remaining space by y axis
            layout: Layout { layout_size: LayoutSize { height: Height::Fill, ..parent.layout.layout_size }, ..parent.layout },
            biggest: 0.0,
            bound_right_bottom: Vec2 { x: std::f32::NEG_INFINITY, y: std::f32::NEG_INFINITY },
            width: self.get_width_left(),
            height: self.get_height_left(),
            abs_size: parent.abs_size,
            box_type: CxBoxType::CenterYAlign,
            available_width: parent.get_width_left(),
            available_height: parent.get_height_left(),
        };
        self.layout_boxes.push(layout_box);
    }

    pub fn end_center_y_align(&mut self) {
        self.assert_last_box_type_matches(CxBoxType::CenterYAlign);

        let layout_box = self.layout_boxes.pop().unwrap();
        let dy = Cx::compute_align_box_y(&layout_box, AlignY::CENTER);
        let align_start = layout_box.align_list_y_start_index;
        self.do_align_y(dy, align_start);

        let parent = self.layout_boxes.last_mut().unwrap();
        // TODO(Dmitry): communicating only few updates to parent for now. It's possible we need more.
        parent.bound_right_bottom.x = parent.bound_right_bottom.x.max(layout_box.bound_right_bottom.x);
        parent.pos = layout_box.pos;
    }

    /// Starts alignment element that fills all remaining space in box and centers content by x and y
    pub fn begin_center_x_and_y_align(&mut self) {
        let parent = self.layout_boxes.last().unwrap();
        let layout_box = CxLayoutBox {
            align_list_x_start_index: self.layout_box_align_list.len(),
            align_list_y_start_index: self.layout_box_align_list.len(),
            origin: parent.pos,
            pos: parent.pos,
            // fills out all remaining space by both axis
            layout: Layout { layout_size: LayoutSize { width: Width::Fill, height: Height::Fill }, ..parent.layout },
            biggest: 0.0,
            bound_right_bottom: Vec2 { x: std::f32::NEG_INFINITY, y: std::f32::NEG_INFINITY },
            width: self.get_width_left(),
            height: self.get_height_left(),
            abs_size: parent.abs_size,
            box_type: CxBoxType::CenterXYAlign,
            available_width: parent.get_width_left(),
            available_height: parent.get_height_left(),
        };
        self.layout_boxes.push(layout_box);
    }

    pub fn end_center_x_and_y_align(&mut self) {
        self.assert_last_box_type_matches(CxBoxType::CenterXYAlign);
        let layout_box = self.layout_boxes.pop().unwrap();

        let dx = Cx::compute_align_box_x(&layout_box, AlignX::CENTER);
        self.do_align_x(dx, layout_box.align_list_x_start_index);

        let dy = Cx::compute_align_box_y(&layout_box, AlignY::CENTER);
        self.do_align_y(dy, layout_box.align_list_y_start_index);

        // TODO(Dmitry): we are not communicating any changes back to parent since we are filling all remaining place
        // it's possible this breaks in some cases
    }

    // Start new box that will be on the bottom by y axis
    pub fn begin_bottom_box(&mut self) {
        let parent = self.layout_boxes.last().unwrap();
        let layout_box = CxLayoutBox {
            align_list_x_start_index: self.layout_box_align_list.len(),
            align_list_y_start_index: self.layout_box_align_list.len(),
            origin: parent.pos,
            pos: parent.pos,
            layout: parent.layout,
            biggest: 0.0,
            bound_right_bottom: Vec2 { x: std::f32::NEG_INFINITY, y: std::f32::NEG_INFINITY },
            width: parent.width,
            height: parent.height,
            abs_size: parent.abs_size,
            box_type: CxBoxType::BottomBox,
            available_width: parent.get_width_left(),
            available_height: parent.get_height_left(),
        };
        self.layout_boxes.push(layout_box);
    }

    pub fn end_bottom_box(&mut self) {
        self.assert_last_box_type_matches(CxBoxType::BottomBox);

        let layout_box = self.layout_boxes.pop().unwrap();
        let parent = self.layout_boxes.last_mut().unwrap();

        let drawn_height = layout_box.bound_right_bottom.y - layout_box.origin.y;
        let last_y = parent.origin.y + parent.available_height;
        let dy = last_y - layout_box.bound_right_bottom.y;
        // update parent
        parent.available_height -= drawn_height;
        parent.pos = layout_box.origin;
        parent.bound_right_bottom.x = parent.bound_right_bottom.x.max(layout_box.bound_right_bottom.x);
        parent.bound_right_bottom.y = last_y;

        let align_start = layout_box.align_list_y_start_index;
        self.do_align_y(dy, align_start);
    }

    /// Starts alignment element that fills all remaining space by x axis and centers content by it
    pub fn begin_center_x_align(&mut self) {
        let parent = self.layout_boxes.last().unwrap();
        let layout_box = CxLayoutBox {
            align_list_x_start_index: self.layout_box_align_list.len(),
            align_list_y_start_index: self.layout_box_align_list.len(),
            origin: parent.pos,
            pos: parent.pos,
            // fills out all remaining space by x axis
            layout: Layout { layout_size: LayoutSize { width: Width::Fill, ..parent.layout.layout_size }, ..parent.layout },
            biggest: 0.0,
            bound_right_bottom: Vec2 { x: std::f32::NEG_INFINITY, y: std::f32::NEG_INFINITY },
            width: self.get_width_left(),
            height: self.get_height_left(),
            abs_size: parent.abs_size,
            box_type: CxBoxType::CenterXAlign,
            available_width: parent.get_width_left(),
            available_height: parent.get_height_left(),
        };
        self.layout_boxes.push(layout_box);
    }

    pub fn end_center_x_align(&mut self) {
        self.assert_last_box_type_matches(CxBoxType::CenterXAlign);

        let layout_box = self.layout_boxes.pop().unwrap();
        let dx = Cx::compute_align_box_x(&layout_box, AlignX::CENTER);
        let align_start = layout_box.align_list_x_start_index;
        self.do_align_x(dx, align_start);

        let parent = self.layout_boxes.last_mut().unwrap();
        // TODO(Dmitry): communicating only few updates to parent for now. It's possible we need more.
        parent.bound_right_bottom.y = parent.bound_right_bottom.y.max(layout_box.bound_right_bottom.y);
        parent.pos = layout_box.pos;
    }

    /// Start new box that will be on the right by x axis
    pub fn begin_right_box(&mut self) {
        let parent = self.layout_boxes.last().unwrap();
        let layout_box = CxLayoutBox {
            align_list_x_start_index: self.layout_box_align_list.len(),
            align_list_y_start_index: self.layout_box_align_list.len(),
            origin: parent.pos,
            pos: parent.pos,
            layout: parent.layout,
            biggest: 0.0,
            bound_right_bottom: Vec2 { x: std::f32::NEG_INFINITY, y: std::f32::NEG_INFINITY },
            width: parent.width,
            height: parent.height,
            abs_size: parent.abs_size,
            box_type: CxBoxType::RightBox,
            available_width: parent.get_width_left(),
            available_height: parent.get_height_left(),
        };
        self.layout_boxes.push(layout_box);
    }

    pub fn end_right_box(&mut self) {
        self.assert_last_box_type_matches(CxBoxType::RightBox);

        let layout_box = self.layout_boxes.pop().unwrap();
        let parent = self.layout_boxes.last_mut().unwrap();

        let drawn_width = layout_box.bound_right_bottom.x - layout_box.origin.x;
        let last_x = parent.origin.x + parent.available_width;
        let dx = last_x - layout_box.bound_right_bottom.x;
        // update parent
        parent.available_width -= drawn_width;
        parent.pos = layout_box.origin;
        parent.bound_right_bottom.x = last_x;
        parent.bound_right_bottom.y = parent.bound_right_bottom.y.max(layout_box.bound_right_bottom.y);

        let align_start = layout_box.align_list_x_start_index;
        self.do_align_x(dx, align_start);
    }

    /// Starts a new box that adds padding to current box context
    pub fn begin_padding_box(&mut self, padding: Padding) {
        let parent = self.layout_boxes.last().expect("Using padding_box without parent is not supported");
        let direction = parent.layout.direction;
        self.begin_typed_box(
            CxBoxType::PaddingBox,
            Layout {
                direction,
                layout_size: LayoutSize { width: Width::Compute, height: Height::Compute },
                padding,
                ..Layout::default()
            },
        );
    }

    /// Ends the current box that was opened by [`Cx::begin_padding_box`]
    pub fn end_padding_box(&mut self) -> Rect {
        self.end_typed_box(CxBoxType::PaddingBox)
    }

    /// Starts new box that is absolutely positioned at (0, 0) coordinate
    pub fn begin_absolute_box(&mut self) {
        self.begin_typed_box(CxBoxType::AbsoluteBox, Layout { absolute: true, ..Layout::default() });
    }

    /// Ends the current box that was opened by [`Cx::begin_absolute_box`]
    pub fn end_absolute_box(&mut self) {
        self.end_typed_box(CxBoxType::AbsoluteBox);
    }

    /// Starts new box that is wrapping its content inside.
    /// This is defined in terms of child boxes (e.g. if any of the immediately nested boxes
    /// goes beyond the bounds, it would be wrapped to new line).
    /// This is only supported for horizontal direction.
    /// Note: text has its own wrapping mechanism via [`TextInsProps::wrapping`].
    pub fn begin_wrapping_box(&mut self) {
        let parent = self.layout_boxes.last().expect("Using wrapping_box without parent is not supported");
        let direction = parent.layout.direction;
        assert_eq!(direction, Direction::Right, "Wrapping is only supported for Direction::Right");
        self.begin_typed_box(
            CxBoxType::WrappingBox,
            Layout {
                direction,
                line_wrap: LineWrap::Overflow,
                layout_size: LayoutSize { width: Width::Compute, height: Height::Compute },
                ..Layout::default()
            },
        );
    }

    /// Ends the current box that was opened by [`Cx::begin_wrapping_box`]
    pub fn end_wrapping_box(&mut self) {
        self.end_typed_box(CxBoxType::WrappingBox);
    }

    /// Returns the full rect corresponding to current box.
    /// It uses all available_width/height plus padding.
    /// Note that these are the inherent dimensions of the [`CxLayoutBox`], not
    /// what the [`CxLayoutBox`] has walked so far. See [`Cx::get_box_bounds`] for that.
    pub fn get_box_rect(&self) -> Rect {
        if let Some(layout_box) = self.layout_boxes.last() {
            return Rect {
                pos: layout_box.origin,
                size: vec2(
                    layout_box.available_width + layout_box.layout.padding.r,
                    layout_box.available_height + layout_box.layout.padding.b,
                ),
            };
        };
        Rect::default()
    }

    /// Get some notion of the width that is "left" for the current [`CxLayoutBox`].
    ///
    /// See also [`Cx::get_width_total`].
    pub fn get_width_left(&self) -> f32 {
        if let Some(layout_box) = self.layout_boxes.last() {
            layout_box.get_width_left()
        } else {
            0.
        }
    }

    /// Get some notion of the total width of the current box. If the width
    /// is well defined, then we return it. If it's computed, then we return the
    /// bound (including padding) of how much we've drawn so far. And if we haven't
    /// drawn anything, we return 0.
    pub fn get_width_total(&self) -> f32 {
        if let Some(layout_box) = self.layout_boxes.last() {
            let nan_val = max_zero_keep_nan(layout_box.width);
            if nan_val.is_nan() {
                // if we are a computed width, if some value is known, use that
                if layout_box.bound_right_bottom.x != std::f32::NEG_INFINITY {
                    return layout_box.bound_right_bottom.x - layout_box.origin.x + layout_box.layout.padding.r;
                } else {
                    return 0.;
                }
            }
            return nan_val;
        }
        0.
    }

    /// Get some notion of the height that is "left" for the current [`CxLayoutBox`].
    ///
    /// See also [`Cx::get_height_total`].
    pub fn get_height_left(&self) -> f32 {
        if let Some(layout_box) = self.layout_boxes.last() {
            layout_box.get_height_left()
        } else {
            0.
        }
    }

    /// Get some notion of the total height of the current box. If the height
    /// is well defined, then we return it. If it's computed, then we return the
    /// bound (including padding) of how much we've drawn so far. And if we haven't
    /// drawn anything, we return 0.
    pub fn get_height_total(&self) -> f32 {
        if let Some(layout_box) = self.layout_boxes.last() {
            let nan_val = max_zero_keep_nan(layout_box.height);
            if nan_val.is_nan() {
                // if we are a computed height, if some value is known, use that
                if layout_box.bound_right_bottom.y != std::f32::NEG_INFINITY {
                    return layout_box.bound_right_bottom.y - layout_box.origin.y + layout_box.layout.padding.b;
                } else {
                    return 0.;
                }
            }
            return nan_val;
        }
        0.
    }

    /// Get the bounds of what the box has *actually* moved (not just its
    /// inherent width/height as given by [`Cx::get_box_rect`]), including any padding that the
    /// layout of the current box specifies.
    pub fn get_box_bounds(&self) -> Vec2 {
        if let Some(layout_box) = self.layout_boxes.last() {
            return Vec2 {
                x: if layout_box.bound_right_bottom.x < 0. { 0. } else { layout_box.bound_right_bottom.x }
                    + layout_box.layout.padding.r
                    - layout_box.origin.x,
                y: if layout_box.bound_right_bottom.y < 0. { 0. } else { layout_box.bound_right_bottom.y }
                    + layout_box.layout.padding.b
                    - layout_box.origin.y,
            };
        }
        Vec2::default()
    }

    /// Same as [`Cx::get_box_rect().pos`].
    ///
    /// TODO(JP): Do we really need two different methods to get to the same data?
    pub fn get_box_origin(&self) -> Vec2 {
        if let Some(layout_box) = self.layout_boxes.last() {
            return layout_box.origin;
        }
        Vec2::default()
    }

    /// Get the current [`CxLayoutBox::pos`] in absolute coordinates.
    ///
    /// TODO(JP): Only used in two places currently; do we really need this?
    pub fn get_draw_pos(&self) -> Vec2 {
        if let Some(layout_box) = self.layout_boxes.last() {
            layout_box.pos
        } else {
            Vec2::default()
        }
    }

    /// Adds Box to current [`CxLayoutBox`], returning a [`Rect`] of its size
    pub fn add_box(&mut self, layout_size: LayoutSize) -> Rect {
        self.move_box_with_old(layout_size, None)
    }

    /// Manually change [`CxLayoutBox::pos`]. Warning! Does not update [`CxLayoutBox::bound_right_bottom`],
    /// like [`Cx::add_box`] does; might result in unexpected behavior.
    ///
    /// TODO(JP): Should we delete this and just always use [`Cx::add_box`] instead?
    pub fn move_draw_pos(&mut self, dx: f32, dy: f32) {
        if let Some(layout_box) = self.layout_boxes.last_mut() {
            layout_box.pos.x += dx;
            layout_box.pos.y += dy;
        }
    }

    /// Manually change [`CxLayoutBox::pos`]. Warning! Does not update [`CxLayoutBox::bound_right_bottom`],
    /// like [`Cx::add_box`] does; might result in unexpected behavior.
    ///
    /// TODO(JP): Should we delete this and just always use [`Cx::add_box`] instead?
    pub fn set_draw_pos(&mut self, pos: Vec2) {
        if let Some(layout_box) = self.layout_boxes.last_mut() {
            layout_box.pos = pos
        }
    }

    /// Explicitly move the current [`CxLayoutBox`] to a new line.
    ///
    /// TODO(JP): Mostly relevant for [`Direction::Right`], should we just disable
    /// this for [`Direction::Down`] to avoid confusion?
    pub fn draw_new_line(&mut self) {
        if let Some(layout_box) = self.layout_boxes.last_mut() {
            assert_eq!(layout_box.layout.direction, Direction::Right, "draw_new_line with Direction::Down is not supported");
            layout_box.pos.x = layout_box.origin.x + layout_box.layout.padding.l;
            layout_box.pos.y += layout_box.biggest;
            layout_box.biggest = 0.0;
        }
    }

    /// [`Cx::draw_new_line`] but allows setting a minimum height for the line.
    ///
    /// TODO(JP): Should we instead include `min_height` in [`Layout`]?
    pub fn draw_new_line_min_height(&mut self, min_height: f32) {
        if let Some(layout_box) = self.layout_boxes.last_mut() {
            assert_eq!(
                layout_box.layout.direction,
                Direction::Right,
                "draw_new_line_min_height with Direction::Down is not supported"
            );
            layout_box.pos.x = layout_box.origin.x + layout_box.layout.padding.l;
            layout_box.pos.y += layout_box.biggest.max(min_height);
            layout_box.biggest = 0.0;
        }
    }
}
