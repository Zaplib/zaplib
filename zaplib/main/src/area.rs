//! "Pointer" into the draw tree.

use crate::*;

/// An [`Area`] can be thought of as a "pointer" into the draw tree. You typically
/// get one as the result of a draw command, like [`Cx::add_instances`],
/// or [`View::end_view`].
///
/// Note that an [`Area`] might point to an old element in the draw tree, so use
/// [`Area::is_valid`] to check if it points to the latest version.
///
/// You can use an [`Area`] pointer to write fields, e.g. using [`Area::get_slice_mut`],
/// [`Area::write_user_uniforms`], and so on. You
/// can also use it for checking if an event was fired on a certain part
/// of the draw tree (using [`Event::hits_pointer`], [`Cx::key_focus`], etc), and more.
///
/// TODO(JP): this can currently point to a [`View`]/[`CxView`] that isn't
/// actually in the draw tree anymore (ie. there the corresponding [`CxView`] is
/// never referenced in the draw tree), or which just doesn't exist at all any
/// more (ie. the [`View`] object has also been removed). There is currently no
/// was of telling if any of this is the case, since there is no "garbage
/// collection" of views. [`CxView`] just sticks around in [`Cx::views`] forever.
#[derive(Clone, Debug, Hash, PartialEq, Ord, PartialOrd, Eq, Copy)]
pub enum Area {
    /// A "null pointer", doesn't point to anything yet.
    Empty,
    /// See [`ViewArea`].
    View(ViewArea),
    /// See [`InstanceRangeArea`].
    InstanceRange(InstanceRangeArea),
}

impl Default for Area {
    fn default() -> Area {
        Area::Empty
    }
}

/// Pointer to a particular view in the draw tree, using a [`ViewArea::view_id`]. Note
/// that a [`View::view_id`] only gets set when it gets drawn.
///
/// See also [`Area`].
#[derive(Clone, Default, Hash, Ord, PartialOrd, Eq, Debug, PartialEq, Copy)]
pub struct ViewArea {
    /// Corresponds to [`View::view_id`] of the [`View`] this points to, which
    /// is the same as the index of [`Cx::views`] of the corresponding [`CxView`].
    pub(crate) view_id: usize,
    /// The [`Cx::redraw_id`] during which this [`Area`] was created. If it
    /// doesn't match the corresponding [`View::redraw_id`], then this pointer is
    /// stale; it likely wasn't properly updated with [`Cx::request_draw`].
    ///
    /// Note that if [`ViewArea::redraw_id`] doesn't match [`Cx::redraw_id`] then that doesn't
    /// necessarily mean that the pointer is stale, since instead rendering for
    /// the corresponding [`View`] could have been skipped (if nothing had
    /// changed).
    ///
    /// See also [`View::view_id`].
    ///
    /// TODO(JP): Is the [`ViewArea`] redundant, since it basically contains the
    /// same information as the [`View`] itself?
    pub(crate) redraw_id: u64,
}

/// Pointer to a part of a [`DrawCall`], e.g. from [`Cx::add_instances`]. This pointer
/// points to a range of instances, where the first index is indicated by
/// [`InstanceRangeArea::instance_offset`], and the size of the
/// range by [`InstanceRangeArea::instance_count`].
///
/// See also [`Area`].
#[derive(Clone, Default, Hash, Ord, PartialOrd, Eq, Debug, PartialEq, Copy)]
pub struct InstanceRangeArea {
    /// Corresponds to [`View::view_id`] of the [`View`] this points to, which
    /// is the same as the index of [`Cx::views`] of the corresponding [`CxView`].
    pub(crate) view_id: usize,
    /// Index of [`CxView::draw_calls`] of the corresponding [`DrawCall`].
    pub(crate) draw_call_id: usize,
    /// Offset in "slots"/nibbles/4 bytes from the start of [`DrawCall::instances`]
    /// to the first instance that this pointer describes.
    pub(crate) instance_offset: usize,
    /// Number of instances that this pointer describes.
    pub(crate) instance_count: usize,
    /// See [`ViewArea::redraw_id`].
    pub(crate) redraw_id: u64,
}

impl Area {
    /// Shorthand for `if let Area::Empty = area`.
    pub fn is_empty(&self) -> bool {
        if let Area::Empty = self {
            return true;
        }
        false
    }

    /// Check if this is an [`Area::InstanceRange`] that points to the first instance
    /// in its corresponding [`DrawCall`]. Useful for setting uniforms on a
    /// [`DrawCall`] only once, when handling the first instance.
    pub fn is_first_instance(&self) -> bool {
        match self {
            Area::InstanceRange(inst) => inst.instance_offset == 0,
            _ => false,
        }
    }

