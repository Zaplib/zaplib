//! For creating different rendering contexts ([`Pass`]es).

use crate::*;

/// A rendering context e.g. for doing 3d rendering.
///
/// Useful if you need:
/// * A different set of top-level uniforms (see [`PassUniforms`]).
/// * To render into a [`Texture`], which you can then manipulate to your liking.
///
/// [`Pass`]es are nested, with each [`CxWindow`] having a [`CxWindow::main_pass_id`]
/// that renders directly to the screen.
#[derive(Default, Clone)]
pub struct Pass {
    pub pass_id: Option<usize>,
}

impl Pass {
    /// This starts a [`Pass`].
    ///
    /// This will automatically add color and depth [`Texture`]s, since most of the time that is what you'd want. If not,
    /// then please use [`Pass::begin_pass_without_textures`].
    ///
    /// Note that if you don't call this at all during a draw cycle, then the [`Pass`]
    /// will stick around and you can still e.g. change its camera using [`Pass::set_matrix_mode`], which will
    /// cause a repaint. Similarly you can still write to shaders within this [`Pass`] using [`Area::get_slice_mut`],
    /// which will also cause a repaint.
    ///
    /// TODO(JP): Decide whether this is a bug or a feature. At the very least there currently no way to clean up a
    /// [`Pass`], so that is definitely a bug. We might want to make that the default behavior, and have an explicit
    /// method to keep a cached [`Pass`] around?
    pub fn begin_pass(&mut self, cx: &mut Cx, background_color: Vec4) {
        self.begin_pass_without_textures(cx);
        let cxpass = unsafe { cx.passes.get_unchecked_mut(self.pass_id.unwrap()) };
        if cxpass.color_textures.is_empty() {
            let color_texture_handle = Texture::default().get_color(cx);
            self.add_color_texture(cx, color_texture_handle, ClearColor::ClearWith(background_color));
            let depth_texture_handle = Texture::default().get_depth(cx);
            self.set_depth_texture(cx, depth_texture_handle, ClearDepth::ClearWith(1.0));
        }
    }

    /// Same as [`Pass::begin_pass`], but doesn't add [`Texture`]s automatically.
    pub fn begin_pass_without_textures(&mut self, cx: &mut Cx) {
        if self.pass_id.is_none() {
            self.pass_id = Some(cx.passes.len());
            cx.passes.push(CxPass::default());
        }
        let pass_id = self.pass_id.unwrap();

        if let Some(window_id) = cx.window_stack.last() {
            if cx.windows[*window_id].main_pass_id.is_none() {
                // we are the main pass of a window
                let cxpass = &mut cx.passes[pass_id];
                cx.windows[*window_id].main_pass_id = Some(pass_id);
                cxpass.dep_of = CxPassDepOf::Window(*window_id);
                cxpass.pass_size = cx.windows[*window_id].get_inner_size();
                cx.current_dpi_factor = cx.get_delegated_dpi_factor(pass_id);
            } else if let Some(dep_of_pass_id) = cx.pass_stack.last() {
                let dep_of_pass_id = *dep_of_pass_id;
                cx.passes[pass_id].dep_of = CxPassDepOf::Pass(dep_of_pass_id);
                cx.passes[pass_id].pass_size = cx.passes[dep_of_pass_id].pass_size;
                cx.current_dpi_factor = cx.get_delegated_dpi_factor(dep_of_pass_id);
            } else {
                cx.passes[pass_id].dep_of = CxPassDepOf::None;
                cx.passes[pass_id].override_dpi_factor = Some(1.0);
                cx.current_dpi_factor = 1.0;
            }
        } else {
            cx.passes[pass_id].dep_of = CxPassDepOf::None;
            cx.passes[pass_id].override_dpi_factor = Some(1.0);
            cx.current_dpi_factor = 1.0;
        }

        let cxpass = &mut cx.passes[pass_id];
        cxpass.main_view_id = None;
        cxpass.color_textures.truncate(0);
        cx.pass_stack.push(pass_id);
    }

