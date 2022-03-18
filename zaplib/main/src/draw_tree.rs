//! The main primitives for rendering to the screen.
//!
//! A draw tree has two kinds of nodes: [`View`] and [`DrawCall`]. It might look like this:
//! * `View` - root
//!   * `DrawCall` - rendering some text
//!   * `DrawCall` - rendering some buttons
//!   * `View` - scrollable container
//!     * `DrawCall` - rendering some more buttons
//!     * `DrawCall` - rendering some more text
//!   * `DrawCall` - maybe even more buttons
//!
//! [`DrawCall`]s contain the actual data that needs to be drawn to the screen.
//!
//! [`View`]s are groups of draw calls, have some special features, such as
//! scrolling/clipping.
//!
//! Note that one level higher, we have a hierarchy of [`Pass`]es.

use crate::*;

use crate::Debugger;

/// A group of [`DrawCall`]s.
///
/// This is really a pointer to a [`CxView`] (indexed in [`Cx::views`] using [`View::view_id`]),
/// so you can find more information there.
///
/// A [`View`] has a few special features:
/// * It has its own [`Rect`], within which everything is clipped (see also [`DrawUniforms`]).
///   This typically gets set by the return value of [`Cx::end_typed_box`] for the [`CxLayoutBox`] that
///   is associated with the [`View`]. TODO(JP): Look into decoupling [`CxLayoutBox`] from [`View`].
/// * It can scroll (but does not have to; again see also [`DrawUniforms`]).
/// * It has its own set of [`DrawCall`]s, which are isolated from the [`DrawCall`]s of the
///   parent [`View`].
///
/// See also [`ViewArea`], which is an [`Area`] pointer to a [`View`].
#[derive(Clone, Default)]
pub struct View {
    /// The index of the corresponding [`CxView`] in [`Cx::views`].
    pub view_id: Option<usize>,
    /// The most recent [`Cx::redraw_id`] that this [`View`] was drawn with.
    pub(crate) redraw_id: u64,
    /// Whether this [`View`] is an overlay/popup, which means all [`DrawCall`]s underneath it
    /// will get rendered last.
    pub(crate) is_overlay: bool,

    debugger: Debugger,
}

impl View {
    /// Creates a new, empty [`View`].

    /// See [`View::is_overlay`].
    #[must_use]
    pub fn with_is_overlay(self, is_overlay: bool) -> Self {
        Self { is_overlay, ..self }
    }

    /// Register the [`View`] in the draw tree.
    ///
    /// This also creates a new [`CxLayoutBox`] with the [`LayoutSize`] that is passed in.
    /// Note that you should not create a [`View`] just in order to get a new
    /// [`CxLayoutBox`], since creating a [`View`] is relatively expensive -- no
    /// [`DrawCall`]s inside this [`View`] will get merged with ones outside of
    /// it, so adding too many [`View`]s will create too many individual calls to
    /// the GPU.
    ///
    /// TODO(JP): Perhaps we should decouple [`CxLayoutBox`] and [`View`] altogether?
    pub fn begin_view(&mut self, cx: &mut Cx, layout_size: LayoutSize) {
        self.begin_view_with_layout(cx, Layout { direction: Direction::Down, layout_size, ..Layout::default() });
    }

