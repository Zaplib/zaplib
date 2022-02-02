use crate::*;
use zaplib::*;

fn build_geom() -> Geometry {
    Geometry3d::cube(1.0, 1.0, 1.0, 1, 1, 1)
}

static SHADER: Shader = Shader {
    build_geom: Some(build_geom),
    code_to_concatenate: &[
        Cx::STD_SHADER,
        Geometry3d::SHADER,
        DRAWCUBE_SHADER_PRELUDE,
        code_fragment!(
            r#"
            const sky_color: vec4 = #000000;
            const edge_color: vec4 = #111111;
            const floor_color: vec4 = #8;

            fn color_form_id() -> vec4 {
                if geom_id>4.5 {
                    return #f00;
                }
                if geom_id>3.5 {
                    return #0f0;
                }
                if geom_id>2.5 {
                    return #00f;
                }
                if geom_id>1.5 {
                    return #0ff;
                }
                return #f0f;
            }
            varying t:float;
            fn vertex() -> vec4 {

                let model_view = camera_view * transform ;
                return camera_projection * (model_view * vec4(
                    geom_pos.x * cube_size.x + cube_pos.x,
                    geom_pos.y * cube_size.y + cube_pos.y,
                    geom_pos.z * cube_size.z + cube_pos.z + draw_zbias,
                    1.
                ));
            }

            fn pixel() -> vec4 {
                let x = geom_uv.x;
                let y = geom_uv.y;
                // walls
                let sky = sky_color;
                let edge = edge_color;
                if geom_id>4.5 || geom_id > 3.5 || geom_id < 1.5 {
                    return mix(edge, sky, y);
                }
                // floor
                if geom_id>2.5 {
                    let coord = geom_uv * 150.0;
                    let grid = abs(
                        fract(coord - 0.5) - 0.5
                    ) / (abs(dFdx(coord)) + abs(dFdy(coord)));
                    let line = min(grid.x, grid.y);
                    let grid2 = floor_color + 0.4 * vec4(vec3(1.0 - min(line, 1.0)), 1.0);
                    let uv2 = abs(2.0 * geom_uv - 1.0);
                    return mix(grid2, edge, min(max(uv2.x, uv2.y) + 0.7, 1.0));
                }
                return sky;
            }"#
        ),
    ],
    ..Shader::DEFAULT
};

pub struct SkyBox;

impl SkyBox {
    pub fn draw(cx: &mut Cx, world_origin: Vec3) {
        let cube = CubeIns {
            cube_size: vec3(200., 100., 200.),
            cube_pos: vec3(0., 50., 0.) + world_origin,
            transform: Mat4::identity(),
        };

        cx.add_instances(&SHADER, &[cube]);
    }
}