    pub fn override_dpi_factor(&mut self, cx: &mut Cx, dpi_factor: f32) {
        if let Some(pass_id) = self.pass_id {
            cx.passes[pass_id].override_dpi_factor = Some(dpi_factor);
            cx.current_dpi_factor = dpi_factor;
        }
    }

    pub fn set_size(&mut self, cx: &mut Cx, pass_size: Vec2) {
        debug_assert!(!pass_size.x.is_nan());
        debug_assert!(!pass_size.y.is_nan());
        let mut pass_size = pass_size;
        if pass_size.x < 1.0 {
            pass_size.x = 1.0
        };
        if pass_size.y < 1.0 {
            pass_size.y = 1.0
        };
        let cxpass = &mut cx.passes[self.pass_id.unwrap()];
        cxpass.pass_size = pass_size;
    }

    pub fn add_color_texture(&mut self, cx: &mut Cx, texture_handle: TextureHandle, clear_color: ClearColor) {
        let pass_id = self.pass_id.expect("Please call add_color_texture after begin_pass");
        let cxpass = &mut cx.passes[pass_id];
        cxpass.color_textures.push(CxPassColorTexture { texture_id: texture_handle.texture_id, clear_color })
    }

    pub fn set_depth_texture(&mut self, cx: &mut Cx, texture_handle: TextureHandle, clear_depth: ClearDepth) {
        let pass_id = self.pass_id.expect("Please call set_depth_texture after begin_pass");
        let cxpass = &mut cx.passes[pass_id];
        cxpass.depth_texture = Some(texture_handle.texture_id);
        cxpass.clear_depth = clear_depth;
    }

    pub fn set_matrix_mode(&mut self, cx: &mut Cx, pmm: PassMatrixMode) {
        if let Some(pass_id) = self.pass_id {
            let cxpass = &mut cx.passes[pass_id];
            cxpass.paint_dirty = true;
            cxpass.matrix_mode = pmm;
        }
    }

    pub fn end_pass(&mut self, cx: &mut Cx) {
        cx.pass_stack.pop();
        if !cx.pass_stack.is_empty() {
            cx.current_dpi_factor = cx.get_delegated_dpi_factor(*cx.pass_stack.last().unwrap());
        }
    }
}

/// The color to either initialize a [`Texture`] with (when rendering it for the very first time),
/// or to clear it with on every paint.
#[derive(Clone)]
pub enum ClearColor {
    InitWith(Vec4),
    ClearWith(Vec4),
}

impl Default for ClearColor {
    fn default() -> Self {
        ClearColor::ClearWith(Vec4::default())
    }
}

/// The depth to either initialize a [`Texture`] with (when rendering it for the very first time),
/// or to clear it with on every paint.
#[derive(Clone)]
pub enum ClearDepth {
    InitWith(f64),
    ClearWith(f64),
}

#[derive(Default, Clone)]
pub(crate) struct CxPassColorTexture {
    pub(crate) clear_color: ClearColor,
    pub(crate) texture_id: u32,
}

#[derive(Default, Clone)]
#[repr(C, align(8))]
pub(crate) struct PassUniforms {
    /// The projection matrix; see e.g. <https://en.wikipedia.org/wiki/3D_projection>
    camera_projection: [f32; 16],
    /// The view matrix; see e.g. <https://en.wikipedia.org/wiki/Camera_matrix>
    camera_view: [f32; 16],
    /// The inverse rotation matrix for a camera. Useful for working with billboards.
    inv_camera_rot: [f32; 16],
    /// More commonly known as the "device pixel ratio". TODO(JP): Rename?
    dpi_factor: f32,
    /// Some amount by which to thicken lines, that depends on the "device pixel ratio"
    /// ([`PassUniforms`]).
    ///
    /// TODO(JP): What does this accomplish exactly? Do we need to compute this globally
    /// or can we make this a helper? It only seems to really be used in text rendering?
    dpi_dilate: f32,
}

