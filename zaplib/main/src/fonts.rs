//! Font drawing primitives.
//!
//! TODO(JP): It's hard to get text to render crisply; see
//! * https://github.com/Zaplib/zaplib/issues/169
//! * https://github.com/Zaplib/zaplib/issues/174
//! * https://github.com/Zaplib/zaplib/issues/175

use std::sync::RwLock;
use std::sync::RwLockReadGuard;

use crate::*;
use zaplib_vector::geometry::Trapezoid;
use zaplib_vector::geometry::{AffineTransformation, Transform, Vector};
use zaplib_vector::internal_iter::*;
use zaplib_vector::path::PathIterator;
use zaplib_vector::trapezoidator::Trapezoidator;

/// The default [Ubuntu font](https://design.ubuntu.com/font/).
const FONT_UBUNTU_REGULAR: Font = Font { font_id: 0 };
/// The monospace [Liberation mono font](https://en.wikipedia.org/wiki/Liberation_fonts).
const FONT_LIBERATION_MONO_REGULAR: Font = Font { font_id: 1 };
/// Actual font data; should match the font_ids above.
#[cfg(not(feature = "disable-fonts"))]
const FONTS_BYTES: &[&[u8]] =
    &[include_bytes!("../resources/Ubuntu-R.ttf"), include_bytes!("../resources/LiberationMono-Regular.ttf")];

/// The default [`TextStyle`].
pub const TEXT_STYLE_NORMAL: TextStyle = TextStyle {
    font: FONT_UBUNTU_REGULAR,
    font_size: 8.0,
    brightness: 1.0,
    curve: 0.6,
    line_spacing: 1.4,
    top_drop: 1.2,
    height_factor: 1.3,
};

/// A monospace [`TextStyle`].
pub const TEXT_STYLE_MONO: TextStyle = TextStyle {
    font: FONT_LIBERATION_MONO_REGULAR,
    brightness: 1.1,
    font_size: 8.0,
    line_spacing: 1.8,
    top_drop: 1.3,
    ..TEXT_STYLE_NORMAL
};

/// A pointer to a [`CxFont`] (indexed in [`CxFontsData::fonts`] using [`Font::font_id`]),
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Font {
    pub font_id: usize,
}

/// Style for how to render text.
///
/// TODO(hernan): Should we include color and font scaling as part of the text style?
///
/// TODO(JP): Make these more easily debuggable: https://github.com/Zaplib/zaplib/issues/174
#[derive(Clone, Debug, Copy)]
pub struct TextStyle {
    pub font: Font,
    pub font_size: f32,
    pub brightness: f32,
    pub curve: f32,
    pub line_spacing: f32,
    pub top_drop: f32,
    pub height_factor: f32,
}

impl Default for TextStyle {
    fn default() -> Self {
        TextStyle {
            font: Font { font_id: 0 },
            font_size: 8.0,
            brightness: 1.0,
            curve: 0.6,
            line_spacing: 1.4,
            top_drop: 1.1,
            height_factor: 1.3,
        }
    }
}

impl Cx {
    pub(crate) fn load_fonts(&mut self) {
        #[cfg(not(feature = "disable-fonts"))]
        {
            let mut write_fonts_data = self.fonts_data.write().unwrap();
            write_fonts_data.fonts = Iterator::map(FONTS_BYTES.iter(), |bytes| {
                let font = zaplib_vector::ttf_parser::parse_ttf(bytes).expect("Error loading font");
                CxFont { font_loaded: Some(font), atlas_pages: vec![] }
            })
            .collect();
        }
    }

    pub fn reset_font_atlas_and_redraw(&mut self) {
        {
            // Use a block here to constraint the lifetime of locks

            let mut write_fonts = self.fonts_data.write().unwrap();

            for font in &mut write_fonts.fonts {
                font.atlas_pages.truncate(0);
            }

            write_fonts.fonts_atlas.alloc_xpos = 0.;
            write_fonts.fonts_atlas.alloc_ypos = 0.;
            write_fonts.fonts_atlas.alloc_hmax = 0.;
            write_fonts.fonts_atlas.clear_buffer = true;
        }

        self.request_draw();
    }
}

