use zaplib::*;

static SHADER: Shader = Shader {
    build_geom: Some(QuadIns::build_geom),
    code_to_concatenate: &[
        Cx::STD_SHADER,
        code_fragment!(
            r#"
            uniform rect_size: vec2;
            uniform use_screen_space: float;
            uniform point_style: float;
            uniform vertex_transform: mat4;

            geometry geom: vec2;

            instance in_pos: vec3;
            instance in_color: vec3;
            instance in_size: float;
            instance in_user_info: vec2;

            // Transforms a vertex to clip space, accounting for aspect ratio
            fn to_clip_space(v: vec4) -> vec4 {
                let w = draw_clip.z - draw_clip.x;
                let h = draw_clip.w - draw_clip.y;
                let aspect = w / h;
                return v / v.w * aspect;
            }

            fn vertex() -> vec4 {
                if use_screen_space == 1. {
                    let projected_pos = camera_projection * camera_view * vertex_transform * vec4(in_pos, 1.0);
                    let point_size = in_size * dpi_factor;
                    let offset = point_size * vec4((geom - vec2(0.5, 0.5))/rect_size, 0, 0);

                    // When rendering screen space points, we convert the projected point to clip space
                    // and then apply the offset.
                    return to_clip_space(projected_pos) + offset;
                } else {
                    let view_pos = camera_view * vertex_transform * vec4(in_pos, 1.0);
                    let point_size = in_size;
                    let offset = point_size * vec4(geom - vec2(0.5, 0.5), 0, 0);

                    // For world space points, we apply the offset in view space so they always
                    // face to the camera.
                    return camera_projection * (view_pos + offset);
                }
            }

            fn pixel() -> vec4 {
                if point_style == 1. {
                    let df = Df::viewport(geom);
                    df.circle(vec2(0.5), 0.5);
                    df.fill(vec4(in_color, 1.));
                    return df.result;
                } else {
                    return vec4(in_color, 1.);
                }
            }"#
        ),
    ],
    ..Shader::DEFAULT
};

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DrawPoints3dInstance {
    pub position: Vec3,
    pub color: Vec3,
    pub size: f32,
    /// Not really used for rendering, this extra field can hold context
    /// sensitive information that identifies each instance.
    pub user_info: Vec2,
}

#[repr(C)]
struct DrawPoints3dUniforms {
    rect_size: Vec2,
    use_screen_space: f32,
    point_style: f32,
    vertex_transform: Mat4,
}

#[derive(Debug, Clone, Copy)]
pub enum DrawPoints3dStyle {
    Quad,
    Circle,
}

const POINT_STYLE_QUAD: f32 = 0.0;
const POINT_STYLE_CIRCLE: f32 = 1.0;

pub struct DrawPoints3dOptions {
    pub use_screen_space: bool,
    pub point_style: DrawPoints3dStyle,
    /// Custom transformation to do on all vertices
    pub vertex_transform: Mat4,
}

impl Default for DrawPoints3dOptions {
    fn default() -> Self {
        Self { use_screen_space: false, point_style: DrawPoints3dStyle::Quad, vertex_transform: Mat4::identity() }
    }
}

pub struct DrawPoints3d {}

impl DrawPoints3d {
    /// Draw points markers.
    /// Following Webviz's implementation, points can be rendered in either world or screen space using the `use_screen_space`
    /// flag. Regardless of the render space, all points are rendered as billboards, facing the camera.
    pub fn draw(cx: &mut Cx, data: &[DrawPoints3dInstance], options: DrawPoints3dOptions) -> Area {
        let area = cx.add_instances(&SHADER, data);

        let rect = cx.get_box_rect();
        area.write_user_uniforms(
            cx,
            DrawPoints3dUniforms {
                rect_size: rect.size,
                use_screen_space: if options.use_screen_space { 1. } else { 0. },
                point_style: match options.point_style {
                    DrawPoints3dStyle::Quad => POINT_STYLE_QUAD,
                    DrawPoints3dStyle::Circle => POINT_STYLE_CIRCLE,
                },
                vertex_transform: options.vertex_transform,
            },
        );

        area
    }
}
