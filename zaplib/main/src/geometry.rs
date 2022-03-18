//! Instance geometries, for rendering the same shape multiple times.

use std::rc::Rc;

use crate::*;

/// Generated geometry data used for instanced rendering, represented as triangles.
///
/// For example, you can define that a quad has 4 vertices, spanning 2 triangles
/// (for use in e.g. [`crate::QuadIns`]), so you don't have to manually create
/// them every time you want to render a quad.
#[derive(Default)]
pub struct Geometry {
    vertex_attributes: Vec<f32>,
    triangle_indices: Vec<u32>,
    vertex_number_of_bytes: usize,
}
impl Geometry {
    /// Instantiates a new [`Geometry`].
    ///
    /// `vertex_attributes` correspond to "geometry" fields in the shader.
    /// Be sure to use `#[repr(C)]` in the structs you pass in. Structs can only
    /// contain fields of type [`f32`]/[`Vec2`]/[`Vec3`]/[`Vec4`].
    ///
    /// Anything that we can get an f32-slice (`[f32]`) from works here.
    ///
    /// `triangle_indices` - the indices of vertex attributes by which to render each triangle.
    /// A triangle has 3 vertices, hence we group indices in sets of 3.
    pub fn new<T: 'static + Copy>(vertex_attributes: Vec<T>, triangle_indices: Vec<[u32; 3]>) -> Self {
        Self {
            vertex_attributes: cast_vec(vertex_attributes),
            triangle_indices: cast_vec(triangle_indices),
            vertex_number_of_bytes: core::mem::size_of::<T>(),
        }
    }

    pub(crate) fn vertices_f32_slice(&self) -> &[f32] {
        &self.vertex_attributes
    }

    pub(crate) fn indices_u32_slice(&self) -> &[u32] {
        &self.triangle_indices
    }
}

/// A pointer to a [`CxGpuGeometry`] (indexed in [`Cx::gpu_geometries`] using [`GpuGeometry::gpu_geometry_id`]),
///
/// Cloning a [`GpuGeometry`] doesn't copy the underlying buffer; it just adds a reference count to the existing buffer.
///
/// The corresponding GPU buffer ([`CxGpuGeometry`]) gets marked for reuse when there are no more references to it.
///
/// TODO(JP): When creating a big [`GpuGeometry`] and then dropping it, we don't clear out the data until we reuse it
/// (which might be never). We might want to add a cleanup sweep at the end of the draw cycle. Also, in some renderers
/// we might not shrink GPU buffers when we reuse a previous [`CxGpuGeometry`].
#[derive(Clone)]
pub struct GpuGeometry {
    gpu_geometry_id: usize,

    // Not actually dead, since this increases/decreases [`CxGpuGeometry::usage_count`].
    #[allow(dead_code)]
    usage_count: Rc<()>,
}
impl GpuGeometry {
    /// Create a [`GpuGeometry`] from a [`Geometry`].
    pub fn new(cx: &mut Cx, geometry: Geometry) -> Self {
        let gpu_geometry_id =
            cx.gpu_geometries.iter().position(|gpu_geometry| gpu_geometry.usage_count() == 0).unwrap_or_else(|| {
                cx.gpu_geometries.push(Default::default());
                cx.gpu_geometries.len() - 1
            });

        let gpu_geometry = &mut cx.gpu_geometries[gpu_geometry_id];
        gpu_geometry.geometry = geometry;
        gpu_geometry.dirty = true;
        Self { gpu_geometry_id, usage_count: Rc::clone(&gpu_geometry.usage_count) }
    }

    pub(crate) fn get_id(cx: &Cx, view_id: usize, draw_call_id: usize) -> usize {
        let cxview = &cx.views[view_id];
        let draw_call = &cxview.draw_calls[draw_call_id];
        let sh = &cx.shaders[draw_call.shader_id];

        let gpu_geometry_id = if let Some(gpu_geometry) = &draw_call.props.gpu_geometry {
            gpu_geometry.gpu_geometry_id
        } else if let Some(gpu_geometry) = &sh.gpu_geometry {
            gpu_geometry.gpu_geometry_id
        } else {
            panic!("Missing geometry");
        };

        let shader_bytes_geom = sh.mapping.geometry_props.total_slots * std::mem::size_of::<f32>();
        let vertex_number_of_bytes = cx.gpu_geometries[gpu_geometry_id].geometry.vertex_number_of_bytes;
        let sh_name = &sh.name;
        assert_eq!(
            shader_bytes_geom, vertex_number_of_bytes,
            "Mismatch between shader instance slots ({shader_bytes_geom} bytes) and vertex struct ({vertex_number_of_bytes} \
             bytes) in shader {sh_name}"
        );

        gpu_geometry_id
    }
}

/// The base fields used for instance rendering. Created from [`Geometry`].
#[derive(Default)]
pub(crate) struct CxGpuGeometry {
    pub(crate) geometry: Geometry,
    pub(crate) dirty: bool,
    usage_count: Rc<()>,
    pub(crate) platform: CxPlatformGpuGeometry,
}

impl CxGpuGeometry {
    /// Get the number of [`GpuGeometry`] objects that hold a reference to this.
    ///
    /// Note that this excludes the reference that [`CxGpuGeometry`] itself holds;
    /// hence the `- 1`.
    pub(crate) fn usage_count(&self) -> usize {
        Rc::strong_count(&self.usage_count) - 1
    }
}
