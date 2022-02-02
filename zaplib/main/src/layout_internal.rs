//! Layout system. 🐢

use crate::debug_log::DebugLog;
use crate::*;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum CxBoxType {
    Normal,
    RightBox,
    BottomBox,
    CenterXAlign,
    CenterYAlign,
    CenterXYAlign,
    PaddingBox,
    Row,
    Column,
    AbsoluteBox,
    WrappingBox,
    View,
}

impl Default for CxBoxType {
    fn default() -> CxBoxType {
        CxBoxType::Normal
    }
}

/// A [`CxLayoutBox`] is an internal implementation details for layouting system.
#[derive(Clone, Default, Debug)]
pub(crate) struct CxLayoutBox {
    /// The layout that is associated directly with this box, which determines a lot of its
    /// behavior.
    pub(crate) layout: Layout,

    /// The index within Cx::layout_box_align_list, which contains all the things that we draw within this
    /// box, and which needs to get aligned at some point. We have a separate list for x/y
    /// because you can manually trigger an alignment (aside from it happening automatically at the
    /// end), which resets this list to no longer align those areas again.
    pub(crate) align_list_x_start_index: usize,

    /// Same as [`CxLayoutBox::align_list_x_start_index`] but for vertical alignment.
    pub(crate) align_list_y_start_index: usize,

    /// The current position of the current box.
    /// This is an absolute position, and starts out at [`CxLayoutBox::origin`]
    /// plus padding.
    pub(crate) pos: Vec2,

    /// The origin of the current box. Starts off at the parent's box [`CxLayoutBox::pos`]
    pub(crate) origin: Vec2,

    /// The inherent width of the current box's walking area. Is [`f32::NAN`] if the width is computed,
    /// and can get set explicitly later.
    pub(crate) width: f32,

    /// The inherent height of the current box's walking area. Is [`f32::NAN`] if the height is computed,
    /// and can get set explicitly later.
    pub(crate) height: f32,

    /// Seems to only be used to be passed down to child boxes, so if one of them gets an absolute
    /// origin passed in, we can just use the entire remaining absolute canvas as the width/height.
    ///
    /// TODO(JP): seems pretty unnecessary; why not just grab this field from the current [`Pass`
    /// directly if necessary? Or just always have the caller pass it in (they can take it from the
    /// current [`Pass`] if they want)?
    pub(crate) abs_size: Vec2,

    /// Keeps track of the bottom right corner where we have walked so far, including the width/height
    /// of the walk, whereas [`CxLayoutBox::pos`] stays in the top left position of what we have last drawn.
    ///
    /// TODO(JP): [`CxLayoutBox::pos`] and [`CxLayoutBox::bound_right_bottom`] can (and seem to regularly and intentionally do)
    /// get out of sync, which makes things more confusing.
    pub(crate) bound_right_bottom: Vec2,

    /// We keep track of the [`LayoutSize`] with the greatest height (or width, when walking down), so that
    /// we know how much to move the box's y-position when wrapping to the next line. When
    /// wrapping to the next line, this value is reset back to 0.
    ///
    /// See also [`Padding`].
    pub(crate) biggest: f32,

    /// Used for additional checks that enclosing box match opening ones
    pub(crate) box_type: CxBoxType,

    /// Available width for the content of the box, starting from box origin, minus right padding.
    /// This is different from [`CxLayoutBox::width`] which is box outer width.
    /// For example, for Width::Compute boxes width would be [`f32::NAN`] as this needs to be computed,
    /// but available_width is defined until the bounds of parent.
    /// This is capped at 0 if the content already overflows the bounds.
    pub(crate) available_width: f32,

    /// Available height for the content of the box, starting from box origin, minus bottom padding.
    /// This is different from [`CxLayoutBox::height`] which is box outer height.
    /// For example, for Height::Compute boxes height would be [`f32::NAN`] as this needs to be computed,
    /// but available_height is defined until the bounds of parent/
    /// This is capped at 0 if the content already overflows the bounds.
    pub(crate) available_height: f32,
}

impl CxLayoutBox {
    /// Returns how much available_width is "left" for current box,
    /// i.e. distance from current box x position until the right bound
    pub(crate) fn get_width_left(&self) -> f32 {
        (self.origin.x + self.available_width - self.pos.x).max(0.)
    }

    /// Returns how much available_height is "left" for current box
    /// i.e. distance from current box y position until the bottom bound
    pub(crate) fn get_height_left(&self) -> f32 {
        (self.origin.y + self.available_height - self.pos.y).max(0.)
    }
}