    fn begin_view_with_layout(&mut self, cx: &mut Cx, layout: Layout) {
        if !cx.in_redraw_cycle {
            panic!("calling begin_view outside of redraw cycle is not possible!");
        }
        assert!(cx.shader_group_instance_offsets.is_empty(), "Can't use begin_view inside a shader group");

        // check if we have a pass id parent
        let pass_id = *cx.pass_stack.last().expect("No pass found when begin_view");

        let view_id = if let Some(view_id) = self.view_id {
            view_id
        } else {
            // we need a draw_list_id
            let view_id = cx.views.len();
            self.view_id = Some(view_id);
            cx.views.push(CxView::default());
            let cxview = &mut cx.views[view_id];
            cxview.redraw_id = cx.redraw_id;
            cxview.pass_id = pass_id;
            view_id
        };

        let (override_layout, is_root_for_pass) = if cx.passes[pass_id].main_view_id.is_none() {
            // we are the first view on a window
            let cxpass = &mut cx.passes[pass_id];
            cxpass.main_view_id = Some(view_id);
            // we should take the window geometry and abs position as our box layout
            (Layout { absolute: true, abs_size: Some(cxpass.pass_size), ..layout }, true)
        } else {
            (layout, false)
        };

        let cxpass = &mut cx.passes[pass_id];
        // find the parent draw list id
        let parent_view_id = if self.is_overlay {
            cxpass.main_view_id.expect("Cannot make overlay inside window without root view")
        } else if is_root_for_pass {
            view_id
        } else if let Some(last_view_id) = cx.view_stack.last() {
            *last_view_id
        } else {
            // we have no parent
            view_id
        };

        // push ourselves up the parent draw_stack
        if view_id != parent_view_id {
            // we need a new draw
            let parent_cxview = &mut cx.views[parent_view_id];

            let id = parent_cxview.draw_calls_len;
            parent_cxview.draw_calls_len += 1;

            // see if we need to add a new one
            if parent_cxview.draw_calls_len > parent_cxview.draw_calls.len() {
                parent_cxview.draw_calls.push({
                    DrawCall {
                        view_id: parent_view_id,
                        draw_call_id: parent_cxview.draw_calls.len(),
                        redraw_id: cx.redraw_id,
                        sub_view_id: view_id,
                        ..Default::default()
                    }
                })
            } else {
                // or reuse a sub list node
                let draw = &mut parent_cxview.draw_calls[id];
                draw.sub_view_id = view_id;
                draw.redraw_id = cx.redraw_id;
            }
        }

        // TODO(JP): Do we want to keep this? We don't really use this for anything except as a
        // convenience. I talked with Rik about redrawing of [`View`]s, and one idea was to always
        // fully invalidate the closest [`View`] parent that did not have a [`Layout`] with
        // [`Width::Compute`] or [`Height::Compute`], but that seems to fragile to me. It would be
        // better to check if a [`CxView::rect`] actually changed and in that case trigger a redraw
        // or even a panic (with some way of manually overriding the panic). So anyway, I think we
        // should strive to remove this after all.
        cx.begin_typed_box(CxBoxType::View, override_layout);

        // prepare drawlist for drawing
        let cxview = &mut cx.views[view_id];

        // TODO(JP): We don't seem to currently support moving a `View` to a different pass. Do we
        // want to?
        assert_eq!(cxview.pass_id, pass_id);

        // update drawlist ids
        self.redraw_id = cx.redraw_id;
        cxview.redraw_id = cx.redraw_id;
        cxview.draw_calls_len = 0;

        cx.view_stack.push(view_id);

        if is_root_for_pass {
            cx.passes[pass_id].paint_dirty = true;
        }
    }

    fn is_main_view(view_id: usize, cx: &mut Cx) -> bool {
        if let Some(window_id) = cx.window_stack.last() {
            if let Some(main_pass_id) = cx.windows[*window_id].main_pass_id {
                let pass_id = cx.views[view_id].pass_id;
                let cxpass = &cx.passes[pass_id];
                if let Some(main_view_id) = cxpass.main_view_id {
                    if main_view_id == view_id && main_pass_id == pass_id {
                        // we are the main view of a main pass of a window
                        return true;
                    }
                }
            }
        }
        false
    }

    /// End the [`View`], by ending the [`CxLayoutBox`]. Returns a [`ViewArea`] that
    /// you can hold onto.
    ///
    /// Should only be called if [`View::begin_view`] returned [`Result::Ok`].
    ///
    /// TODO(JP): Is the [`ViewArea`] redundant, since it basically contains the
    /// same information as the [`View`] itself?
    pub fn end_view(&mut self, cx: &mut Cx) -> Area {
        assert!(cx.shader_group_instance_offsets.is_empty(), "Can't use end_view inside a shader group");

        let view_id = self.view_id.expect("Not inside a View::begin_view currently");

        if cx.debug_flags.enable_layout_debugger && View::is_main_view(view_id, cx) {
            self.debugger.draw(cx);
        }

        let view_area = Area::View(ViewArea { view_id, redraw_id: cx.redraw_id });
        // Make sure that ViewArea would also be aligned when underlying calls getting moved
        cx.layout_box_align_list.push(view_area);

        let rect = cx.end_typed_box(CxBoxType::View);
        cx.views[view_id].rect = rect;
        cx.view_stack.pop();
        view_area
    }

