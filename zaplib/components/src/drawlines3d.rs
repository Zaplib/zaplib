use zaplib::*;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DrawLines3dInstance {
    /// Starting point for the previous line segment. We use this to compute
    /// the corner between the previous line segment and the current one.
    pub position_before: Vec3,
    /// Starting point for the current line segment. This is also the end point
    /// for the previous segment. If the current segment is the very first one
    /// in the strip, [`DrawLines3dInstance::position_before`] and
    /// [`DrawLines3dInstance::position_start`] will be the same point.
    pub position_start: Vec3,
    /// End point for the current line segment. This is also the start point
    /// for the next segment in the strip. If the current segment is the last one
    /// in the strip, both [`DrawLines3dInstance::position_end`] and [`DrawLines3dInstance::position_after`]
    /// will have the same value.
    pub position_end: Vec3,
    /// End point for the next segment in the strip. This point is used to compute
    /// the corner between the current segment and the next one.
    pub position_after: Vec3,
    /// Color for the start point in the current line segment.
    pub color_start: Vec4,
    /// Color for the end point in the current line segment. If it's different
    /// from the start color, we get a gradient effect.
    pub color_end: Vec4,
    /// Thichness of the line strip
    pub scale: f32,
}

impl DrawLines3dInstance {
    pub fn from_segment(start: Vec3, end: Vec3, color: Vec4, scale: f32) -> Self {
        Self {
            position_before: start,
            position_start: start,
            position_end: end,
            position_after: end,
            color_start: color,
            color_end: color,
            scale,
        }
    }
}

#[repr(C)]
struct DrawLines3dUniforms {
    vertex_transform: Mat4,
}

/// Renderer for line markers.
///
/// In order to support custom like thickness, we render each line segment using 2D quads
/// which are transformed accordingly. Line segments are represented by two 3D points
/// (start and end) and each of them can have different colors to support gradients. In addition,
/// each line segment also gets the point before and the point after, so we can compute the corners
/// correctly. For individual line segments, we don't need to handle the corner.
///
/// Each of the 4 points in the 2D quad is transformed by computing a normal vector multipled
/// by a thickness value.
///
/// Roughly an individual line segment looks like:
///       TL   -   -   -  .TR
///       |          ,.-' |
///     A/B - - -,.-' - - C/D
///       |  ,.-'         |
///      BL-' -   -   -   BR
/// Notice that the pairs A/B and C/D are duplicated. We can use that to avoid doing extra
/// calculation for corners (since individual line segments don't have corners)
///
/// Line segments in a strip looks like:
///       TL   -   -   -  .TR
///       |          ,.-' |
/// A - - B - - -,.-' - - C - - D
///       |  ,.-'         |
///      BL-' -   -   -   BR
///
/// Corner points are computed based on the values of the 2D position (i.e. top corner is `(0, 1)`).
/// When two adjacent segments form an obtuse angle, we draw a miter join:
///                       TR/TL.
///                  , '   _/|   ' .
///              , '     _/  |       ' .
///          , '       _/    C           ' .
///      , '         _/      |               ' .
///    TL          _/        |        ______,----'TR
///     \        _/       ,BR/BL.----'            /
///      B     _/    , '          ' .            D
///       \  _/ , '                   ' .       /
///        BL'                            ' . BR
///
/// But when the angle gets too sharp, we switch to a "fold" join, where the two segments overlap at
/// the corner:
///         ,TR/BL---C--BR/TL
///        ,    |.\__  ,     .
///       ,     | .  \,_      .
///      ,      |  . ,  \_     .
///     ,       |   ,     \__   .
///    ,        |  , .       \__ .
///   TL._      | ,   .        _.TR
///       'B._  |,     .   _.C'
///           'BL       BR'

pub struct DrawLines3dOptions {
    /// Custom transformation to do on all vertices
    pub vertex_transform: Mat4,
}

impl Default for DrawLines3dOptions {
    fn default() -> Self {
        Self { vertex_transform: Mat4::identity() }
    }
}
pub struct DrawLines3d {}

impl DrawLines3d {
    pub fn draw(cx: &mut Cx, data: &[DrawLines3dInstance], options: DrawLines3dOptions) {
        let area = cx.add_instances(&SHADER, data);
        area.write_user_uniforms(cx, DrawLines3dUniforms { vertex_transform: options.vertex_transform });
    }
}

