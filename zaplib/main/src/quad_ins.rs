//! Drawing rectangles; by far the most commonly used `Draw*` struct.

use crate::*;

/// [`QuadIns`] is the basis for most draw structs.This renders a rectangle.
/// There are some default shaders available at [`QuadIns::SHADER`].
///
/// Example usage with your own struct:
///
/// ```ignore
/// struct MyStruct {
///   pub base: QuadIns,
///   pub field1: f32,
///   pub field2: f32,
/// }
/// ```
///
/// And render using:
///
/// ```ignore
/// let s = MyStruct {
///   base: QuadIns::from_rect(rect),
///   field1: 0.0,
///   field2: 0.0,
/// };
/// cx.add_instances(&SHADER, &[s]);
/// ```
#[derive(Clone, Copy, Default)]
#[repr(C)]
pub struct QuadIns {
    /// The top-left corner position of the quad, in absolute coordinates.
    pub rect_pos: Vec2,
    /// The size of the quad.
    pub rect_size: Vec2,
    /// Z-index.
    pub draw_depth: f32,
}

impl QuadIns {
    #[must_use]
    pub fn from_rect(rect: Rect) -> Self {
        debug_assert!(!rect.size.x.is_nan());
        debug_assert!(!rect.size.y.is_nan());
        Self { rect_pos: rect.pos, rect_size: rect.size, ..Default::default() }
    }

    #[must_use]
    pub fn with_draw_depth(mut self, draw_depth: f32) -> Self {
        self.draw_depth = draw_depth;
        self
    }

    #[must_use]
    pub fn rect(&self) -> Rect {
        Rect { pos: self.rect_pos, size: self.rect_size }
    }

    //ANCHOR: build_geom
    pub fn build_geom() -> Geometry {
        // First, represent each corner of the quad as a vertex,
        // with each side having a length of 1.
        let vertex_attributes = vec![
            // top left vertex
            vec2(0., 0.),
            // top right vertex
            vec2(1., 0.),
            // bottom right vertex
            vec2(1., 1.),
            // bottom left vertex
            vec2(0., 1.),
        ];
        // Group the vertices into two triangles, right triangles
        // on opposing corner coming together to share a hypotenuse.
        let indices = vec![
            // top-right triangle
            [0, 1, 2],
            // bottom-left triangle
            [2, 3, 0],
        ];
        Geometry::new(vertex_attributes, indices)
    }
    //ANCHOR_END: build_geom

    /// Common [`Shader`] code for using [`QuadIns`].
    pub const SHADER: CodeFragment = code_fragment!(
        r#"
        instance rect_pos: vec2;
        instance rect_size: vec2;
        instance draw_depth: float;
        geometry geom: vec2;
        varying pos: vec2;

        fn scroll() -> vec2 {
            return draw_scroll;
        }

        fn vertex() -> vec4 {
            let scr = scroll();

            let clipped: vec2 = clamp(
                geom * rect_size + rect_pos - scr,
                draw_clip.xy,
                draw_clip.zw
            );
            pos = (clipped + scr - rect_pos) / rect_size;
            // only pass the clipped position forward
            return camera_projection * (camera_view * vec4(
                clipped.x,
                clipped.y,
                draw_depth + draw_zbias,
                1.
            ));
        }
    "#
    );
}