    /// Check if this [`Area`] points to valid data. Will return false for
    /// [`Area::Empty`], and if the [`View::redraw_id`] is different
    /// than the [`ViewArea::redraw_id`] or [`InstanceRangeArea::redraw_id`].
    ///
    /// TODO(JP): this will return [`true`] when the [`Area`] points to data that
    /// is not visible on the screen / when it's gone from the draw tree, or
    /// even when the original [`View`] object is gone. That's probably bad, or at
    /// least confusing. See [`Area`] for more information.
    pub(crate) fn is_valid(&self, cx: &Cx) -> bool {
        match self {
            Area::InstanceRange(inst) => {
                let cxview = &cx.views[inst.view_id];
                if cxview.redraw_id != inst.redraw_id {
                    return false;
                }
                true
            }
            Area::View(view_area) => {
                let cxview = &cx.views[view_area.view_id];
                if cxview.redraw_id != view_area.redraw_id {
                    return false;
                }
                true
            }
            _ => false,
        }
    }

    /// The scroll position of an [`Area`] is the cumulative offset of all scroll
    /// containers (compared to if there had not been scrolling at all).
    pub fn get_scroll_pos(&self, cx: &Cx) -> Vec2 {
        if !self.is_valid(cx) {
            panic!("get_scroll_pos was called for an invalid Area");
        }
        match self {
            Area::InstanceRange(inst) => {
                // Pull it directly out of the draw uniforms.
                let cxview = &cx.views[inst.view_id];
                let draw_call = &cxview.draw_calls[inst.draw_call_id];
                Vec2 { x: draw_call.draw_uniforms.draw_scroll_x, y: draw_call.draw_uniforms.draw_scroll_y }
            }
            Area::View(view_area) => {
                let cxview = &cx.views[view_area.view_id];
                cxview.parent_scroll
            }
            _ => unreachable!(),
        }
    }

    /// Returns the final screen [`Rect`] for the first instance of the [`Area`].
    ///
    /// TODO(JP): The "first instance" bit is confusing; in most (if not all)
    /// cases you'd want to get something that covers the entire [`Area`]. Maybe
    /// returning a single [`Rect`] isn't the right thing then, since the
    /// individual rectangles can be all over the place. We could return a [`Vec`]
    /// instead?
    ///
    /// TODO(JP): Specifically, this seems to return very weird values for
    /// [`crate::TextIns`] (only the first character, and offset to the bottom it seems).
    pub fn get_rect_for_first_instance(&self, cx: &Cx) -> Option<Rect> {
        if !self.is_valid(cx) {
            return None;
        }
        match self {
            Area::InstanceRange(inst) => {
                if inst.instance_count == 0 {
                    return None;
                }
                let cxview = &cx.views[inst.view_id];
                let draw_call = &cxview.draw_calls[inst.draw_call_id];
                assert!(!draw_call.instances.is_empty());
                let sh = &cx.shaders[draw_call.shader_id];
                if let Some(rect_pos) = sh.mapping.rect_instance_props.rect_pos {
                    let x = draw_call.instances[inst.instance_offset + rect_pos];
                    let y = draw_call.instances[inst.instance_offset + rect_pos + 1];
                    if let Some(rect_size) = sh.mapping.rect_instance_props.rect_size {
                        let w = draw_call.instances[inst.instance_offset + rect_size];
                        let h = draw_call.instances[inst.instance_offset + rect_size + 1];
                        return Some(draw_call.clip_and_scroll_rect(x, y, w, h));
                    }
                }
                None
            }
            Area::View(view_area) => {
                let cxview = &cx.views[view_area.view_id];
                Some(Rect { pos: cxview.rect.pos - cxview.parent_scroll, size: cxview.rect.size })
            }
            _ => None,
        }
    }