static SHADER: Shader = Shader {
    build_geom: Some(QuadIns::build_geom),
    code_to_concatenate: &[
        Cx::STD_SHADER,
        code_fragment!(
            r#"
            geometry geom: vec2;

            uniform vertex_transform: mat4;

            instance in_pos_before: vec3;

            instance in_pos_start: vec3;

            instance in_pos_end: vec3;

            instance in_pos_after: vec3;

            instance in_color_start: vec4;
            instance in_color_end: vec4;

            instance in_scale: float;

            varying color: vec4;

            fn project(pos: vec3) -> vec4 {
                return camera_projection * camera_view * vertex_transform * vec4(pos, 1.0);
            }

            // Transforms a vertex to clip space, accounting for aspect ratio
            fn clip(v: vec4) -> vec4 {
                let w = draw_clip.z - draw_clip.x;
                let h = draw_clip.w - draw_clip.y;
                let aspect = w / h;
                return v / v.w * aspect;
            }

            fn rotate_ccw(v: vec2) -> vec2 {
                return vec2(-v.y, v.x);
            }

            fn vertex() -> vec4 {
                let is_top = geom.y == 1.;
                let is_left = geom.x == 0.;

                let pos = is_left ? in_pos_start : in_pos_end;
                color = is_left ? in_color_start : in_color_end;

                // The line thickness should be scaled the same way the camera scales other distances.
                // projection[0].xyz is the result of projecting a unit x-vector, so its length represents
                // how much distances are scaled by the camera projection.
                let thickness = 0.5 * in_scale * length(camera_projection[0].xyz);
                thickness = is_top ? thickness : -thickness;

                let projected_pos = project(pos);

                // Transform all points to clip space so calculations are done in 2D and
                // the resulting normal are already facing the camera
                let clip_before = clip(project(in_pos_before)).xy;
                let clip_start = clip(project(in_pos_start)).xy;
                let clip_end = clip(project(in_pos_end)).xy;
                let clip_after = clip(project(in_pos_after)).xy;

                // Vector comparision (i.e, `clip_before == clip_start`) is not allowed?
                // TODO(hernan): fix vector comparision
                let is_first = length(clip_before - clip_start) < 0.001;
                let is_last = length(clip_end - clip_after) < 0.001;
                let is_endpoint = is_left ? is_first : is_last;

                let dir_left = normalize(clip_start - clip_before);
                let dir_current = normalize(clip_end - clip_start);
                let dir_right = normalize(clip_after - clip_end);

                let normal_current = rotate_ccw(dir_current);
                let normal_left = normalize(rotate_ccw(dir_left));
                let normal_right = normalize(rotate_ccw(dir_right));

                if is_endpoint == true {
                    projected_pos += vec4(normal_current * thickness, 0, 0);
                    return projected_pos;
                }

                let cos_start = clamp(-dot(dir_left, dir_current), -1., 1.);
                let cos_end = clamp(-dot(dir_current, dir_right), -1., 1.);

                let too_sharp_start = cos_start > 0.01;
                let too_sharp_end = cos_end > 0.01;
                let too_sharp = is_left ? too_sharp_start : too_sharp_end;

                let normal: vec2;
                if too_sharp == true {
                    // Fold join: The resulting offset is a vector perpendicular to the bisector
                    // between the direction of the previous (or next) line segment and the current
                    // one. Folding swap top/bottom vertices, so we need to acount for the direction
                    // of the folding
                    let turning_right_start = dot(dir_left, rotate_ccw(dir_current)) > 0.;
                    let turning_right_end = dot(dir_current, rotate_ccw(dir_right)) > 0.;
                    let turning_right = is_left ? turning_right_start : turning_right_end;

                    let perp = is_left ? normal_left : normal_current;
                    let dir = is_left ? dir_left : dir_current;
                    let scale_perp = is_left ? -1. : 1.;
                    let scale_dir = (turning_right == is_left) ? 1. : -1.;
                    let tan_half_start = sqrt((1. - cos_start) / (1. + cos_start));
                    let tan_half_end = sqrt((1. - cos_end) / (1. + cos_end));
                    let tan_half = is_left ? tan_half_start : tan_half_end;
                    normal = scale_perp * perp + scale_dir * dir * tan_half;
                } else {
                    // Miter join: compute the corner normal as the vector pointing
                    // halfway between the previous (or next) segment and the current one
                    let bisector_start = rotate_ccw(normalize(dir_left + dir_current)); // angle bisector of ABC
                    let bisector_end = rotate_ccw(normalize(dir_current + dir_right)); // angle bisector of BCD
                    let bisector = is_left ? bisector_start : bisector_end;
                    let sin_half_start = sqrt((1. - cos_start) / 2.);
                    let sin_half_end = sqrt((1. - cos_end) / 2.);
                    let sin_half = is_left ? sin_half_start : sin_half_end;
                    normal = bisector / sin_half;
                }

                projected_pos += vec4(normal * thickness, 0, 0);

                return projected_pos;
            }

            fn pixel() -> vec4 {
                return color;
            }"#
        ),
    ],
    ..Shader::DEFAULT
};
