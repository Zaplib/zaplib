//! Drawing text.

use std::sync::RwLock;

use crate::*;

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct TextIns {
    /// Texture coordinates for the bottom-left corner of the glyph in the texture atlas
    pub font_t1: Vec2,
    /// Texture coordinates for the top-right corner of the glyph in the texture atlas
    pub font_t2: Vec2,
    /// Color for a glyph, usually set at the same color as [`TextIns`]
    pub color: Vec4,
    /// Glyph position in view space
    pub rect_pos: Vec2,
    /// Glyph size in view space
    pub rect_size: Vec2,
    /// Depth offset (prevents z-fighting)
    pub char_depth: f32,
    /// Position used in [`TextIns::closest_offset`].
    pub base: Vec2,
    /// Font size in pixels
    pub font_size: f32,
    /// Character index in the text string
    pub char_offset: f32,
    /// TODO(JP): document.
    pub marker: f32,
}

#[repr(C)]
struct TextInsUniforms {
    brightness: f32,
    curve: f32,
}

pub static TEXT_INS_SHADER: Shader = Shader {
    build_geom: Some(QuadIns::build_geom),
    code_to_concatenate: &[
        Cx::STD_SHADER,
        code_fragment!(
            r#"
            uniform brightness: float;
            uniform curve: float;

            texture texture: texture2D;

            instance font_t1: vec2;
            instance font_t2: vec2;
            instance color: vec4;
            instance rect_pos: vec2;
            instance rect_size: vec2;
            instance char_depth: float;
            instance base: vec2;
            instance font_size: float;
            instance char_offset: float;
            instance marker: float;

            geometry geom: vec2;

            varying tex_coord1: vec2;
            varying tex_coord2: vec2;
            varying tex_coord3: vec2;
            varying clipped: vec2;

            fn get_color() -> vec4 {
                return color;
            }

            fn pixel() -> vec4 {
                let dx = dFdx(vec2(tex_coord1.x * 2048.0, 0.)).x;
                let dp = 1.0 / 2048.0;

                // basic hardcoded mipmapping so it stops 'swimming' in VR
                // mipmaps are stored in red/green/blue channel
                let s = 1.0;

                if dx > 7.0 {
                    s = 0.7;
                }
                else if dx > 2.75 {
                    s = (
                        sample2d(texture, tex_coord3.xy + vec2(0., 0.)).z
                            + sample2d(texture, tex_coord3.xy + vec2(dp, 0.)).z
                            + sample2d(texture, tex_coord3.xy + vec2(0., dp)).z
                            + sample2d(texture, tex_coord3.xy + vec2(dp, dp)).z
                    ) * 0.25;
                }
                else if dx > 1.75 {
                    s = sample2d(texture, tex_coord3.xy).z;
                }
                else if dx > 1.3 {
                    s = sample2d(texture, tex_coord2.xy).y;
                }
                else {
                    s = sample2d(texture, tex_coord1.xy).x;
                }

                s = pow(s, curve);
                let col = get_color(); //color!(white);//get_color();
                return vec4(s * col.rgb * brightness * col.a, s * col.a);
            }

            fn vertex() -> vec4 {
                let min_pos = vec2(rect_pos.x, rect_pos.y);
                let max_pos = vec2(rect_pos.x + rect_size.x, rect_pos.y - rect_size.y);

                clipped = clamp(
                    mix(min_pos, max_pos, geom) - draw_scroll,
                    draw_clip.xy,
                    draw_clip.zw
                );

                let normalized: vec2 = (clipped - min_pos + draw_scroll) / vec2(rect_size.x, -rect_size.y);
                //rect = vec4(min_pos.x, min_pos.y, max_pos.x, max_pos.y) - draw_scroll.xyxy;

                tex_coord1 = mix(
                    font_t1.xy,
                    font_t2.xy,
                    normalized.xy
                );

                tex_coord2 = mix(
                    font_t1.xy,
                    font_t1.xy + (font_t2.xy - font_t1.xy) * 0.75,
                    normalized.xy
                );

                tex_coord3 = mix(
                    font_t1.xy,
                    font_t1.xy + (font_t2.xy - font_t1.xy) * 0.6,
                    normalized.xy
                );

                return camera_projection * (camera_view * vec4(
                    clipped.x,
                    clipped.y,
                    char_depth + draw_zbias,
                    1.
                ));
            }"#
        ),
    ],
    ..Shader::DEFAULT
};

