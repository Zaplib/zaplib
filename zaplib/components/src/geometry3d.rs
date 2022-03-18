use std::f32::consts::PI;

use zaplib::*;

/// Represents a single vertex used in 3d objects.
#[derive(Clone, Copy)]
#[repr(C)]
pub struct Geometry3d {
    /// Vertex position.
    geom_pos: Vec3,
    /// Some sort of identifier (dependent on the type of object).
    geom_id: f32,
    /// Vertex normal vector (perpendicular to the surface).
    geom_normal: Vec3,
    /// 2d coordinates for mapping textures (dependent on the type of object).
    geom_uv: Vec2,
}

#[derive(Clone, Copy)]
enum GeometryAxis {
    X = 0,
    Y = 1,
    Z = 2,
}

impl Geometry3d {
    /// Shader fields corresponding to the fields in [`Geometry3d`].
    ///
    /// TODO(JP): Should we make this part of the [`Geometry`] instead so they are
    /// always together, and can be automatically included?
    pub const SHADER: CodeFragment = code_fragment!(
        r#"
        geometry geom_pos: vec3;
        geometry geom_id: float;
        geometry geom_normal: vec3;
        geometry geom_uv: vec2;
    "#
    );

    /// 3d cube.
    pub fn cube(
        width: f32,
        height: f32,
        depth: f32,
        width_segments: usize,
        height_segments: usize,
        depth_segments: usize,
    ) -> Geometry {
        // TODO(JP): Would be nice to do this without any reallocations.
        let mut vertex_attributes = vec![];
        let mut indices = vec![];
        add_plane_3d(
            &mut vertex_attributes,
            &mut indices,
            GeometryAxis::Z,
            GeometryAxis::Y,
            GeometryAxis::X,
            -1.0,
            -1.0,
            depth,
            height,
            width,
            depth_segments,
            height_segments,
            0.0,
        );
        add_plane_3d(
            &mut vertex_attributes,
            &mut indices,
            GeometryAxis::Z,
            GeometryAxis::Y,
            GeometryAxis::X,
            1.0,
            -1.0,
            depth,
            height,
            -width,
            depth_segments,
            height_segments,
            1.0,
        );
        add_plane_3d(
            &mut vertex_attributes,
            &mut indices,
            GeometryAxis::X,
            GeometryAxis::Z,
            GeometryAxis::Y,
            1.0,
            1.0,
            width,
            depth,
            height,
            width_segments,
            depth_segments,
            2.0,
        );
        add_plane_3d(
            &mut vertex_attributes,
            &mut indices,
            GeometryAxis::X,
            GeometryAxis::Z,
            GeometryAxis::Y,
            1.0,
            -1.0,
            width,
            depth,
            -height,
            width_segments,
            depth_segments,
            3.0,
        );
        add_plane_3d(
            &mut vertex_attributes,
            &mut indices,
            GeometryAxis::X,
            GeometryAxis::Y,
            GeometryAxis::Z,
            1.0,
            -1.0,
            width,
            height,
            depth,
            width_segments,
            height_segments,
            4.0,
        );
        add_plane_3d(
            &mut vertex_attributes,
            &mut indices,
            GeometryAxis::X,
            GeometryAxis::Y,
            GeometryAxis::Z,
            -1.0,
            -1.0,
            width,
            height,
            -depth,
            width_segments,
            height_segments,
            5.0,
        );
        Geometry::new(vertex_attributes, indices)
    }

    /// 3d sphere.
    ///
    /// TODO(JP): `geom_id` and `geom_uv` are set to 0.0.
    pub fn sphere(num_parallels: u32, num_meridians: u32, radius: f32) -> Geometry {
        let mut vertex_attributes = Vec::with_capacity((num_parallels * num_meridians + 2) as usize);
        // TODO(JP): Would be nice to do this without any reallocations.
        let mut indices = vec![];

        // north pole
        vertex_attributes.push(Geometry3d {
            geom_pos: vec3(0., 0., radius),
            geom_id: 0.0,
            geom_normal: vec3(0., 0., radius),
            geom_uv: vec2(0., 0.),
        });

        // south pole
        vertex_attributes.push(Geometry3d {
            geom_pos: vec3(0., 0., -radius),
            geom_id: 0.0,
            geom_normal: vec3(0., 0., -radius),
            geom_uv: vec2(0., 0.),
        });

        let mut point_count: u32 = 2;

        for i in 0..num_parallels {
            for j in 0..num_meridians {
                let phi = (((i + 1) as f32) / ((num_parallels + 1) as f32)) * PI;
                let z = radius * phi.cos();
                let width = radius * phi.sin();
                let theta = (j as f32 * 2. * PI) / (num_meridians as f32);
                let x = width * theta.cos();
                let y = width * theta.sin();

                vertex_attributes.push(Geometry3d {
                    geom_pos: vec3(x, y, z),
                    geom_id: 0.0,
                    geom_normal: vec3(x, y, z),
                    geom_uv: vec2(0., 0.),
                });

                point_count += 1;

                if j > 0 {
                    let prev_meridian: u32 = if i == 0 { 0 } else { point_count - 1 - num_meridians };
                    indices.push([point_count - 2, point_count - 1, prev_meridian]);

                    if i > 0 {
                        indices.push([point_count - 2, prev_meridian - 1, prev_meridian]);
                    }
                }
            }

            let prev_meridian: u32 = if i == 0 { 0 } else { point_count - 2 * num_meridians };
            indices.push([point_count - 1, point_count - num_meridians, prev_meridian]);

            if i > 0 {
                indices.push([point_count - 1, point_count - num_meridians - 1, prev_meridian]);
            }
        }

        // connect last parallel to south pole
        for j in 0..num_meridians {
            let pt = point_count - num_meridians + j;
            let prev_pt = if j == 0 { point_count - 1 } else { pt - 1 };
            indices.push([pt, prev_pt, 1]);
        }

        Geometry::new(vertex_attributes, indices)
    }