static SHADER: Shader = Shader {
    build_geom: Some(QuadIns::build_geom),
    code_to_concatenate: &[
        Cx::STD_SHADER,
        code_fragment!(
            r#"
            geometry geom: vec2;

            // trapezoid
            instance a_xs: vec2;
            instance a_ys: vec4;
            // index
            instance chan: float;

            varying v_p0: vec2;
            varying v_p1: vec2;
            varying v_p2: vec2;
            varying v_p3: vec2;
            varying v_pixel: vec2;

            fn intersect_line_segment_with_vertical_line(p0: vec2, p1: vec2, x: float) -> vec2 {
                return vec2(
                    x,
                    mix(p0.y, p1.y, (x - p0.x) / (p1.x - p0.x))
                );
            }

            fn intersect_line_segment_with_horizontal_line(p0: vec2, p1: vec2, y: float) -> vec2 {
                return vec2(
                    mix(p0.x, p1.x, (y - p0.y) / (p1.y - p0.y)),
                    y
                );
            }

            fn compute_clamped_right_trapezoid_area(p0: vec2, p1: vec2, p_min: vec2, p_max: vec2) -> float {
                let x0 = clamp(p0.x, p_min.x, p_max.x);
                let x1 = clamp(p1.x, p_min.x, p_max.x);
                if (p0.x < p_min.x && p_min.x < p1.x) {
                    p0 = intersect_line_segment_with_vertical_line(p0, p1, p_min.x);
                }
                if (p0.x < p_max.x && p_max.x < p1.x) {
                    p1 = intersect_line_segment_with_vertical_line(p0, p1, p_max.x);
                }
                if (p0.y < p_min.y && p_min.y < p1.y) {
                    p0 = intersect_line_segment_with_horizontal_line(p0, p1, p_min.y);
                }
                if (p1.y < p_min.y && p_min.y < p0.y) {
                    p1 = intersect_line_segment_with_horizontal_line(p1, p0, p_min.y);
                }
                if (p0.y < p_max.y && p_max.y < p1.y) {
                    p1 = intersect_line_segment_with_horizontal_line(p0, p1, p_max.y);
                }
                if (p1.y < p_max.y && p_max.y < p0.y) {
                    p0 = intersect_line_segment_with_horizontal_line(p1, p0, p_max.y);
                }
                p0 = clamp(p0, p_min, p_max);
                p1 = clamp(p1, p_min, p_max);
                let h0 = p_max.y - p0.y;
                let h1 = p_max.y - p1.y;
                let a0 = (p0.x - x0) * h0;
                let a1 = (p1.x - p0.x) * (h0 + h1) * 0.5;
                let a2 = (x1 - p1.x) * h1;
                return a0 + a1 + a2;
            }

            fn compute_clamped_trapezoid_area(p_min: vec2, p_max: vec2) -> float {
                let a0 = compute_clamped_right_trapezoid_area(v_p0, v_p1, p_min, p_max);
                let a1 = compute_clamped_right_trapezoid_area(v_p2, v_p3, p_min, p_max);
                return a0 - a1;
            }

            fn pixel() -> vec4 {
                let p_min = v_pixel.xy - 0.5;
                let p_max = v_pixel.xy + 0.5;
                let t_area = compute_clamped_trapezoid_area(p_min, p_max);
                if chan < 0.5 {
                    return vec4(t_area, 0., 0., 0.);
                }
                if chan < 1.5 {
                    return vec4(0., t_area, 0., 0.);
                }
                if chan < 2.5 {
                    return vec4(0., 0., t_area, 0.);
                }
                return vec4(t_area, t_area, t_area, 0.);
            }

            fn vertex() -> vec4 {
                let pos_min = vec2(a_xs.x, min(a_ys.x, a_ys.y));
                let pos_max = vec2(a_xs.y, max(a_ys.z, a_ys.w));
                let pos = mix(pos_min - 1.0, pos_max + 1.0, geom);

                // set the varyings
                v_p0 = vec2(a_xs.x, a_ys.x);
                v_p1 = vec2(a_xs.y, a_ys.y);
                v_p2 = vec2(a_xs.x, a_ys.z);
                v_p3 = vec2(a_xs.y, a_ys.w);
                v_pixel = pos;
                return camera_projection * vec4(pos, 0.0, 1.0);
            }"#
        ),
    ],
    ..Shader::DEFAULT
};