    /// Get the [`Rect`] that the [`CxLayoutBox`] associated with the [`View`]
    /// returned.
    ///
    /// TODO(JP): Should we return an [`Option<Rect>`] instead of just
    /// returning a zero-sized [`Rect`] when the [`View`] has never been
    /// drawn yet?
    ///
    /// TODO(JP): Doesn't check if the [`View::redraw_id`] is still up to
    /// date, so we might be returning an outdated [`Rect`] here.
    pub fn get_rect(&self, cx: &Cx) -> Rect {
        if let Some(view_id) = self.view_id {
            let cxview = &cx.views[view_id];
            return cxview.rect;
        }
        Rect::default()
    }

    /// Returns an [`Area::View`] for this [`View`], or [`Area::Empty`] if the
    /// [`View`] hasn't been instantiated yet.
    pub fn area(&self) -> Area {
        if let Some(view_id) = self.view_id {
            Area::View(ViewArea { view_id, redraw_id: self.redraw_id })
        } else {
            Area::Empty
        }
    }

    /// Get the current [`CxView::unsnapped_scroll`] if the [`View`] has been
    /// instantiated.
    pub fn get_scroll_pos(&self, cx: &Cx) -> Vec2 {
        if let Some(view_id) = self.view_id {
            let cxview = &cx.views[view_id];
            cxview.unsnapped_scroll
        } else {
            Vec2::default()
        }
    }
}

impl Cx {
    /// Returns an existing [`DrawCall`] based on the given [`Shader`], or
    /// creates a new one if none can be found in the current [`CxView`].
    ///
    /// Reuses an existing [`DrawCall`] if [`CxView::draw_calls_len`] is less than
    /// [`CxView::draw_calls.len()`], so we can reuse existing GPU resources.
    ///
    /// TODO(JP): It's unclear to me if the reusing of GPU resources in this way
    /// is beneficial. And if it is, if it should instead be done in the
    /// platform-specific code instead.
    fn create_draw_call(&mut self, shader_id: usize, props: DrawCallProps) -> &mut DrawCall {
        assert!(self.in_redraw_cycle, "Must be in redraw cycle to append to draw calls");

        let sh = &self.shaders[shader_id];

        let current_view_id = *self.view_stack.last().expect("Not inside a View::begin_view currently");
        let cxview = &mut self.views[current_view_id];
        let draw_call_id = cxview.draw_calls_len;

        // Find a draw call to append to.
        if props.is_batchable() {
            let shader_group_size = self.shader_group_instance_offsets.len();
            if shader_group_size > 0 {
                // If we're in a shader group then the given shader must be part of the group, so we just
                // search within the group.
                assert!(cxview.draw_calls_len >= shader_group_size);
                for index in cxview.draw_calls_len - shader_group_size..cxview.draw_calls_len {
                    if cxview.draw_calls[index].shader_id == shader_id {
                        return &mut cxview.draw_calls[index];
                    }
                }
                panic!("Trying to use Shader within a shader group that isn't part of the group");
            } else {
                // Do the most basic of [`DrawCall`] batching, by checking if the very last [`DrawCall`] matches
                // the shader that we're drawing, and if so, appending to that.
                if cxview.draw_calls_len > 0 && !self.debug_flags.disable_draw_call_batching {
                    let dc = &mut cxview.draw_calls[cxview.draw_calls_len - 1];
                    if dc.props.is_batchable() && dc.sub_view_id == 0 && dc.shader_id == shader_id {
                        return &mut cxview.draw_calls[cxview.draw_calls_len - 1];
                    }
                }
            }
        }

        // add one
        cxview.draw_calls_len += 1;

        // see if we need to add a new one
        if draw_call_id >= cxview.draw_calls.len() {
            cxview.draw_calls.push(DrawCall {
                props,
                draw_call_id,
                view_id: current_view_id,
                redraw_id: self.redraw_id,
                sub_view_id: 0,
                shader_id,
                instances: Vec::new(),
                draw_uniforms: DrawUniforms::default(),
                user_uniforms: {
                    let mut f = Vec::new();
                    f.resize(sh.mapping.user_uniform_props.total_slots, 0.0);
                    f
                },
                textures_2d: {
                    let mut f = Vec::new();
                    f.resize(sh.mapping.textures.len(), 0);
                    f
                },
                //current_instance_offset: 0,
                instance_dirty: true,
                uniforms_dirty: true,
                platform: CxPlatformDrawCall::default(),
            });
            let dc = &mut cxview.draw_calls[draw_call_id];
            return dc;
        }
        // reuse an older one, keeping all GPU resources attached
        let dc = &mut cxview.draw_calls[draw_call_id];
        dc.shader_id = shader_id;
        dc.props = props;
        dc.sub_view_id = 0; // make sure its recognised as a draw call
                            // truncate buffers and set update frame
        dc.redraw_id = self.redraw_id;
        dc.instances.truncate(0);
        dc.user_uniforms.truncate(0);
        dc.user_uniforms.resize(sh.mapping.user_uniform_props.total_slots, 0.0);
        dc.textures_2d.truncate(0);
        dc.textures_2d.resize(sh.mapping.textures.len(), 0);
        dc.instance_dirty = true;
        dc.uniforms_dirty = true;
        dc
    }

