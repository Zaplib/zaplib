//! Collection of widgets for use with Zaplib.
//!
//! Doesn't contain lower level primitives; those are in [`zaplib`].

// Not great but we do these comparisons all over the place..
#![allow(clippy::float_cmp)]
// We want to use links to private fields, since we use `--document-private-items`.
#![allow(rustdoc::private_intra_doc_links)]

mod background;
pub use crate::background::*;
mod axis;
pub use crate::axis::*;
mod scrollview;
pub use crate::scrollview::*;
mod button;
pub use crate::button::*;
mod splitter;
pub use crate::splitter::*;
mod tab;
pub use crate::tab::*;
mod tabcontrol;
pub use crate::tabcontrol::*;
mod dock;
pub use crate::dock::*;
mod desktopwindow;
pub use crate::desktopwindow::*;
mod list;
pub use crate::list::*;
mod textbuffer;
pub use crate::textbuffer::*;
mod texteditor;
pub use crate::texteditor::*;
mod textcursor;
pub use crate::textcursor::*;
mod textinput;
pub use crate::textinput::*;
mod scrollshadow;
pub use crate::scrollshadow::*;
mod tokentype;
pub use crate::tokentype::*;
mod foldcaption;
pub use crate::foldcaption::*;
mod floatslider;
pub use crate::floatslider::*;
mod skybox;
pub use crate::skybox::*;
mod popover;
pub use crate::popover::*;
mod checkbox;
pub use crate::checkbox::*;
mod viewport3d;
pub use crate::viewport3d::*;
mod fps_counter;
pub use crate::fps_counter::*;
mod geometry3d;
pub use crate::geometry3d::*;

mod chart;
pub use crate::chart::*;
mod drawlines3d;
pub use crate::drawlines3d::*;
mod drawpoints3d;
pub use crate::drawpoints3d::*;
mod arrow_pointer;
pub use crate::arrow_pointer::*;

mod internal;
pub(crate) use crate::internal::*;