impl Cx {
    /// Begin a new [`CxLayoutBox`] with a given [`Layout`]. This new [`CxLayoutBox`] will be added to the
    /// [`Cx::layout_boxes`] stack.
    pub(crate) fn begin_typed_box(&mut self, box_type: CxBoxType, layout: Layout) {
        if !self.in_redraw_cycle {
            panic!("calling begin_typed_box outside of redraw cycle is not possible!");
        }
        if layout.direction == Direction::Down && layout.line_wrap != LineWrap::None {
            panic!("Direction down with line wrapping is not supported");
        }

        // fetch origin and size from parent
        let (mut origin, mut abs_size) = if let Some(parent) = self.layout_boxes.last() {
            (Vec2 { x: parent.pos.x, y: parent.pos.y }, parent.abs_size)
        } else {
            assert!(layout.absolute);
            assert!(layout.abs_size.is_some());
            (Vec2 { x: 0., y: 0. }, Vec2::default())
        };

        // see if layout overrode size
        if let Some(layout_abs_size) = layout.abs_size {
            abs_size = layout_abs_size;
        }

        let width;
        let height;
        if layout.absolute {
            // absolute overrides origin to start from (0, 0)
            origin = vec2(0.0, 0.0);
            // absolute overrides the computation of width/height to use the parent absolute
            width = self.eval_absolute_width(&layout.layout_size.width, abs_size.x);
            height = self.eval_absolute_height(&layout.layout_size.height, abs_size.y);
        } else {
            width = self.eval_width(&layout.layout_size.width);
            height = self.eval_height(&layout.layout_size.height);
        }

        let pos = Vec2 { x: origin.x + layout.padding.l, y: origin.y + layout.padding.t };

        let available_width =
            (self.eval_available_width(&layout.layout_size.width, layout.absolute, abs_size) - layout.padding.r).max(0.);
        let available_height =
            (self.eval_available_height(&layout.layout_size.height, layout.absolute, abs_size) - layout.padding.b).max(0.);

        // By induction property this values should never be NaN
        assert!(!available_width.is_nan());
        assert!(!available_height.is_nan());

        let layout_box = CxLayoutBox {
            align_list_x_start_index: self.layout_box_align_list.len(),
            align_list_y_start_index: self.layout_box_align_list.len(),
            origin,
            pos,
            layout,
            biggest: 0.0,
            bound_right_bottom: Vec2 { x: std::f32::NEG_INFINITY, y: std::f32::NEG_INFINITY },
            width,
            height,
            abs_size,
            box_type,
            available_height,
            available_width,
        };

        self.layout_boxes.push(layout_box);
    }

    /// Pop the current [`CxLayoutBox`] from the [`Cx::layout_boxes`] stack, returning a [`Rect`] that the box walked
    /// during its lifetime. The parent [`CxLayoutBox`] will be made to walk this [`Rect`].
    pub(crate) fn end_typed_box(&mut self, box_type: CxBoxType) -> Rect {
        self.assert_last_box_type_matches(box_type);
        self.end_last_box_unchecked()
    }

    pub(crate) fn assert_last_box_type_matches(&self, box_type: CxBoxType) {
        let layout_box = self.layout_boxes.last().unwrap();
        if layout_box.box_type != box_type {
            panic!("Closing box type doesn't match! Expected: {:?}, found: {:?}", box_type, layout_box.box_type);
        }
    }

    /// Similar to [`Cx::end_typed_box`], but doesn't do any matching checks on the box. Use at your own risk!
    fn end_last_box_unchecked(&mut self) -> Rect {
        let old = self.layout_boxes.pop().unwrap();
        let w = if old.width.is_nan() {
            // when nesting Fill box inside Compute the former would have nan width
            if old.layout.layout_size.width == Width::Fill {
                // use all available width + padding
                Width::Fix(old.available_width + old.layout.padding.r)
            } else if old.bound_right_bottom.x == std::f32::NEG_INFINITY {
                // nothing happened, use padding
                Width::Fix(old.layout.padding.l + old.layout.padding.r)
            } else {
                // use the bounding box
                Width::Fix(max_zero_keep_nan(old.bound_right_bottom.x - old.origin.x + old.layout.padding.r))
            }
        } else {
            Width::Fix(old.width)
        };

        let h = if old.height.is_nan() {
            // when nesting Fill box inside Compute the former would have nan height
            if old.layout.layout_size.height == Height::Fill {
                // use all available height + padding
                Height::Fix(old.available_height + old.layout.padding.b)
            } else if old.bound_right_bottom.y == std::f32::NEG_INFINITY {
                // nothing happened use the padding
                Height::Fix(old.layout.padding.t + old.layout.padding.b)
            } else {
                // use the bounding box
                Height::Fix(max_zero_keep_nan(old.bound_right_bottom.y - old.origin.y + old.layout.padding.b))
            }
        } else {
            Height::Fix(old.height)
        };

        let rect = {
            // when a box is absolutely positioned don't walk the parent
            if old.layout.absolute {
                let w = if let Width::Fix(vw) = w { vw } else { 0. };
                let h = if let Height::Fix(vh) = h { vh } else { 0. };
                Rect { pos: vec2(0., 0.), size: vec2(w, h) }
            } else {
                self.move_box_with_old(LayoutSize { width: w, height: h }, Some(&old))
            }
        };
        self.debug_logs.push(DebugLog::EndBox { rect });
        rect
    }