    /// Add a slice of instances to [`DrawCall::instances`]. See [`Cx::add_instances`].
    fn add_instances_internal<T: 'static + Copy>(&mut self, shader: &'static Shader, data: &[T], props: DrawCallProps) -> Area {
        if data.is_empty() {
            // This is important, because otherwise you can call this function with empty data in order to force
            // a particular ordering of `DrawCall`s, and then depend on batching of `DrawCall`s. That should
            // generally be avoided -- we might change how `DrawCall` batching works in the future.
            return Area::Empty;
        }
        let shader_id = self.get_shader_id(shader);
        let cxshader = &self.shaders[shader_id];

        let total_instance_slots = cxshader.mapping.instance_props.total_slots;
        let shader_bytes_instance = total_instance_slots * std::mem::size_of::<f32>();
        let struct_bytes_instance = std::mem::size_of::<T>();
        assert_eq!(
            shader_bytes_instance, struct_bytes_instance,
            "Mismatch between shader instance slots ({shader_bytes_instance} bytes) and instance struct \
             ({struct_bytes_instance} bytes)"
        );

        let dc = self.create_draw_call(shader_id, props);

        let ia = InstanceRangeArea {
            view_id: dc.view_id,
            draw_call_id: dc.draw_call_id,
            instance_count: data.len(),
            instance_offset: dc.instances.len(),
            redraw_id: dc.redraw_id,
        };
        dc.instances.extend_from_slice(cast_slice::<T, f32>(&data));
        let area = Area::InstanceRange(ia);
        self.add_to_box_align_list(area);

        area
    }

    /// Add a slice of instances to [`DrawCall::instances`].
    ///
    /// Supports appending any data that has the correct size.
    ///
    /// You should assume that any call to this creates a new [`DrawCall`], even though in reality we might
    /// batch certain [`DrawCall`]s. For more information about [`DrawCall`] batching, see [`Shader`].
    ///
    /// Uses [`Cx::create_draw_call`] under the hood to find the [`DrawCall`]
    /// to add to.
    pub fn add_instances<T: 'static + Copy>(&mut self, shader: &'static Shader, data: &[T]) -> Area {
        assert!(shader.build_geom.is_some(), "Can't add instances without `build_geom` defined");

        self.add_instances_internal(shader, data, DrawCallProps::default())
    }

    /// Add a slice of instances while specifying a custom Geometry
    pub fn add_mesh_instances<T: 'static + Copy>(
        &mut self,
        shader: &'static Shader,
        data: &[T],
        gpu_geometry: GpuGeometry,
    ) -> Area {
        assert!(self.shader_group_instance_offsets.is_empty(), "Can't add mesh instances when in a shader group");

        self.add_instances_internal(shader, data, DrawCallProps { gpu_geometry: Some(gpu_geometry), ..Default::default() })
    }

    /// By default, [`DrawCall`] gets horizontal and vertical scrolling applied to
    /// its uniforms, but you can disable that by calling this method. This only
    /// applies to scrolling from its direct parent [`View`].
    ///
    /// This always creates a new [`DrawCall`]; no batching ever happens when using
    /// sticky scrolling.
    ///
    /// TODO(JP): The fact that this only applies to the direct parent [`View`]
    /// makes it so you can't just arbitrarily wrap [`DrawCall`]s inside [`View`]s,
    /// which is somewhat unexpected. It might be better to have this apply to
    /// the nearest `zaplib_components::ScrollView` instead?
    ///
    /// TODO(JP): Do we need to track as fields on [`DrawCall`]? The same behavior
    /// can also be accomplished by overriding the [`Shader`]'s `scroll`
    /// function, by doing `draw_scroll - draw_local_scroll`. It's not as
    /// convenient, but then again, it might not be used very often, and it would
    /// encourage people to do more stuff in shaders.
    pub fn add_instances_with_scroll_sticky<T: 'static + Copy>(
        &mut self,
        shader: &'static Shader,
        data: &[T],
        horizontal: bool,
        vertical: bool,
    ) -> Area {
        assert!(self.shader_group_instance_offsets.is_empty(), "Can't add instances with scroll sticky when in a shader group");

        self.add_instances_internal(
            shader,
            data,
            DrawCallProps { scroll_sticky_horizontal: horizontal, scroll_sticky_vertical: vertical, ..Default::default() },
        )
    }

    /// Start a "shader group", which is a group of [`Shader`]s that will always be drawn in
    /// the same order.
    ///
    /// When drawing the same "shader group" multiple times in a row, the existing [`DrawCall`]s
    /// will be reused (batched) instead of new ones being created.
    ///
    /// For example, calling `cx.begin_shader_group(&[&FOO_SHADER, &BAR_SHADER]);` will guarantee
    /// that for that shader group exactly two [`DrawCall`]s will be created, with `FOO_SHADER`
    /// being always drawn first.
    pub fn begin_shader_group(&mut self, shaders_ordered: &[&'static Shader]) {
        assert!(self.in_redraw_cycle, "Must be in redraw cycle to call begin_shader_group");
        assert!(self.shader_group_instance_offsets.is_empty(), "Nested shader groups are not supported (yet)");

        let shader_ids: Vec<usize> = shaders_ordered.iter().map(|&shader| self.get_shader_id(shader)).collect();

        // Make sure shaders are unique.
        debug_assert!(
            {
                let mut unique_shader_ids = shader_ids.clone();
                unique_shader_ids.sort_unstable();
                unique_shader_ids.dedup();
                shader_ids.len() == unique_shader_ids.len()
            },
            "Can't use shader more than once in shader group"
        );

        let shader_group_size = shaders_ordered.len();
        let current_view_id = *self.view_stack.last().expect("Not inside a View::begin_view currently");
        let cxview = &self.views[current_view_id];

        // We have to hold the following invariant: if shader_group_instance_offsets is not empty, then the last
        // set of `DrawCall`s in the current `CxView` have to match exactly the `shaders_ordered`. If that
        // invariant already holds (e.g. if the exact same shader group was previously used) then we can skip
        // creating new `DrawCall`s.
        if self.debug_flags.disable_draw_call_batching
            || cxview.draw_calls_len < shader_group_size
            || shader_ids.iter().enumerate().any(|(index, &shader_id)| {
                let dc = &cxview.draw_calls[cxview.draw_calls_len - shader_group_size + index];
                dc.shader_id != shader_id || dc.sub_view_id != 0
            })
        {
            for shader_id in shader_ids {
                self.create_draw_call(shader_id, DrawCallProps::default());
            }
        }

        // Since `shader_group_instance_offsets` is empty (see assertion above) we can just extend with an iterator.
        let cxview = &self.views[current_view_id];
        self.shader_group_instance_offsets.extend(
            (cxview.draw_calls_len - shader_group_size..cxview.draw_calls_len)
                .map(|index| cxview.draw_calls[index].instances.len()),
        );
    }

    /// End a "shader group". See [`Cx::begin_shader_group`].
    pub fn end_shader_group(&mut self) {
        assert!(!self.shader_group_instance_offsets.is_empty(), "Call begin_shader_group before end_shader_group");
        self.shader_group_instance_offsets.clear();
    }

    /// Sets the horizontal scroll position for a [`View`]/[`CxView`].
    pub fn set_view_scroll_x(&mut self, view_id: usize, scroll_pos: f32) {
        let fac = self.get_delegated_dpi_factor(self.views[view_id].pass_id);
        let cxview = &mut self.views[view_id];
        cxview.unsnapped_scroll.x = scroll_pos;
        let snapped = scroll_pos - scroll_pos % (1.0 / fac);
        if cxview.snapped_scroll.x != snapped {
            cxview.snapped_scroll.x = snapped;
            self.passes[cxview.pass_id].paint_dirty = true;
        }
    }

    /// Sets the vertical scroll position for a [`View`]/[`CxView`].
    pub fn set_view_scroll_y(&mut self, view_id: usize, scroll_pos: f32) {
        let fac = self.get_delegated_dpi_factor(self.views[view_id].pass_id);
        let cxview = &mut self.views[view_id];
        cxview.unsnapped_scroll.y = scroll_pos;
        let snapped = scroll_pos - scroll_pos % (1.0 / fac);
        if cxview.snapped_scroll.y != snapped {
            cxview.snapped_scroll.y = snapped;
            self.passes[cxview.pass_id].paint_dirty = true;
        }
    }
}

