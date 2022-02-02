use crate::font::Glyph;
use crate::geometry::Rectangle;

/// A font.
#[derive(Clone, Debug, PartialEq)]
pub struct VectorFont {
    pub units_per_em: f32,
    pub(crate) ascender: f32,
    pub(crate) descender: f32,
    pub(crate) line_gap: f32,
    pub(crate) bounds: Rectangle,
    pub char_code_to_glyph_index_map: Vec<usize>,
    pub glyphs: Vec<Glyph>,
}
