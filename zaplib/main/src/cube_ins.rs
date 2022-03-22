//! Drawing 3d cubes.

use crate::*;

/// Draw a cube; similar to [`crate::QuadIns`]. Is currently not used much, so mostly
/// for demonstration purposes.
#[derive(Clone, Copy)]
#[repr(C)]
pub struct CubeIns {
    /// Raw transform matrix for the cube.
    pub transform: Mat4,
    /// Dimensions of the cube.
    pub cube_size: Vec3,
    /// Position in 3d space.
    pub cube_pos: Vec3,
}

/// Common [`Shader`] code for using [`CubeIns`].
pub const DRAWCUBE_SHADER_PRELUDE: CodeFragment = code_fragment!(
    r#"
    instance transform: mat4;
    instance cube_size: vec3;
    instance cube_pos: vec3;
"#
);

/*
 Example shader (using cube 3d geometry):
    varying lit_col: vec4;

    fn vertex() -> vec4 {
        let normal_matrix = mat3(transform);
        let normal = normalize(normal_matrix * geom_normal);
        let dp = abs(normal.z);

        lit_col = vec4(color.rgb * dp, color.a);
        return camera_projection * (camera_view * transform * vec4(
            geom_pos.x * cube_size.x + cube_pos.x,
            geom_pos.y * cube_size.y + cube_pos.y,
            geom_pos.z * cube_size.z + cube_pos.z + draw_zbias,
            1.
        ));
    }

    fn pixel() -> vec4 {
        return lit_col;
    }
*/