/// Hardcoded set of uniforms that are present on every [`DrawCall`].
///
/// TODO(JP): Should we just use [`Vec4`]s and [`Vec2`] here instead of individual
/// [`f32`]s?
#[derive(Default, Clone)]
#[repr(C, align(8))]
pub(crate) struct DrawUniforms {
    /// Clip region top left x-position.
    draw_clip_x1: f32,
    /// Clip region top left y-position.
    draw_clip_y1: f32,
    /// Clip region bottom right x-position.
    draw_clip_x2: f32,
    /// Clip region bottom right y-position.
    draw_clip_y2: f32,
    /// The total horizontal scroll offset, including all its parents.
    pub(crate) draw_scroll_x: f32,
    /// The total vertical scroll offset, including all its parents.
    pub(crate) draw_scroll_y: f32,
    /// The horizontal scroll offset of just the containing [`View`].
    pub(crate) draw_local_scroll_x: f32,
    /// The vertical scroll offset of just the containing [`View`].
    pub(crate) draw_local_scroll_y: f32,
    /// A small increment that you can add to the z-axis of your vertices, which is based on the
    /// position of the [`DrawCall`] in the draw tree.
    ///
    /// TODO(JP): Not entirely sure why we need this, given that we're already drawing everything
    /// in order?
    draw_zbias: f32,
}