    /// Get an immutable slice for an [`Area::InstanceRange`].
    pub fn get_slice<T: 'static>(&self, cx: &Cx) -> &[T] {
        if !self.is_valid(cx) {
            return &mut [];
        }
        match self {
            Area::InstanceRange(inst) => {
                let cxview = &cx.views[inst.view_id];
                let draw_call = &cxview.draw_calls[inst.draw_call_id];
                let sh = &cx.shaders[draw_call.shader_id];

                let total_instance_slots = sh.mapping.instance_props.total_slots;
                let shader_bytes = total_instance_slots * std::mem::size_of::<f32>();
                let struct_bytes = std::mem::size_of::<T>();
                assert_eq!(
                    shader_bytes, struct_bytes,
                    "Mismatch between shader instance slots ({shader_bytes} bytes) and instance struct ({struct_bytes} bytes)"
                );

                // TODO(JP): Move to cast.rs?
                unsafe {
                    std::slice::from_raw_parts(
                        draw_call.instances.as_ptr().add(inst.instance_offset) as *const T,
                        inst.instance_count,
                    )
                }
            }
            _ => &mut [],
        }
    }

    /// Get an immutable reference to the first element.
    ///
    /// If no such element exists, then a default element is returned.
    ///
    /// Example:
    /// ```ignore
    /// let glyph = area.get_first::<TextIns>(cx);
    /// ```
    ///
    /// TODO(JP): It would be nice if we can eliminate the default fallback altogether;
    /// see [`Cx::temp_default_data`] for ideas.
    pub fn get_first<'a, T: 'static + Default>(&'a self, cx: &'a mut Cx) -> &'a T {
        if let Some(first) = self.get_slice::<T>(cx).get(0) {
            first
        } else {
            let len = cx.temp_default_data.len();
            cx.temp_default_data.push(Box::new(T::default()));
            // TODO(JP): Use https://doc.rust-lang.org/std/option/enum.Option.html#method.unwrap_unchecked here
            // once it's stable.
            unsafe { cx.temp_default_data.get_unchecked(len).downcast_ref().unwrap() }
        }
    }

    /// Get a mutable slice for an [`Area::InstanceRange`].
    pub fn get_slice_mut<T: 'static>(&self, cx: &mut Cx) -> &mut [T] {
        if !self.is_valid(cx) {
            return &mut [];
        }
        match self {
            Area::InstanceRange(inst) => {
                let cxview = &mut cx.views[inst.view_id];
                let draw_call = &mut cxview.draw_calls[inst.draw_call_id];
                let sh = &cx.shaders[draw_call.shader_id];

                let total_instance_slots = sh.mapping.instance_props.total_slots;
                let shader_bytes = total_instance_slots * std::mem::size_of::<f32>();
                let struct_bytes = std::mem::size_of::<T>();
                assert_eq!(
                    shader_bytes, struct_bytes,
                    "Mismatch between shader instance slots ({shader_bytes} bytes) and instance struct ({struct_bytes} bytes)"
                );

                // If we have no instances, bail early so we don't mark the entire draw call as dirty.
                if inst.instance_count == 0 {
                    return &mut [];
                }

                cx.passes[cxview.pass_id].paint_dirty = true;
                draw_call.instance_dirty = true;

                // TODO(JP): Move to cast.rs?
                unsafe {
                    std::slice::from_raw_parts_mut(
                        draw_call.instances.as_mut_ptr().add(inst.instance_offset) as *mut T,
                        inst.instance_count,
                    )
                }
            }
            _ => &mut [],
        }
    }

    /// Get a mutable reference to the first element.
    ///
    /// If no such element exists, then a default element is returned. Mutating such a default
    /// element won't do anything, but it also won't hurt.
    ///
    /// Note that in general you can't rely on these mutations to last very long, since they'll
    /// be cleared on the next redraw.
    ///
    /// Example:
    /// ```ignore
    /// let glyph = area.get_first_mut::<TextIns>(cx);
    /// ```
    ///
    /// TODO(JP): It would be nice if we can eliminate the default fallback altogether;
    /// see [`Cx::temp_default_data`] for ideas.
    pub fn get_first_mut<'a, T: 'static + Default>(&'a self, cx: &'a mut Cx) -> &'a mut T {
        if let Some(first) = self.get_slice_mut::<T>(cx).get_mut(0) {
            first
        } else {
            let len = cx.temp_default_data.len();
            cx.temp_default_data.push(Box::new(T::default()));
            // TODO(JP): Use https://doc.rust-lang.org/std/option/enum.Option.html#method.unwrap_unchecked here
            // once it's stable.
            unsafe { cx.temp_default_data.get_unchecked_mut(len).downcast_mut().unwrap() }
        }
    }

    /// Get a write user-level uniforms for the [`DrawCall`] that this [`Area`] points to.
    ///
    /// It can be useful to wrap this in [`Area::is_first_instance`] to avoid having to write
    /// this multiple times for the same [`DrawCall`].
    pub fn write_user_uniforms<T: 'static>(&self, cx: &mut Cx, uniforms: T) {
        if !self.is_valid(cx) {
            return;
        }
        match self {
            Area::InstanceRange(inst) => {
                let cxview = &mut cx.views[inst.view_id];
                let draw_call = &mut cxview.draw_calls[inst.draw_call_id];

                let shader_bytes = draw_call.user_uniforms.len() * std::mem::size_of::<f32>();
                let struct_bytes = std::mem::size_of::<T>();
                assert_eq!(
                    shader_bytes, struct_bytes,
                    "Mismatch between shader uniform slots ({shader_bytes} bytes) and instance struct ({struct_bytes} bytes)"
                );

                cx.passes[cxview.pass_id].paint_dirty = true;
                draw_call.uniforms_dirty = true;

                let data = unsafe { &mut *(draw_call.user_uniforms.as_mut_ptr() as *mut T) };
                *data = uniforms;
            }
            _ => (),
        }
    }

    /// Write a [`Texture`] value into the the [`DrawCall`] associated with this
    /// [`Area::InstanceRange`].
    pub fn write_texture_2d(&self, cx: &mut Cx, name: &str, texture_handle: TextureHandle) {
        if self.is_valid(cx) {
            if let Area::InstanceRange(inst) = self {
                let cxview = &mut cx.views[inst.view_id];
                let draw_call = &mut cxview.draw_calls[inst.draw_call_id];
                let sh = &cx.shaders[draw_call.shader_id];
                for (index, prop) in sh.mapping.textures.iter().enumerate() {
                    if prop.name == name {
                        draw_call.textures_2d[index] = texture_handle.texture_id as u32;
                        return;
                    }
                }
            }
        }
        panic!("Cannot find texture2D prop {}", name)
    }
}
