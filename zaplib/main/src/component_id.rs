use std::sync::atomic::{AtomicU32, Ordering};

#[cfg(doc)]
use crate::*;

/// Identifier that represents a particular "component" on the screen, even if it
/// gets moved around or disappears temporarily.
///
/// This identity gets used mostly in eventing, e.g. [`Event::hits_pointer`],
/// [`Event::hits_keyboard`], and [`Cx::set_key_focus`].
///
/// It's 32-bit, so you can use it in instance data and then read it out again,
/// either directly or when by using [`Area::get_slice`] or [`Area::get_first`].
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ComponentId(u32);

/// The next number to use for [`ComponentId`].
///
/// Starting with 1 instead of 0 to avoid confusion (someone might think 0 means empty).
static NEXT_COMPONENT_ID: AtomicU32 = AtomicU32::new(1);

impl Default for ComponentId {
    fn default() -> Self {
        // TODO(JP): Not sure if `default()` is supposed to be idempotent.. but this is
        // really convenient, so let's just go for it for now.
        Self(NEXT_COMPONENT_ID.fetch_add(1, Ordering::Relaxed))
    }
}