    /// Move the box with the given [`LayoutSize`]
    ///
    /// Returns a [`Rect`] containing the area that the box moved
    ///
    /// TODO(JP): This `old_box` stuff is a bit awkward and only used for the
    /// alignment stuff at the end. We can probably structure this in a nicer way.
    pub(crate) fn move_box_with_old(&mut self, layout_size: LayoutSize, old_box: Option<&CxLayoutBox>) -> Rect {
        let mut align_dx = 0.0;
        let mut align_dy = 0.0;

        // TODO(JP): This seems a bit weird: you can technically pass in Width::Compute, which would
        // return a NaN for `w`, but that doesn't make much sense when you explicitly do a walk.
        // It looks like it's assumed that that never gets passed in here, but it would be better to
        // verify that.
        // NOTE(Dmitry): now this methods will panic when receiving Compute sizes.
        // We can probably express this better in type system, but this is good enough for now.
        let w = self.eval_walking_width(&layout_size.width);
        let h = self.eval_walking_height(&layout_size.height);

        let ret = if let Some(layout_box) = self.layout_boxes.last_mut() {
            let old_pos = match layout_box.layout.direction {
                Direction::Right => {
                    match layout_box.layout.line_wrap {
                        LineWrap::Overflow => {
                            if (layout_box.pos.x + w) > (layout_box.origin.x + layout_box.available_width) + 0.01 {
                                // what is the move delta.
                                let old_x = layout_box.pos.x;
                                let old_y = layout_box.pos.y;
                                layout_box.pos.x = layout_box.origin.x + layout_box.layout.padding.l;
                                layout_box.pos.y += layout_box.biggest;
                                layout_box.biggest = 0.0;
                                align_dx = layout_box.pos.x - old_x;
                                align_dy = layout_box.pos.y - old_y;
                            }
                        }
                        LineWrap::None => {}
                    }

                    let old_pos = layout_box.pos;
                    // walk it normally
                    layout_box.pos.x += w;

                    // keep track of biggest item in the line
                    layout_box.biggest = layout_box.biggest.max(h);
                    old_pos
                }
                Direction::Down => {
                    let old_pos = layout_box.pos;
                    // walk it normally
                    layout_box.pos.y += h;

                    // keep track of biggest item in the line
                    layout_box.biggest = layout_box.biggest.max(w);
                    old_pos
                }
            };

            // update bounds
            let new_bound = old_pos + vec2(w, h);
            layout_box.bound_right_bottom = layout_box.bound_right_bottom.max(&new_bound);

            Rect { pos: old_pos, size: vec2(w, h) }
        } else {
            Rect { pos: vec2(0.0, 0.0), size: vec2(w, h) }
        };

        if align_dx != 0.0 {
            if let Some(old_box) = old_box {
                self.move_by_x(align_dx, old_box.align_list_x_start_index);
            }
        };
        if align_dy != 0.0 {
            if let Some(old_box) = old_box {
                self.move_by_y(align_dy, old_box.align_list_y_start_index);
            }
        };

        ret
    }

    /// Actually perform a horizontal movement of items in [`Cx::layout_box_align_list`], but only for positive dx
    pub(crate) fn do_align_x(&mut self, dx: f32, align_start: usize) {
        if dx < 0. {
            // do only forward moving alignment
            // backwards alignment could happen if the size of content became larger than the container
            // in which case the alignment is not well defined
            return;
        }
        self.move_by_x(dx, align_start)
    }