    /// 3d cylinder or cone.
    ///
    /// TODO(JP): `geom_id` and `geom_uv` are set to 0.0.
    pub fn cylinder_or_cone(num_segments: u32, is_cone: bool) -> Geometry {
        // TODO(JP): Would be nice to do this without any reallocations.
        let mut vertex_attributes = vec![];
        let mut indices = vec![];

        // "poles" are the centers of top/bottom faces
        // north pole
        vertex_attributes.push(Geometry3d {
            geom_pos: vec3(0., 0., 0.5),
            geom_id: 0.0,
            geom_normal: vec3(0., 1., 0.),
            geom_uv: vec2(0., 0.),
        });
        // south pole
        vertex_attributes.push(Geometry3d {
            geom_pos: vec3(0., 0., -0.5),
            geom_id: 0.0,
            geom_normal: vec3(0., -1., 0.),
            geom_uv: vec2(0., 0.),
        });

        // Keep side faces separate from top/bottom to improve appearance for semi-transparent colors.
        // We don't have a good approach to transparency right now but this is a small improvement over mixing the faces.
        let mut side_faces = vec![];
        let mut end_cap_faces = vec![];

        let mut point_count = 0;

        for i in 0..num_segments {
            let theta = (2. * PI * i as f32) / num_segments as f32;
            let x = 0.5 * theta.cos();
            let y = 0.5 * theta.sin();

            vertex_attributes.push(Geometry3d {
                geom_pos: vec3(x, y, 0.5),
                geom_id: 0.0,
                geom_normal: vec3(x, y, 0.5),
                geom_uv: vec2(0., 0.),
            });
            vertex_attributes.push(Geometry3d {
                geom_pos: vec3(x, y, -0.5),
                geom_id: 0.0,
                geom_normal: vec3(x, y, -0.5),
                geom_uv: vec2(0., 0.),
            });

            point_count += 2;

            let bottom_left_pt = point_count - 1;
            let top_right_pt = if is_cone {
                0
            } else if i + 1 == num_segments {
                2
            } else {
                point_count
            };
            let bottom_right_pt = if i + 1 == num_segments { 3 } else { point_count + 1 };

            side_faces.push([bottom_left_pt, top_right_pt, bottom_right_pt]);
            end_cap_faces.push([bottom_left_pt, bottom_right_pt, 1]);

            if !is_cone {
                let top_left_pt = point_count - 2;
                side_faces.push([top_left_pt, bottom_left_pt, top_right_pt]);
                end_cap_faces.push([top_left_pt, top_right_pt, 0]);
            }
        }

        indices.extend_from_slice(&side_faces[..]);
        indices.extend_from_slice(&end_cap_faces[..]);

        Geometry::new(vertex_attributes, indices)
    }
}

// Clippy TODO
#[warn(clippy::many_single_char_names)]
fn add_plane_3d(
    vertex_attributes: &mut Vec<Geometry3d>,
    indices: &mut Vec<[u32; 3]>,
    u: GeometryAxis,
    v: GeometryAxis,
    w: GeometryAxis,
    udir: f32,
    vdir: f32,
    width: f32,
    height: f32,
    depth: f32,
    grid_x: usize,
    grid_y: usize,
    id: f32,
) {
    let segment_width = width / (grid_x as f32);
    let segment_height = height / (grid_y as f32);
    let width_half = width / 2.0;
    let height_half = height / 2.0;
    let depth_half = depth / 2.0;
    let grid_x1 = grid_x + 1;
    let grid_y1 = grid_y + 1;

    let vertex_offset = vertex_attributes.len();

    for iy in 0..grid_y1 {
        let y = (iy as f32) * segment_height - height_half;

        for ix in 0..grid_x1 {
            let x = (ix as f32) * segment_width - width_half;

            let mut vertex_attribute = Geometry3d {
                geom_pos: Default::default(),
                geom_id: id,
                geom_normal: Default::default(),
                geom_uv: vec2((ix as f32) / (grid_x as f32), 1.0 - (iy as f32) / (grid_y as f32)),
            };

            let geom_pos = vertex_attribute.geom_pos.as_mut_array();
            geom_pos[u as usize] = x * udir;
            geom_pos[v as usize] = y * vdir;
            geom_pos[w as usize] = depth_half;

            let geom_normal = vertex_attribute.geom_normal.as_mut_array();
            geom_normal[w as usize] = if depth > 0.0 { 1.0 } else { -1.0 };

            vertex_attributes.push(vertex_attribute);
        }
    }

    for iy in 0..grid_y {
        for ix in 0..grid_x {
            let a = vertex_offset + ix + grid_x1 * iy;
            let b = vertex_offset + ix + grid_x1 * (iy + 1);
            let c = vertex_offset + (ix + 1) + grid_x1 * (iy + 1);
            let d = vertex_offset + (ix + 1) + grid_x1 * iy;
            indices.push([a as u32, b as u32, d as u32]);
            indices.push([b as u32, c as u32, d as u32]);
        }
    }
}