impl DrawUniforms {
    /// Get as a raw `[f32]` slice.
    pub fn as_slice(&self) -> &[f32; std::mem::size_of::<DrawUniforms>()] {
        unsafe { std::mem::transmute(self) }
    }
}

/// Some user-defined props to initialize a [`DrawCall`] with.
#[derive(Default)]
pub(crate) struct DrawCallProps {
    /// The base [`Geometry`] object that will be used for generating the initial
    /// vertex locations for every instance, such as a rectangle or cube.
    /// This is currently only used when specifying custom meshes.
    pub(crate) gpu_geometry: Option<GpuGeometry>,
    /// See [`Cx::add_instances_with_scroll_sticky`].
    scroll_sticky_vertical: bool,
    /// See [`Cx::add_instances_with_scroll_sticky`].
    scroll_sticky_horizontal: bool,
}
impl DrawCallProps {
    /// Whether the draw call can be batched, or if a new one should be created.
    fn is_batchable(&self) -> bool {
        self.gpu_geometry.is_none() && !self.scroll_sticky_horizontal && !self.scroll_sticky_vertical
    }
}

/// This represents an actual call to the GPU, _or_ it can represent a
/// sub-[`View`], in case [`DrawCall::sub_view_id`] is set. Note that all of this behaves
/// completely differently if [`DrawCall::sub_view_id`] is set; all regular drawing fields
/// are ignored in that case!
///
/// TODO(JP): this sub-[`View`] behavior is confusing, and we should instead
/// split this out to something like [`enum DrawTreeItem { DrawCall(DrawCall),
/// NestedView(usize) }`] or so.
///
/// That said, for a regular [`DrawCall`], this contains all the information that
/// you need to make a draw call on the GPU: the [`Shader`], [`DrawCall::instances`],
/// [`DrawCall::draw_uniforms`], and so on.
///
/// It is always kept in [`CxView::draw_calls`], and as said, is part of a tree
/// structure, called the "draw tree". To print a textual representation of the
/// draw tree, use [`Cx::debug_flags_mut`].
#[derive(Default)]
pub struct DrawCall {
    /// The index of this [`DrawCall`] within its parent [`CxView::draw_calls`].
    pub(crate) draw_call_id: usize,
    /// The parent [`CxView`]/[`View`] that this [`DrawCall`] is a part of.
    pub(crate) view_id: usize,
    /// The [`Cx::redraw_id`] of the last time this [`DrawCall`] was accessed.
    pub(crate) redraw_id: u64,
    /// If not 0, this [`DrawCall`] actually represents a nested sub-[`View`].
    /// See [`DrawCall`] for a TODO on fixing this, because this is confusing!
    pub(crate) sub_view_id: usize,
    /// The actual [`Shader`] to use when drawing.
    pub(crate) shader_id: usize,
    /// The instance buffer that will be sent directly to the GPU.
    pub(crate) instances: Vec<f32>,
    /// Buffer of user-defined uniforms (in addition to the [`draw_uniforms`
    /// below.)
    pub(crate) user_uniforms: Vec<f32>,
    /// Buffer of texture IDs.
    pub(crate) textures_2d: Vec<u32>,
    /// Whether or not the draw call has been accessed since the last paint.
    /// Should currently always be the same as [`DrawCall::uniforms_dirty`] below.
    pub(crate) instance_dirty: bool,
    /// Whether or not the draw call has been accessed since the last paint.
    /// Should currently always be the same as [`DrawCall::instance_dirty`] above.
    pub(crate) uniforms_dirty: bool,
    /// Hardcoded set of uniforms that are present on every [`DrawCall`].
    pub(crate) draw_uniforms: DrawUniforms,
    /// Platform-specific data for use during painting.
    pub(crate) platform: CxPlatformDrawCall,
    pub(crate) props: DrawCallProps,
}