// Some constants for text anchoring
// Addition can be used to combine them together: LEFT + TOP
// Values are multipled by offsets later. For example, CENTER_H
// uses half of the horizontal offset.
pub const TEXT_ANCHOR_LEFT: Vec2 = vec2(0., 0.);
pub const TEXT_ANCHOR_CENTER_H: Vec2 = vec2(0.5, 0.);
pub const TEXT_ANCHOR_RIGHT: Vec2 = vec2(1., 0.);
pub const TEXT_ANCHOR_TOP: Vec2 = vec2(0., 0.);
pub const TEXT_ANCHOR_CENTER_V: Vec2 = vec2(0., 0.5);
pub const TEXT_ANCHOR_BOTTOM: Vec2 = vec2(0., 1.);

/// Some props for how to render the text.
#[derive(Debug)]
pub struct TextInsProps {
    /// TODO(JP): document.
    pub text_style: TextStyle,
    /// TODO(JP): document.
    pub wrapping: Wrapping,
    /// TODO(JP): document.
    pub font_scale: f32,
    /// TODO(JP): document.
    pub draw_depth: f32,
    /// TODO(JP): document.
    pub color: Vec4,
    /// By default, the position describes the top-left corner of the string
    pub position_anchoring: Vec2,
    /// See [`Padding`].
    pub padding: Padding,
}
impl TextInsProps {
    /// TODO(JP): Replace these with TextInsProps::default() when
    /// <https://github.com/rust-lang/rust/issues/67792> gets done
    pub const DEFAULT: TextInsProps = TextInsProps {
        text_style: TEXT_STYLE_NORMAL,
        wrapping: Wrapping::DEFAULT,
        font_scale: 1.0,
        draw_depth: 0.0,
        color: COLOR_WHITE,
        position_anchoring: vec2(0., 0.),
        padding: Padding::DEFAULT,
    };
}
impl Default for TextInsProps {
    fn default() -> Self {
        TextInsProps::DEFAULT
    }
}

/// Determines when to emit a set of glyphs, which has roughly the effect of
/// wrapping at these boundaries.
#[derive(Copy, Clone, Debug)]
pub enum Wrapping {
    None,
    Char,
    Word,
    /// TODO(JP): This seems to be equivalent to Wrapping::None (except for strings
    /// with specifically char code 10 as newline) because we already do a check
    /// to set emit=true and newline=true (and note that the newline=true
    /// doesn't do anything without emit=true).
    Line,
    Ellipsis(f32),
}
impl Wrapping {
    /// TODO(JP): Replace these with Wrapping::default() when
    /// <https://github.com/rust-lang/rust/issues/67792> gets done
    pub const DEFAULT: Wrapping = Wrapping::None;
}
impl Default for Wrapping {
    fn default() -> Self {
        Wrapping::DEFAULT
    }
}

#[derive(Default)]
pub struct DrawGlyphsProps {
    pub text_style: TextStyle,
    pub position_anchoring: Vec2,
}