#[derive(Clone, Default)]
pub(crate) struct TrapezoidText {
    trapezoidator: Trapezoidator,
}

impl TrapezoidText {
    // test api for directly drawing a glyph
    /*
    pub(crate) fn draw_char(&mut self, cx: &mut Cx, c: char, font_id: usize, font_size: f32) {
        // now lets make a draw_character function
        let trapezoids = {
            let cxfont = &cx.fonts[font_id];
            let font = cxfont.font_loaded.as_ref().unwrap();

            let slot = if c < '\u{10000}' {
                cx.fonts[font_id].font_loaded.as_ref().unwrap().char_code_to_glyph_index_map[c as usize]
            } else {
                0
            };

            if slot == 0 {
                return;
            }
            let glyph = &cx.fonts[font_id].font_loaded.as_ref().unwrap().glyphs[slot];
            let dpi_factor = cx.current_dpi_factor;
            let pos = cx.get_draw_pos();
            let font_scale_logical = font_size * 96.0 / (72.0 * font.units_per_em);
            let font_scale_pixels = font_scale_logical * dpi_factor;
            let mut trapezoids = Vec::new();
            let trapezoidate = self.trapezoidator.trapezoidate(
                glyph
                    .outline
                    .commands()
                    .map({
                        move |command| {
                            command.transform(
                                &AffineTransformation::identity()
                                    .translate(Vector::new(-glyph.bounds.p_min.x, -glyph.bounds.p_min.y))
                                    .uniform_scale(font_scale_pixels)
                                    .translate(Vector::new(pos.x, pos.y)),
                            )
                        }
                    })
                    .linearize(0.5),
            );
            if let Some(trapezoidate) = trapezoidate {
                trapezoids.extend_from_internal_iter(trapezoidate);
            }
            trapezoids
        };
        for trapezoid in trapezoids {
            let data =
                [trapezoid.xs[0], trapezoid.xs[1], trapezoid.ys[0], trapezoid.ys[1], trapezoid.ys[2], trapezoid.ys[3], 3.0];
            many.instances.extend_from_slice(&data);
        }
    }
    */

    // atlas drawing function used by CxAfterDraw
    fn draw_todo(&mut self, cx: &mut Cx, todo: CxFontsAtlasTodo, instances: &mut Vec<(Trapezoid, f32)>) {
        let mut size = 1.0;
        for i in 0..3 {
            if i == 1 {
                size = 0.75;
            }
            if i == 2 {
                size = 0.6;
            }
            let read_fonts = cx.fonts_data.read().unwrap();
            let trapezoids = {
                let cxfont = &read_fonts.fonts[todo.font_id];
                let font = cxfont.font_loaded.as_ref().unwrap();
                let atlas_page = &cxfont.atlas_pages[todo.atlas_page_id];
                let glyph = &font.glyphs[todo.glyph_id];

                if todo.glyph_id == font.char_code_to_glyph_index_map[10]
                    || todo.glyph_id == font.char_code_to_glyph_index_map[9]
                    || todo.glyph_id == font.char_code_to_glyph_index_map[13]
                {
                    return;
                }

                let glyphtc = atlas_page.atlas_glyphs[todo.glyph_id][todo.subpixel_id].unwrap();
                let texture_size = read_fonts.fonts_atlas.texture_size;
                let tx = glyphtc.tx1 * texture_size.x + todo.subpixel_x_fract * atlas_page.dpi_factor;
                let ty = 1.0 + glyphtc.ty1 * texture_size.y - todo.subpixel_y_fract * atlas_page.dpi_factor;

                let font_scale_logical = atlas_page.font_size * 96.0 / (72.0 * font.units_per_em);
                let font_scale_pixels = font_scale_logical * atlas_page.dpi_factor;
                assert!(font_scale_logical > 0.);
                assert!(font_scale_pixels > 0.);
                let mut trapezoids = Vec::new();
                let trapezoidate = self.trapezoidator.trapezoidate(
                    glyph
                        .outline
                        .commands()
                        .map({
                            move |command| {
                                command.transform(
                                    &AffineTransformation::identity()
                                        .translate(Vector::new(-glyph.bounds.p_min.x, -glyph.bounds.p_min.y))
                                        .uniform_scale(font_scale_pixels * size)
                                        .translate(Vector::new(tx, ty)),
                                )
                            }
                        })
                        .linearize(0.5),
                );
                if let Some(trapezoidate) = trapezoidate {
                    trapezoidate.for_each(&mut |item| {
                        trapezoids.push(item);
                        true
                    });
                }
                trapezoids
            };
            for trapezoid in trapezoids {
                instances.push((trapezoid, i as f32));
            }
        }
    }
}