    /// Actually perform a horizontal movement of items in [`Cx::layout_box_align_list`].
    /// Unlike "do_align_x" negative moves can happen here because of wrapping behavior.
    ///
    /// TODO(JP): Should we move some of this stuff to [`Area`], where we already seem to do a bunch
    /// of rectangle and position calculations?
    fn move_by_x(&mut self, dx: f32, align_start: usize) {
        let dx = (dx * self.current_dpi_factor).floor() / self.current_dpi_factor;
        for i in align_start..self.layout_box_align_list.len() {
            let align_item = &self.layout_box_align_list[i];
            match align_item {
                Area::InstanceRange(inst) => {
                    let cxview = &mut self.views[inst.view_id];
                    let draw_call = &mut cxview.draw_calls[inst.draw_call_id];
                    let sh = &self.shaders[draw_call.shader_id];
                    for i in 0..inst.instance_count {
                        if let Some(rect_pos) = sh.mapping.rect_instance_props.rect_pos {
                            draw_call.instances[inst.instance_offset + rect_pos + i * sh.mapping.instance_props.total_slots] +=
                                dx;
                        }
                    }
                }
                Area::View(view_area) => {
                    let cxview = &mut self.views[view_area.view_id];
                    cxview.rect.pos.x += dx;
                }
                // TODO(JP): Would be nice to implement this for [`Align::View`], which would
                // probably require some offset field on [`CxView`] that gets used during rendering.
                _ => unreachable!(),
            }
        }
    }

    /// Actually perform a vertical movement of items in [`Cx::layout_box_align_list`], but only for positive dy
    pub(crate) fn do_align_y(&mut self, dy: f32, align_start: usize) {
        if dy < 0. {
            // do only forward moving alignment
            // backwards alignment could happen if the size of content became larger than the container
            // in which case the alignment is not well defined
            return;
        }
        self.move_by_y(dy, align_start);
    }

    /// Actually perform a vertical movement of items in [`Cx::layout_box_align_list`].
    /// Unlike "do_align_y" negative moves can happen here because of wrapping behavior.
    ///
    /// TODO(JP): Should we move some of this stuff to [`Area`], where we already seem to do a bunch
    /// of rectangle and position calculations?
    fn move_by_y(&mut self, dy: f32, align_start: usize) {
        let dy = (dy * self.current_dpi_factor).floor() / self.current_dpi_factor;
        for i in align_start..self.layout_box_align_list.len() {
            let align_item = &self.layout_box_align_list[i];
            match align_item {
                Area::InstanceRange(inst) => {
                    let cxview = &mut self.views[inst.view_id];
                    let draw_call = &mut cxview.draw_calls[inst.draw_call_id];
                    let sh = &self.shaders[draw_call.shader_id];
                    for i in 0..inst.instance_count {
                        if let Some(rect_pos) = sh.mapping.rect_instance_props.rect_pos {
                            draw_call.instances
                                [inst.instance_offset + rect_pos + 1 + i * sh.mapping.instance_props.total_slots] += dy;
                        }
                    }
                }
                Area::View(view_area) => {
                    let cxview = &mut self.views[view_area.view_id];
                    cxview.rect.pos.y += dy;
                }
                // TODO(JP): Would be nice to implement this for `Align::View`, which would
                // probably require some offset field on `CxView` that gets used during rendering.
                _ => unreachable!(),
            }
        }
    }

    /// Returns how many pixels we should move over based on the [`AlignX`] ratio
    /// (which is between 0 and 1). We do this by looking at the bound
    /// ([`CxLayoutBox::bound_right_bottom`]) to see how much we have actually drawn, and how
    /// subtract that from the width of this box. That "remaining width" is
    /// then multiplied with the ratio. If there is no inherent width then this
    /// will return 0.
    pub(crate) fn compute_align_box_x(layout_box: &CxLayoutBox, align: AlignX) -> f32 {
        let AlignX(fx) = align;
        if fx > 0.0 {
            // TODO(Dmitry): check if we need use padding here
            let dx = fx
                * ((layout_box.available_width - (layout_box.layout.padding.l + layout_box.layout.padding.r))
                    - (layout_box.bound_right_bottom.x - (layout_box.origin.x + layout_box.layout.padding.l)));
            if dx.is_nan() {
                return 0.0;
            }
            dx
        } else {
            0.
        }
    }