impl TextIns {
    pub fn generate_2d_glyphs<F>(
        text_style: &TextStyle,
        fonts_data: &RwLock<CxFontsData>,
        dpi_factor: f32,
        font_scale: f32,
        draw_depth: f32,
        color: Vec4,
        pos: Vec2,
        char_offset: usize,
        chunk: &[char],
        mut char_callback: F,
    ) -> Vec<TextIns>
    where
        F: FnMut(char, usize, f32, f32) -> f32,
    {
        let mut ret = Vec::with_capacity(chunk.len());

        let font_id = text_style.font.font_id;

        let (atlas_page_id, mut read_lock) = get_font_atlas_page_id(fonts_data, font_id, dpi_factor, text_style.font_size);

        let font_size_logical =
            text_style.font_size * 96.0 / (72.0 * read_lock.fonts[font_id].font_loaded.as_ref().unwrap().units_per_em);
        let font_size_pixels = font_size_logical * dpi_factor;

        let mut x = pos.x;
        let mut char_offset = char_offset;

        for wc in chunk {
            let unicode = *wc as usize;

            // Scope the `cxfont` borrow to these variables.
            let (glyph_id, advance, w, h, min_pos_x, subpixel_x_fract, subpixel_y_fract, scaled_min_pos_x, scaled_min_pos_y) = {
                let cxfont = read_lock.fonts[font_id].font_loaded.as_ref().unwrap();
                let glyph_id = cxfont.char_code_to_glyph_index_map[unicode];
                if glyph_id >= cxfont.glyphs.len() {
                    println!("GLYPHID OUT OF BOUNDS {} {} len is {}", unicode, glyph_id, cxfont.glyphs.len());
                    continue;
                }

                let glyph = &cxfont.glyphs[glyph_id];

                let advance = glyph.horizontal_metrics.advance_width * font_size_logical * font_scale;

                // snap width/height to pixel granularity
                let w = ((glyph.bounds.p_max.x - glyph.bounds.p_min.x) * font_size_pixels).ceil() + 1.0;
                let h = ((glyph.bounds.p_max.y - glyph.bounds.p_min.y) * font_size_pixels).ceil() + 1.0;

                // this one needs pixel snapping
                let min_pos_x = x + font_size_logical * glyph.bounds.p_min.x;
                let min_pos_y = pos.y - font_size_logical * glyph.bounds.p_min.y + text_style.font_size * text_style.top_drop;

                // compute subpixel shift
                let subpixel_x_fract = min_pos_x - (min_pos_x * dpi_factor).floor() / dpi_factor;
                let subpixel_y_fract = min_pos_y - (min_pos_y * dpi_factor).floor() / dpi_factor;

                // scale and snap it
                let scaled_min_pos_x = x + font_size_logical * font_scale * glyph.bounds.p_min.x - subpixel_x_fract;
                let scaled_min_pos_y = pos.y - font_size_logical * font_scale * glyph.bounds.p_min.y
                    + text_style.font_size * font_scale * text_style.top_drop
                    - subpixel_y_fract;

                (glyph_id, advance, w, h, min_pos_x, subpixel_x_fract, subpixel_y_fract, scaled_min_pos_x, scaled_min_pos_y)
            };

            // only use a subpixel id for small fonts
            let subpixel_id = if text_style.font_size > 32.0 {
                0
            } else {
                // subtle 64 index subpixel id
                ((subpixel_y_fract * 7.0) as usize) << 3 | (subpixel_x_fract * 7.0) as usize
            };

            let tc = if let Some(tc) = read_lock.fonts[font_id].atlas_pages[atlas_page_id].atlas_glyphs[glyph_id][subpixel_id] {
                tc
            } else {
                // Drop `read_lock` to do some writes, and then reacquire it.
                drop(read_lock);
                {
                    let mut write_fonts_data = fonts_data.write().unwrap();

                    write_fonts_data.fonts_atlas.atlas_todo.push(CxFontsAtlasTodo {
                        subpixel_x_fract,
                        subpixel_y_fract,
                        font_id,
                        atlas_page_id,
                        glyph_id,
                        subpixel_id,
                    });

                    let new_glyph = write_fonts_data.fonts_atlas.alloc_atlas_glyph(w, h);
                    write_fonts_data.fonts[font_id].atlas_pages[atlas_page_id].atlas_glyphs[glyph_id][subpixel_id] =
                        Some(new_glyph);
                }
                read_lock = fonts_data.read().unwrap();
                read_lock.fonts[font_id].atlas_pages[atlas_page_id].atlas_glyphs[glyph_id][subpixel_id].unwrap()
            };

            ret.push(TextIns {
                font_t1: vec2(tc.tx1, tc.ty1),
                font_t2: vec2(tc.tx2, tc.ty2),
                color,
                rect_pos: vec2(scaled_min_pos_x, scaled_min_pos_y),
                rect_size: vec2(w * font_scale / dpi_factor, h * font_scale / dpi_factor),
                char_depth: draw_depth + 0.00001 * min_pos_x,
                base: vec2(x, pos.y),
                font_size: text_style.font_size,
                char_offset: char_offset as f32,

                // give the callback a chance to do things
                marker: char_callback(*wc, char_offset, x, advance),
            });

            x += advance;
            char_offset += 1;
        }

        ret
    }

    pub fn set_color(cx: &mut Cx, area: Area, color: Vec4) {
        let glyphs = area.get_slice_mut::<TextIns>(cx);
        for glyph in glyphs {
            glyph.color = color;
        }
    }