impl DrawCall {
    /// Set the scroll uniforms in [`DrawCall::draw_uniforms`], as computed when
    /// walking the draw tree during painting.
    pub(crate) fn set_local_scroll(&mut self, scroll: Vec2, local_scroll: Vec2) {
        self.draw_uniforms.draw_scroll_x = scroll.x;
        if !self.props.scroll_sticky_horizontal {
            self.draw_uniforms.draw_scroll_x += local_scroll.x;
        }
        self.draw_uniforms.draw_scroll_y = scroll.y;
        if !self.props.scroll_sticky_vertical {
            self.draw_uniforms.draw_scroll_y += local_scroll.y;
        }
        self.draw_uniforms.draw_local_scroll_x = local_scroll.x;
        self.draw_uniforms.draw_local_scroll_y = local_scroll.y;
    }

    /// Set the zbias in [`DrawCall::draw_uniforms`], as computed when
    /// walking the draw tree during painting.
    pub(crate) fn set_zbias(&mut self, zbias: f32) {
        self.draw_uniforms.draw_zbias = zbias;
    }

    /// Set the clip dimensions in [`DrawCall::draw_uniforms`], as computed when
    /// walking the draw tree during painting.
    pub(crate) fn set_clip(&mut self, clip: (Vec2, Vec2)) {
        self.draw_uniforms.draw_clip_x1 = clip.0.x;
        self.draw_uniforms.draw_clip_y1 = clip.0.y;
        self.draw_uniforms.draw_clip_x2 = clip.1.x;
        self.draw_uniforms.draw_clip_y2 = clip.1.y;
    }

    /// Get the actual position on the screen given the scroll and clip uniforms
    /// in [`DrawCall::draw_uniforms`].
    pub(crate) fn clip_and_scroll_rect(&self, x: f32, y: f32, w: f32, h: f32) -> Rect {
        let mut x1 = x - self.draw_uniforms.draw_scroll_x;
        let mut y1 = y - self.draw_uniforms.draw_scroll_y;
        let mut x2 = x1 + w;
        let mut y2 = y1 + h;
        x1 = self.draw_uniforms.draw_clip_x1.max(x1).min(self.draw_uniforms.draw_clip_x2);
        y1 = self.draw_uniforms.draw_clip_y1.max(y1).min(self.draw_uniforms.draw_clip_y2);
        x2 = self.draw_uniforms.draw_clip_x1.max(x2).min(self.draw_uniforms.draw_clip_x2);
        y2 = self.draw_uniforms.draw_clip_y1.max(y2).min(self.draw_uniforms.draw_clip_y2);
        Rect { pos: vec2(x1, y1), size: vec2(x2 - x1, y2 - y1) }
    }
}

/// Uniforms that can be set on the [`View`] that wraps a [`DrawCall`].
///
/// TODO(JP): Currently empty, but I can see this be potentially useful, so I left
/// the code around. Might want to either make use of this directly, or expose it
/// as something users can configure, or just remove altogether.
///  - This could potentially be used for adding transformations of many instances,
///    for example translating or rotating, similarly to ThreeJS's Group abstraction.
#[derive(Default, Clone)]
#[repr(C)]
pub struct ViewUniforms {}