/// Some font-related stuff gets drawn at the end of each draw cycle.
///
/// TODO(JP): This feels pretty arbitrary / one-off; find a way to better integrate this into the
/// normal draw cycle.
pub struct CxAfterDraw {
    pub(crate) trapezoid_text: TrapezoidText,
    pub(crate) atlas_pass: Pass,
    pub(crate) atlas_view: View,
    pub(crate) atlas_texture_handle: TextureHandle,
    pub(crate) counter: usize,
}

impl CxAfterDraw {
    pub fn new(cx: &mut Cx) -> Self {
        let atlas_texture_handle = {
            let mut texture = Texture::default();
            let texture_handle = texture.get_color(cx);

            let mut fonts_atlas = &mut cx.fonts_data.write().unwrap().fonts_atlas;
            fonts_atlas.texture_size = Vec2 { x: 2048.0, y: 2048.0 };
            fonts_atlas.texture_handle = Some(texture_handle);

            texture_handle
        };

        Self {
            counter: 0,
            trapezoid_text: TrapezoidText::default(),
            atlas_pass: Pass::default(),
            atlas_view: View::default(),
            atlas_texture_handle,
        }
    }

    pub fn after_draw(&mut self, cx: &mut Cx) {
        //let start = Cx::profile_time_ns();

        // we need to start a pass that just uses the texture
        if !cx.fonts_data.read().unwrap().fonts_atlas.atlas_todo.is_empty() {
            self.atlas_pass.begin_pass_without_textures(cx);
            let pass_size = cx.fonts_data.read().unwrap().fonts_atlas.texture_size;
            self.atlas_pass.set_size(cx, pass_size);
            let clear = if cx.fonts_data.read().unwrap().fonts_atlas.clear_buffer {
                cx.fonts_data.write().unwrap().fonts_atlas.clear_buffer = false;
                ClearColor::ClearWith(Vec4::default())
            } else {
                ClearColor::InitWith(Vec4::default())
            };
            self.atlas_pass.add_color_texture(cx, self.atlas_texture_handle, clear);
            let _ = self.atlas_view.begin_view(cx, LayoutSize::FILL);
            let mut atlas_todo = Vec::new();
            std::mem::swap(&mut cx.fonts_data.write().unwrap().fonts_atlas.atlas_todo, &mut atlas_todo);

            let mut instances = vec![];
            for todo in atlas_todo {
                self.trapezoid_text.draw_todo(cx, todo, &mut instances);
            }
            cx.add_instances(&SHADER, &instances);

            self.counter += 1;
            self.atlas_view.end_view(cx);
            self.atlas_pass.end_pass(cx);
        }
        //println!("TOTALT TIME {}", Cx::profile_time_ns() - start);
    }
}

#[derive(Default, Debug, Clone)]
pub(crate) struct CxFont {
    pub(crate) font_loaded: Option<zaplib_vector::font::VectorFont>,
    pub(crate) atlas_pages: Vec<CxFontAtlasPage>,
}

const ATLAS_SUBPIXEL_SLOTS: usize = 64;

