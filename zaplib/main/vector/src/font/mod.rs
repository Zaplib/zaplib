// Clippy TODO
#![warn(clippy::module_inception)]

pub mod outline;

mod font;
mod glyph;
mod horizontal_metrics;
mod outline_point;

pub use self::font::VectorFont;
pub use self::glyph::Glyph;
pub use self::horizontal_metrics::HorizontalMetrics;
pub use self::outline::Outline;
pub(crate) use self::outline_point::OutlinePoint;