impl PassUniforms {
    pub fn as_slice(&self) -> &[f32; std::mem::size_of::<PassUniforms>()] {
        unsafe { std::mem::transmute(self) }
    }
}

/// Standard types of projection matrices.
///
/// See e.g. <https://en.wikipedia.org/wiki/3D_projection>
#[derive(Clone)]
pub enum PassMatrixMode {
    Ortho,
    Projection { fov_y: f32, near: f32, far: f32, cam: Mat4 },
}

#[derive(Clone)]
pub(crate) struct CxPass {
    pub(crate) matrix_mode: PassMatrixMode,
    pub(crate) color_textures: Vec<CxPassColorTexture>,
    pub(crate) depth_texture: Option<u32>,
    pub(crate) clear_depth: ClearDepth,
    pub(crate) override_dpi_factor: Option<f32>,
    pub(crate) main_view_id: Option<usize>,
    pub(crate) dep_of: CxPassDepOf,
    pub(crate) paint_dirty: bool,
    pub(crate) pass_size: Vec2,
    pub(crate) pass_uniforms: PassUniforms,
    pub(crate) zbias_step: f32,
    #[allow(dead_code)] // Not used in all platforms currently.
    pub(crate) platform: CxPlatformPass,
}

impl Default for CxPass {
    fn default() -> Self {
        CxPass {
            matrix_mode: PassMatrixMode::Ortho,
            zbias_step: 0.001,
            pass_uniforms: PassUniforms::default(),
            color_textures: Vec::new(),
            depth_texture: None,
            override_dpi_factor: None,
            clear_depth: ClearDepth::ClearWith(1.0),
            main_view_id: None,
            dep_of: CxPassDepOf::None,
            paint_dirty: false,
            pass_size: Vec2::default(),
            platform: CxPlatformPass::default(),
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) enum CxPassDepOf {
    Window(usize),
    Pass(usize),
    None,
}

impl CxPass {
    fn uniform_camera_projection(&mut self, v: &Mat4) {
        //dump in uniforms
        for i in 0..16 {
            self.pass_uniforms.camera_projection[i] = v.v[i];
        }
    }

    fn uniform_camera_view(&mut self, v: &Mat4) {
        //dump in uniforms
        for i in 0..16 {
            self.pass_uniforms.camera_view[i] = v.v[i];
        }
    }

    fn uniform_inv_camera_rot(&mut self, v: &Mat4) {
        //dump in uniforms
        for i in 0..16 {
            self.pass_uniforms.inv_camera_rot[i] = v.v[i];
        }
    }

    pub(crate) fn set_dpi_factor(&mut self, dpi_factor: f32) {
        let dpi_dilate = (2. - dpi_factor).max(0.).min(1.);
        self.pass_uniforms.dpi_factor = dpi_factor;
        self.pass_uniforms.dpi_dilate = dpi_dilate;
    }

    pub(crate) fn set_matrix(&mut self, offset: Vec2, size: Vec2) {
        match self.matrix_mode {
            PassMatrixMode::Ortho => {
                let ortho = Mat4::ortho(offset.x, offset.x + size.x, offset.y, offset.y + size.y, 100., -100., 1.0, 1.0);
                self.uniform_camera_projection(&ortho);
                self.uniform_camera_view(&Mat4::identity());
                self.uniform_inv_camera_rot(&Mat4::identity());
            }
            PassMatrixMode::Projection { fov_y, near, far, cam } => {
                let proj = Mat4::perspective(fov_y, size.x / size.y, near, far);
                self.uniform_camera_projection(&proj);
                self.uniform_camera_view(&cam);

                // Computes the camera's inverse rotation matrix by transposing the rotation values, which
                // is faster than computing the inverse matrix. This takes advantage of the fact that
                // rotation matrices are orthogonal, meaning that their inverse is equal to their tranpose.
                self.uniform_inv_camera_rot(&cam.as_rotation().transpose());
            }
        };
    }
}
