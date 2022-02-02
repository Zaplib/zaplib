use crate::background::*;
use zaplib::*;

/// Popover that can be used for menus, tooltips, etc.
///
/// For more general information about popovers, see
/// <https://uxdesign.cc/pop-up-popover-or-popper-a-quick-look-into-ui-terms-cb4114fca2a>.
///
/// TODO(JP): This currently only supports showing a popover _above_ the current
/// box position. To make this more useful we should add different ways
/// of positioning this.
///
/// TODO(JP): This currently assumes you draw this on top of everything. That is
/// not always practical (e.g. if you want to show a tooltip in a deeply nested
/// widget), and it also doesn't break out of [`View`]s currently, so we might
/// need to use some combination of [`View::is_overlay`] and z-depth. To show
/// tooltips on top of everything while using the scroll position of containing
/// [`View`]s we might need some framework changes.
///
/// TODO(JP): Aligning the popover doesn't actually work for nested [`View`]s yet,
/// like [`crate::TextInput`] or [`crate::TextEditor`]. See [`Cx::do_align_x`] and [`Cx::do_align_y`].
#[derive(Default)]
pub struct Popover {
    background: Background,
}

impl Popover {
    /// Handle events for the [`Popover`] widget.
    ///
    /// Always marks events like [`PointerDownEvent`] and [`PointerHoverEvent`] as "handled".
    /// This way, we don't leak those events to whatever is sitting in the background.
    /// This does require this function to get called before any other event handlers!
    ///
    /// TODO(JP): This might actually be a place where our eventing system breaks
    /// down a bit. If you instantiate a [`Popover`] deep inside your application
    /// inside some widget, then any events (e.g. [`PointerMoveEvent`]) might already
    /// get handled earlier in the application. There are a few ways we can fix
    /// this:
    /// - We could do all of this in user space, by having some notion of a
    ///   "popover manager" that adds a [`View`] at the top level of the application
    ///   and can listen to "signal" events or so if you want to instantiate
    ///   a popover. Otherwise it might be tricky to communicate from inside
    ///   a widget to this "popover manager", or you'd have to pass a reference
    ///   to every widget! If we want to make this better, we can also consider
    ///   adding a function to get access to the application state through
    ///   [`Cx`], since we already pass that everywhere.
    /// - We could also make such a "popover manager" or "popover layer" part of
    ///   the framework itself. Rik Arends told me that that has been on his
    ///   TODO list, so there indeed isn't proper support for this yet. I think
    ///   it might be better to start in user space though, since that's more
    ///   flexible, and then we can "graduate" it when we're happy with it.
    pub fn handle(&mut self, _cx: &mut Cx, _event: &mut Event) {
        // event.hits_pointer(cx, self.component_id, self.background.area().get_rect_for_first_instance(cx));
    }

    /// Draw the popover.
    pub fn begin_draw(&mut self, cx: &mut Cx, width: Width, height: Height, color: Vec4) {
        // TODO(JP): This feels like a bit of a hack; using [`Layout::align`] like this. It might be
        // nicer to have an API that is like "move everything over by this dx/dy".
        let popover_y_bottom = cx.get_draw_pos().y;
        cx.begin_absolute_box();
        cx.begin_column(Width::Fill, Height::Fix(popover_y_bottom));
        cx.begin_bottom_box();
        self.background.begin_draw(cx, width, height, color);
    }

    /// Ends the corresponding [`Popover::begin_draw`], using its final [`Rect`] to
    /// draw and position the [`Popover::background`].
    pub fn end_draw(&mut self, cx: &mut Cx) {
        self.background.end_draw(cx);
        cx.end_bottom_box();
        cx.end_column();
        cx.end_absolute_box();
    }
}
