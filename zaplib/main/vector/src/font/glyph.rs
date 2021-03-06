use crate::font::{HorizontalMetrics, Outline};
use crate::geometry::Rectangle;

/// A glyph in a font.
#[derive(Clone, Debug, PartialEq)]
pub struct Glyph {
    pub horizontal_metrics: HorizontalMetrics,
    pub bounds: Rectangle,
    pub outline: Outline,
}
