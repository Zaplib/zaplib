//! Drawing [`Texture`]s.

use crate::quad_ins::*;
use crate::*;

/// For drawing a [`Texture`].
#[derive(Clone, Copy)]
#[repr(C)]
pub struct ImageIns {
    base: QuadIns,
    /// TODO(JP): `pt1`, `pt2`, `alpha` are currently never used.
    pt1: Vec2,
    pt2: Vec2,
    alpha: f32,
}

static SHADER: Shader = Shader {
    build_geom: Some(QuadIns::build_geom),
    code_to_concatenate: &[
        Cx::STD_SHADER,
        QuadIns::SHADER,
        code_fragment!(
            r#"
            texture texture: texture2D;
            instance pt1: vec2;
            instance pt2: vec2;
            instance alpha: float;
            varying tc: vec2;
            varying v_pixel: vec2;
            //let dpi_dilate: float<Uniform>;

            fn vertex() -> vec4 {
                // return vec4(geom.x-0.5, geom.y, 0., 1.);
                let shift: vec2 = -draw_scroll;
                let clipped: vec2 = clamp(
                    geom * rect_size + rect_pos + shift,
                    draw_clip.xy,
                    draw_clip.zw
                );
                let pos = (clipped - shift - rect_pos) / rect_size;
                tc = mix(pt1, pt2, pos);
                v_pixel = clipped;
                // only pass the clipped position forward
                return camera_projection * vec4(clipped.x, clipped.y, draw_depth, 1.);
            }

            fn pixel() -> vec4 {
                return vec4(sample2d(texture, tc.xy).rgb * alpha, alpha);
            }"#
        ),
    ],
    ..Shader::DEFAULT
};

impl Default for ImageIns {
    fn default() -> Self {
        Self { base: Default::default(), pt1: vec2(0., 0.), pt2: vec2(1., 1.), alpha: 1.0 }
    }
}

impl ImageIns {
    pub fn draw(cx: &mut Cx, rect: Rect, texture_handle: TextureHandle) -> Area {
        let area = cx.add_instances(&SHADER, &[ImageIns { base: QuadIns::from_rect(rect), ..Default::default() }]);
        area.write_texture_2d(cx, "texture", texture_handle);
        area
    }
}