    fn write_uniforms(cx: &mut Cx, area: &Area, text_style: &TextStyle) {
        if area.is_first_instance() {
            let texture_handle = cx.fonts_data.read().unwrap().get_fonts_atlas_texture_handle();
            area.write_texture_2d(cx, "texture", texture_handle);
            area.write_user_uniforms(cx, TextInsUniforms { brightness: text_style.brightness, curve: text_style.curve });
        }
    }

    pub fn draw_glyphs(cx: &mut Cx, glyphs: &[TextIns], props: &DrawGlyphsProps) -> Area {
        let area = if props.position_anchoring != vec2(0., 0.) {
            // The horizontal offset is based on the total size of the string
            let horizontal_offset = glyphs.iter().map(|g| g.rect_size.x).sum();
            let vertical_offset = {
                // The vertical offset is the logical size of the font
                let text_style = TextInsProps::DEFAULT.text_style;
                text_style.font_size * text_style.top_drop
            };
            let offset = vec2(horizontal_offset, vertical_offset);
            let anchor_offset = offset * props.position_anchoring;

            let moved_glyphs: Vec<TextIns> = glyphs
                .iter()
                .map(|g| {
                    let mut g = g.clone();
                    g.rect_pos -= anchor_offset; // Offset must be subtracted
                    g
                })
                .collect();
            cx.add_instances(&TEXT_INS_SHADER, &moved_glyphs)
        } else {
            cx.add_instances(&TEXT_INS_SHADER, glyphs)
        };
        Self::write_uniforms(cx, &area, &props.text_style);
        area
    }

    pub fn draw_glyphs_with_scroll_sticky(
        cx: &mut Cx,
        glyphs: &[TextIns],
        text_style: &TextStyle,
        horizontal: bool,
        vertical: bool,
    ) -> Area {
        let area = cx.add_instances_with_scroll_sticky(&TEXT_INS_SHADER, glyphs, horizontal, vertical);
        Self::write_uniforms(cx, &area, text_style);
        area
    }

    pub fn draw_str(cx: &mut Cx, text: &str, pos: Vec2, props: &TextInsProps) -> Area {
        let glyphs = Self::generate_2d_glyphs(
            &props.text_style,
            &cx.fonts_data,
            cx.current_dpi_factor,
            props.font_scale,
            props.draw_depth,
            props.color,
            pos,
            0,
            &text.chars().collect::<Vec<char>>(),
            |_, _, _, _| 0.0,
        );

        Self::draw_glyphs(
            cx,
            &glyphs,
            &DrawGlyphsProps { text_style: props.text_style, position_anchoring: props.position_anchoring },
        )
    }