#[derive(Clone, Debug)]
pub(crate) struct CxFontAtlasPage {
    dpi_factor: f32,
    font_size: f32,
    pub(crate) atlas_glyphs: Vec<[Option<CxFontAtlasGlyph>; ATLAS_SUBPIXEL_SLOTS]>,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct CxFontAtlasGlyph {
    pub(crate) tx1: f32,
    pub(crate) ty1: f32,
    pub(crate) tx2: f32,
    pub(crate) ty2: f32,
}

/// TODO(JP): subpixel_x_fract and subpixel_y_fract work in confusing (maybe even wrong) ways.
/// See https://github.com/Zaplib/zaplib/issues/175
#[derive(Default, Debug)]
pub(crate) struct CxFontsAtlasTodo {
    pub(crate) subpixel_x_fract: f32,
    pub(crate) subpixel_y_fract: f32,
    pub(crate) font_id: usize,
    pub(crate) atlas_page_id: usize,
    pub(crate) glyph_id: usize,
    pub(crate) subpixel_id: usize,
}

/// An "atlas" for font glyphs, which is like a cached version of glyphs.
#[derive(Debug, Default)]
pub(crate) struct CxFontsAtlas {
    texture_handle: Option<TextureHandle>,
    texture_size: Vec2,
    clear_buffer: bool,
    alloc_xpos: f32,
    alloc_ypos: f32,
    alloc_hmax: f32,
    pub(crate) atlas_todo: Vec<CxFontsAtlasTodo>,
}

/// Get the page id for a particular font_id/dpi_factor/font_size combination.
///
/// Returns a read lock in addition to the page id, since you typically need to read more stuff out of
/// `fonts_data`, and this avoids you having to get another lock after this.
pub fn get_font_atlas_page_id(
    fonts_data: &RwLock<CxFontsData>,
    font_id: usize,
    dpi_factor: f32,
    font_size: f32,
) -> (usize, RwLockReadGuard<CxFontsData>) {
    let fonts_data_read_lock = fonts_data.read().unwrap();
    for (index, sg) in fonts_data_read_lock.fonts[font_id].atlas_pages.iter().enumerate() {
        #[allow(clippy::float_cmp)]
        if sg.dpi_factor == dpi_factor && sg.font_size == font_size {
            return (index, fonts_data_read_lock);
        }
    }

    let glyphs_len = match &fonts_data_read_lock.fonts[font_id].font_loaded {
        Some(font) => font.glyphs.len(),
        _ => panic!("Font not loaded {}", font_id),
    };
    drop(fonts_data_read_lock);

    let glyph_index = {
        let write_fonts_atlas_pages = &mut fonts_data.write().unwrap().fonts[font_id].atlas_pages;
        write_fonts_atlas_pages.push(CxFontAtlasPage {
            dpi_factor,
            font_size,
            atlas_glyphs: {
                let mut v = Vec::new();
                v.resize(glyphs_len, [None; ATLAS_SUBPIXEL_SLOTS]);
                v
            },
        });
        write_fonts_atlas_pages.len() - 1
    };

    (glyph_index, fonts_data.read().unwrap())
}

impl CxFontsAtlas {
    pub fn alloc_atlas_glyph(&mut self, w: f32, h: f32) -> CxFontAtlasGlyph {
        if w + self.alloc_xpos >= self.texture_size.x {
            self.alloc_xpos = 0.0;
            self.alloc_ypos += self.alloc_hmax + 1.0;
            self.alloc_hmax = 0.0;
        }
        if h + self.alloc_ypos >= self.texture_size.y {
            println!("FONT ATLAS FULL, TODO FIX THIS");
        }
        if h > self.alloc_hmax {
            self.alloc_hmax = h;
        }

        let tx1 = self.alloc_xpos / self.texture_size.x;
        let ty1 = self.alloc_ypos / self.texture_size.y;

        self.alloc_xpos += w + 1.0;

        if h > self.alloc_hmax {
            self.alloc_hmax = h;
        }

        CxFontAtlasGlyph { tx1, ty1, tx2: tx1 + (w / self.texture_size.x), ty2: ty1 + (h / self.texture_size.y) }
    }
}

/// A context object containing everything font releated. This is used in different places to render text
/// and also
#[derive(Debug, Default)]
pub struct CxFontsData {
    /// List of actual [`CxFont`] objects. [`Font::font_id`] represents an index in this list.
    pub(crate) fonts: Vec<CxFont>,
    /// See [`CxFontsAtlas`].
    pub(crate) fonts_atlas: CxFontsAtlas,
}

impl CxFontsData {
    pub fn get_fonts_atlas_texture_handle(&self) -> TextureHandle {
        self.fonts_atlas.texture_handle.unwrap()
    }

    pub fn new_dummy_for_tests() -> Self {
        CxFontsData::default()
    }
}