impl ViewUniforms {
    pub fn as_slice(&self) -> &[f32; std::mem::size_of::<ViewUniforms>()] {
        unsafe { std::mem::transmute(self) }
    }
}

/// An actual instantiation of a [`View`]. It's a node in the draw tree with
/// children, which can be either [`DrawCall`]s or other [`View`]s.
///
/// Child [`View`]s are represented by [`DrawCall`]s that have [`DrawCall::sub_view_id`] set.
///
/// TODO(JP): this sub-[`View`] behavior is confusing, and we should instead
/// split out [`DrawCall`] into something like [`enum DrawTreeItem { DrawCall(DrawCall),
/// NestedView(usize) }`] or so.
///
/// See also [`View`] and [`ViewArea`].
#[derive(Default)]
pub struct CxView {
    /// The actual children, which are always [`DrawCall`] objects, but those can
    /// represent either actual draw calls or child [`CxView`]s (see [`DrawCall`] and
    /// [`CxView`] for more documentation).
    pub(crate) draw_calls: Vec<DrawCall>,
    /// The [`Rect`] of the [`CxLayoutBox`] that we created in [`View::begin_view`].
    ///
    /// TODO(JP): We want to decouple [`CxLayoutBox`] more from [`CxView`], so we have to
    /// figure out what to do with this. For [`View`]s that are actively used for
    /// scrolling and clipping, having this `rect` makes sense, but maybe not for
    /// other uses?
    pub(crate) rect: Rect,
    /// The [`Cx::redraw_id`] of the last time this [`CxView`] was drawn. Can be used
    /// to see if an [`ViewArea`] pointer is still valid.
    ///
    /// TODO(JP): There is no way to tell if a [`CxView`] is still part of the draw tree,
    /// since merely comparing [`CxView::redraw_id`] and [`Cx::redraw_id`] is not
    /// enough, since those can also be different if the [`CxView`] was simply not
    /// marked for redrawing recently. It would be good to have some way to clean up
    /// old [`CxView`]s.
    pub(crate) redraw_id: u64,
    /// The [`Pass`]/[`CxPass`] that this is part of.
    ///
    /// TODO(JP): What happens if you change this after instantiating a [`CxView`]?
    /// Does that even work? Should it be supported?
    pub(crate) pass_id: usize,
    /// The actual number of fields in [`CxView::draw_calls`] that we use, so we can keep
    /// GPU resources associated with each [`DrawCall`] associated even when not in use.
    ///
    /// TODO(JP): Is this actually useful? Is caching of resources like that worth it, or
    /// should we do it on a per-platform basis, and only where it's really necessary?
    pub(crate) draw_calls_len: usize,
    /// The cumulative scroll offset from all of the parents. Gets set during painting.
    pub(crate) parent_scroll: Vec2,
    /// See [`ViewUniforms`].
    pub(crate) view_uniforms: ViewUniforms,
    /// The actual scroll position, including fractional offsets.
    pub(crate) unsnapped_scroll: Vec2,
    /// The scroll position that gets snapped to actual pixel values (taking into account
    /// the device pixel ratio; called `dpi_factor` internally).
    pub(crate) snapped_scroll: Vec2,

    /// Platform-specific fields. Currently only used on Windows.
    #[cfg_attr(not(target_os = "windows"), allow(dead_code))]
    pub(crate) platform: CxPlatformView,
}

impl CxView {
    /// Returns the intersection of clip coordinates and [`CxView::rect`], taking
    /// into account [`CxView::parent_scroll`].
    ///
    /// TODO(JP): Should this instead take and return a [`Rect`]?
    pub(crate) fn intersect_clip(&self, clip: (Vec2, Vec2)) -> (Vec2, Vec2) {
        let min_x = self.rect.pos.x - self.parent_scroll.x;
        let min_y = self.rect.pos.y - self.parent_scroll.y;
        let max_x = self.rect.pos.x + self.rect.size.x - self.parent_scroll.x;
        let max_y = self.rect.pos.y + self.rect.size.y - self.parent_scroll.y;

        (Vec2 { x: min_x.max(clip.0.x), y: min_y.max(clip.0.y) }, Vec2 { x: max_x.min(clip.1.x), y: max_y.min(clip.1.y) })
    }
}