    /// TODO(JP): This doesn't seem to work well with [`Direction::Down`] (or other directions for
    /// that matter). Not a high priority but might good to be aware of.
    ///
    /// [`TextInsProps::position_anchoring`] is ignored by this function.
    pub fn draw_walk(cx: &mut Cx, text: &str, props: &TextInsProps) -> Area {
        let mut width = 0.0;
        let mut elipct = 0;

        let text_style = &props.text_style;
        let font_size = text_style.font_size;
        let line_spacing = text_style.line_spacing;
        let height_factor = text_style.height_factor;
        let mut iter = text.chars().peekable();

        let font_id = text_style.font.font_id;
        let font_size_logical = text_style.font_size * 96.0
            / (72.0 * cx.fonts_data.read().unwrap().fonts[font_id].font_loaded.as_ref().unwrap().units_per_em);

        let mut buf = Vec::with_capacity(text.len());
        let mut glyphs: Vec<TextIns> = Vec::with_capacity(text.len());

        cx.begin_row(Width::Compute, Height::Compute);
        cx.begin_padding_box(props.padding);
        cx.begin_wrapping_box();

        while let Some(c) = iter.next() {
            let last = iter.peek().is_none();

            let mut emit = last;
            let mut newline = false;
            let slot = if c < '\u{10000}' {
                cx.fonts_data.read().unwrap().fonts[font_id].font_loaded.as_ref().unwrap().char_code_to_glyph_index_map
                    [c as usize]
            } else {
                0
            };
            if c == '\n' {
                emit = true;
                newline = true;
            }
            if slot != 0 {
                let read_fonts = &cx.fonts_data.read().unwrap().fonts;
                let glyph = &read_fonts[font_id].font_loaded.as_ref().unwrap().glyphs[slot];
                width += glyph.horizontal_metrics.advance_width * font_size_logical * props.font_scale;
                match props.wrapping {
                    Wrapping::Char => {
                        buf.push(c);
                        emit = true
                    }
                    Wrapping::Word => {
                        buf.push(c);
                        if c == ' ' || c == '\t' || c == ',' || c == '\n' {
                            emit = true;
                        }
                    }
                    Wrapping::Line => {
                        buf.push(c);
                        if c == 10 as char || c == 13 as char {
                            emit = true;
                        }
                        newline = true;
                    }
                    Wrapping::None => {
                        buf.push(c);
                    }
                    Wrapping::Ellipsis(ellipsis_width) => {
                        if width > ellipsis_width {
                            // output ...
                            if elipct < 3 {
                                buf.push('.');
                                elipct += 1;
                            }
                        } else {
                            buf.push(c)
                        }
                    }
                }
            }
            if emit {
                let height = font_size * height_factor * props.font_scale;
                let rect = cx.add_box(LayoutSize { width: Width::Fix(width), height: Height::Fix(height) });

                if !rect.pos.x.is_nan() && !rect.pos.y.is_nan() {
                    glyphs.extend(Self::generate_2d_glyphs(
                        &props.text_style,
                        &cx.fonts_data,
                        cx.current_dpi_factor,
                        props.font_scale,
                        props.draw_depth,
                        props.color,
                        rect.pos,
                        0,
                        &buf,
                        |_, _, _, _| 0.0,
                    ));
                }

                width = 0.0;
                buf.truncate(0);
                if newline {
                    cx.draw_new_line_min_height(font_size * line_spacing * props.font_scale);
                }
            }
        }

        cx.end_wrapping_box();
        cx.end_padding_box();
        cx.end_row();

        Self::draw_glyphs(
            cx,
            &glyphs,
            &DrawGlyphsProps {
                text_style: *text_style,
                // Position anchoring is ignored when using walk
                ..DrawGlyphsProps::default()
            },
        )
    }

    /// Looks up text with the behavior of a text selection mouse cursor.
    pub fn closest_offset(cx: &Cx, area: &Area, pos: Vec2, line_spacing: f32) -> Option<usize> {
        if let Area::InstanceRange(instance) = area {
            if instance.instance_count == 0 {
                return None;
            }
        }

        let scroll_pos = area.get_scroll_pos(cx);
        let spos = Vec2 { x: pos.x + scroll_pos.x, y: pos.y + scroll_pos.y };

        let glyphs = area.get_slice::<TextIns>(cx);
        let mut i = 0;
        let len = glyphs.len();
        while i < len {
            let glyph = &glyphs[i];
            if glyph.base.y + glyph.font_size * line_spacing > spos.y {
                // Find a matching character within this line.
                while i < len {
                    let glyph = &glyphs[i];
                    let width = glyph.rect_size.x;
                    if glyph.base.x > spos.x + width * 0.5 || glyph.base.y > spos.y {
                        let prev_glyph = &glyphs[if i == 0 { 0 } else { i - 1 }];
                        let prev_width = prev_glyph.rect_size.x;
                        if i < len - 1 && prev_glyph.base.x > spos.x + prev_width {
                            // fix newline jump-back
                            return Some(glyph.char_offset as usize);
                        }
                        return Some(prev_glyph.char_offset as usize);
                    }
                    i += 1;
                }
            }
            i += 1;
        }
        Some(glyphs[len - 1].char_offset as usize)
    }

    pub fn get_monospace_base(cx: &Cx, text_style: &TextStyle) -> Vec2 {
        let font_id = text_style.font.font_id;
        let read_fonts = &cx.fonts_data.read().unwrap().fonts;
        let font = read_fonts[font_id].font_loaded.as_ref().unwrap();
        let slot = font.char_code_to_glyph_index_map[33];
        let glyph = &font.glyphs[slot];

        //let font_size = if let Some(font_size) = font_size{font_size}else{self.font_size};
        Vec2 { x: glyph.horizontal_metrics.advance_width * (96.0 / (72.0 * font.units_per_em)), y: text_style.line_spacing }
    }
}