    /// Returns how many pixels we should move over based on the [`AlignY`] ratio
    /// (which is between 0 and 1). We do this by looking at the bound
    /// ([`CxLayoutBox::bound_right_bottom`]) to see how much we have actually drawn, and how
    /// subtract that from the height of this box. That "remaining height" is
    /// then multiplied with the ratio. If there is no inherent height then this
    /// will return 0.
    pub(crate) fn compute_align_box_y(layout_box: &CxLayoutBox, align: AlignY) -> f32 {
        let AlignY(fy) = align;
        if fy > 0.0 {
            // TODO(Dmitry): check if we need use padding here
            let dy = fy
                * ((layout_box.available_height - (layout_box.layout.padding.t + layout_box.layout.padding.b))
                    - (layout_box.bound_right_bottom.y - (layout_box.origin.y + layout_box.layout.padding.t)));
            if dy.is_nan() {
                return 0.0;
            }
            dy
        } else {
            0.
        }
    }

    // TODO(Dmitry): simplify all the following eval functions
    fn eval_width(&self, width: &Width) -> f32 {
        match width {
            Width::Compute => std::f32::NAN,
            Width::Fix(v) => v.max(0.),
            Width::Fill => self.get_width_left(),
            Width::FillUntil(v) => self.get_width_left().min(*v),
        }
    }

    fn eval_absolute_width(&self, width: &Width, abs_size: f32) -> f32 {
        match width {
            Width::Compute => std::f32::NAN,
            Width::Fix(v) => max_zero_keep_nan(*v),
            Width::Fill => max_zero_keep_nan(abs_size),
            Width::FillUntil(v) => min_keep_nan(*v, abs_size),
        }
    }

    fn eval_walking_width(&self, width: &Width) -> f32 {
        match width {
            Width::Compute => panic!("Walking with Width:Compute is not supported"),
            Width::Fix(v) => v.max(0.),
            Width::Fill => self.get_width_left(),
            Width::FillUntil(v) => self.get_width_left().min(*v),
        }
    }

    fn eval_available_width(&self, width: &Width, absolute: bool, abs_size: Vec2) -> f32 {
        if absolute {
            return abs_size.x;
        }

        // Non-absolute layouts will always have parents
        let parent = self.layout_boxes.last().unwrap();
        match width {
            Width::Fix(v) => *v,
            Width::FillUntil(v) => parent.get_width_left().min(*v),
            Width::Compute | Width::Fill => parent.get_width_left(),
        }
    }

    fn eval_height(&self, height: &Height) -> f32 {
        match height {
            Height::Compute => std::f32::NAN,
            Height::Fix(v) => v.max(0.),
            Height::Fill => self.get_height_left(),
            Height::FillUntil(v) => self.get_height_left().min(*v),
        }
    }

    fn eval_absolute_height(&self, height: &Height, abs_size: f32) -> f32 {
        match height {
            Height::Compute => std::f32::NAN,
            Height::Fix(v) => v.max(0.),
            Height::Fill => max_zero_keep_nan(abs_size),
            Height::FillUntil(v) => min_keep_nan(*v, abs_size),
        }
    }

    fn eval_walking_height(&self, height: &Height) -> f32 {
        match height {
            Height::Compute => panic!("Walking with Height:Compute is not supported"),
            Height::Fix(v) => v.max(0.),
            Height::Fill => self.get_height_left(),
            Height::FillUntil(v) => self.get_height_left().min(*v),
        }
    }

    fn eval_available_height(&self, height: &Height, absolute: bool, abs_size: Vec2) -> f32 {
        if absolute {
            return abs_size.y;
        }
        // Non-absolute layouts will always have parents
        let parent = self.layout_boxes.last().unwrap();
        match height {
            Height::Fix(v) => *v,
            Height::FillUntil(v) => parent.get_height_left().min(*v),
            Height::Compute | Height::Fill => parent.get_height_left(),
        }
    }

    /// Add an `Area::InstanceRange` to the [`Cx::layout_box_align_list`], so that it will get aligned,
    /// e.g. when you call [`Cx::end_typed_box`].
    pub(crate) fn add_to_box_align_list(&mut self, area: Area) {
        match area {
            Area::InstanceRange(_) => self.layout_box_align_list.push(area),
            _ => panic!("Only Area::InstanceRange can be aligned currently"),
        }
    }
}

pub(crate) fn max_zero_keep_nan(v: f32) -> f32 {
    if v.is_nan() {
        v
    } else {
        f32::max(v, 0.0)
    }
}

pub(crate) fn min_keep_nan(a: f32, b: f32) -> f32 {
    if a.is_nan() || b.is_nan() {
        f32::NAN
    } else {
        f32::min(a, b)
    }
}
