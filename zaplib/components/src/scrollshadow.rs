use zaplib::*;

#[derive(Clone, Copy, Default)]
#[repr(C)]
struct ScrollShadowIns {
    base: QuadIns,
    shadow_top: f32,
}

static SHADER: Shader = Shader {
    build_geom: Some(QuadIns::build_geom),
    code_to_concatenate: &[
        Cx::STD_SHADER,
        QuadIns::SHADER,
        code_fragment!(
            r#"
            instance shadow_top: float;
            varying is_viz: float;

            fn scroll() -> vec2 {
                if shadow_top > 0.5 {
                    is_viz = clamp(draw_local_scroll.y * 0.1, 0., 1.);
                }
                else {
                    is_viz = clamp(draw_local_scroll.x * 0.1, 0., 1.);
                }
                return draw_scroll;
            }

            // TODO make the corner overlap properly with a distance field eq.
            fn pixel() -> vec4 {
                if shadow_top > 0.5 {
                    return mix(vec4(0., 0., 0., is_viz), vec4(0., 0., 0., 0.), pow(geom.y, 0.5));
                }
                return mix(vec4(0., 0., 0., is_viz), vec4(0., 0., 0., 0.), pow(geom.x, 0.5));
            }"#
        ),
    ],
    ..Shader::DEFAULT
};

pub struct ScrollShadow;

const SHADOW_SIZE: f32 = 4.;

impl ScrollShadow {
    pub fn draw_shadow_top(cx: &mut Cx, draw_depth: f32) {
        Self::draw_shadow_top_at(cx, Rect { pos: vec2(0., 0.), size: vec2(cx.get_width_total(), 0.) }, draw_depth);
    }

    pub fn draw_shadow_left(cx: &mut Cx, draw_depth: f32) {
        Self::draw_shadow_left_at(cx, Rect { pos: vec2(0., 0.), size: vec2(0., cx.get_height_total()) }, draw_depth);
    }

    pub fn draw_shadow_top_at(cx: &mut Cx, rect: Rect, draw_depth: f32) {
        cx.add_instances_with_scroll_sticky(
            &SHADER,
            &[ScrollShadowIns {
                base: QuadIns::from_rect(Rect { pos: cx.get_box_origin() + rect.pos, size: vec2(rect.size.x, SHADOW_SIZE) })
                    .with_draw_depth(draw_depth),
                shadow_top: 1.0,
            }],
            true,
            true,
        );
    }

    pub fn draw_shadow_left_at(cx: &mut Cx, rect: Rect, draw_depth: f32) {
        cx.add_instances_with_scroll_sticky(
            &SHADER,
            &[ScrollShadowIns {
                base: QuadIns::from_rect(Rect { pos: cx.get_box_origin() + rect.pos, size: vec2(SHADOW_SIZE, rect.size.y) })
                    .with_draw_depth(draw_depth),
                shadow_top: 0.0,
            }],
            true,
            true,
        );
    }
}
