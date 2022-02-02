//! Managing [GPU shaders](https://en.wikipedia.org/wiki/Shader).

use crate::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use zaplib_shader_compiler::error::ParseError;
use zaplib_shader_compiler::span::{CodeFragmentId, Span};
use zaplib_shader_compiler::ty::Ty;
use zaplib_shader_compiler::{Decl, ShaderAst};

/// Contains all information necessary to build a shader.
/// Define a new shader.
///
/// Pass in a [`Geometry`] which gets used for instancing (e.g. a quad or a
/// cube).
///
/// The different [`CodeFragment`]s are appended together (but preserving their
/// filename/line/column information for error messages). They are split out
/// into `base_code_fragments` and `main_code_fragment` purely for
/// convenience. (We could instead have used a single [`slice`] but they are
/// annoying to get through concatenation..)
///
/// TODO(JP): Would be good to instead compile shaders beforehand, ie. during
/// compile time. Should look into that at some point.
pub struct Shader {
    /// The [`Geometry`] that we will draw with, if any. Can be overridden using [`DrawCallProps::gpu_geometry`].
    pub build_geom: Option<fn() -> Geometry>,
    /// A bunch of [`CodeFragment`]s that will get concatenated.
    pub code_to_concatenate: &'static [CodeFragment],
    /// The id of the shader (index into [`Cx::shaders`]), or [`Shader::UNCOMPILED_SHADER_ID`] if uninitialized.
    /// You should never read or modify this manually (see TODO below).
    ///
    /// TODO(JP): This shouldn't be public, but right now that's necessary for using [`Shader::DEFAULT`]. We might want to
    /// switch to a `define_shader` helper function once we can pass function pointers to `const` functions (see
    /// <https://github.com/rust-lang/rust/issues/63997> and <https://github.com/rust-lang/rust/issues/57563>).
    pub shader_id: AtomicUsize,
}

impl Shader {
    /// TODO(JP): We might want to switch to a `define_shader` helper function once we can pass function pointers
    /// to `const` functions (see <https://github.com/rust-lang/rust/issues/63997> and
    /// <https://github.com/rust-lang/rust/issues/57563>).
    ///
    /// We suppress `clippy::declare_interior_mutable_const` here since we don't actually want shader_id in this constant
    /// to be editable.
    #[allow(clippy::declare_interior_mutable_const)]
    pub const DEFAULT: Shader =
        Shader { build_geom: None, code_to_concatenate: &[], shader_id: AtomicUsize::new(Self::UNCOMPILED_SHADER_ID) };

    const UNCOMPILED_SHADER_ID: usize = usize::MAX;

    pub fn update(&'static self, cx: &mut Cx, new_code_to_concatenate: &[CodeFragment]) -> Result<(), ParseError> {
        let shader_id = cx.get_shader_id(self);

        let shader = &mut cx.shaders[shader_id];
        let shader_ast = cx.shader_ast_generator.generate_shader_ast(new_code_to_concatenate)?;
        if shader.mapping != CxShaderMapping::from_shader_ast(shader_ast.clone()) {
            return Err(ParseError {
                span: Span { code_fragment_id: CodeFragmentId(0), start: 0, end: 0 },
                message: "Mismatch in shader mapping".to_string(),
            });
        }
        shader.shader_ast = Some(shader_ast);
        cx.shader_recompile_ids.push(shader_id);

        Ok(())
    }
}

/// Contains information of a [`CxShader`] of what instances, instances, textures
/// and so on it contains. That information can then be used to modify a [`Shader`
/// or [`DrawCall`].
#[derive(Debug, Default, Clone, PartialEq)]
pub(crate) struct CxShaderMapping {
    /// Contains information about the special "rect_pos" and "rect_size" fields.
    /// See [`RectInstanceProps`].
    pub(crate) rect_instance_props: RectInstanceProps,
    /// Special structure for user-level uniforms.
    pub(crate) user_uniform_props: UniformProps,
    /// Special structure for reading/editing instance properties.
    pub(crate) instance_props: InstanceProps,
    /// Special structure for reading/editing geometry properties.
    #[cfg(any(target_arch = "wasm32", target_os = "linux", target_os = "windows"))]
    pub(crate) geometry_props: InstanceProps,
    /// Raw definition of all textures.
    pub(crate) textures: Vec<PropDef>,
    /// Raw definition of all geometries.
    #[cfg(target_os = "windows")]
    pub(crate) geometries: Vec<PropDef>,
    /// Raw definition of all instances.
    #[cfg(target_os = "windows")]
    pub(crate) instances: Vec<PropDef>,
    /// Raw definition of all user-level uniforms.
    #[cfg(any(target_arch = "wasm32", target_os = "linux"))]
    pub(crate) user_uniforms: Vec<PropDef>,
    /// Raw definition of all framework-level uniforms that get set per [`DrawCall`].
    #[cfg(any(target_arch = "wasm32", target_os = "linux"))]
    pub(crate) draw_uniforms: Vec<PropDef>,
    /// Raw definition of all framework-level uniforms that get set per [`View`].
    #[cfg(any(target_arch = "wasm32", target_os = "linux"))]
    pub(crate) view_uniforms: Vec<PropDef>,
    /// Raw definition of all framework-level uniforms that get set per [`Pass`].
    #[cfg(any(target_arch = "wasm32", target_os = "linux"))]
    pub(crate) pass_uniforms: Vec<PropDef>,
}

impl CxShaderMapping {
    fn from_shader_ast(shader_ast: ShaderAst) -> Self {
        let mut instances = Vec::new();
        let mut geometries = Vec::new();
        let mut user_uniforms = Vec::new();
        let mut draw_uniforms = Vec::new();
        let mut view_uniforms = Vec::new();
        let mut pass_uniforms = Vec::new();
        let mut textures = Vec::new();
        for decl in shader_ast.decls {
            match decl {
                Decl::Geometry(decl) => {
                    let prop_def = PropDef { name: decl.ident.to_string(), ty: decl.ty_expr.ty.borrow().clone().unwrap() };
                    geometries.push(prop_def);
                }
                Decl::Instance(decl) => {
                    let prop_def = PropDef { name: decl.ident.to_string(), ty: decl.ty_expr.ty.borrow().clone().unwrap() };
                    instances.push(prop_def);
                }
                Decl::Uniform(decl) => {
                    let prop_def = PropDef { name: decl.ident.to_string(), ty: decl.ty_expr.ty.borrow().clone().unwrap() };
                    match decl.block_ident {
                        Some(bi) if bi.with(|string| string == "draw") => {
                            draw_uniforms.push(prop_def);
                        }
                        Some(bi) if bi.with(|string| string == "view") => {
                            view_uniforms.push(prop_def);
                        }
                        Some(bi) if bi.with(|string| string == "pass") => {
                            pass_uniforms.push(prop_def);
                        }
                        None => {
                            user_uniforms.push(prop_def);
                        }
                        _ => (),
                    }
                }
                Decl::Texture(decl) => {
                    let prop_def = PropDef { name: decl.ident.to_string(), ty: decl.ty_expr.ty.borrow().clone().unwrap() };
                    textures.push(prop_def);
                }
                _ => (),
            }
        }

        CxShaderMapping {
            rect_instance_props: RectInstanceProps::construct(&instances),
            user_uniform_props: UniformProps::construct(&user_uniforms),
            instance_props: InstanceProps::construct(&instances),
            #[cfg(any(target_arch = "wasm32", target_os = "linux", target_os = "windows"))]
            geometry_props: InstanceProps::construct(&geometries),
            textures,
            #[cfg(target_os = "windows")]
            instances,
            #[cfg(target_os = "windows")]
            geometries,
            #[cfg(any(target_arch = "wasm32", target_os = "linux"))]
            pass_uniforms,
            #[cfg(any(target_arch = "wasm32", target_os = "linux"))]
            view_uniforms,
            #[cfg(any(target_arch = "wasm32", target_os = "linux"))]
            draw_uniforms,
            #[cfg(any(target_arch = "wasm32", target_os = "linux"))]
            user_uniforms,
        }
    }
}

/// The raw definition of an input property to a [`Shader`].
#[derive(Debug, Clone, Hash, PartialEq)]
pub(crate) struct PropDef {
    pub(crate) name: String,
    pub(crate) ty: Ty,
}

/// Contains information about the special "rect_pos" and "rect_size" fields.
/// These fields describe the typical rectangles drawn in [`crate::QuadIns`]. It is
/// useful to have generalized access to them, so we can e.g. move a whole bunch
/// of rectangles at the same time, e.g. for alignment in [`CxLayoutBox`].
/// TODO(JP): We might want to consider instead doing bulk moves using [`DrawCall`-
/// or [`View`]-level uniforms.
#[derive(Debug, Default, Clone, PartialEq)]
pub(crate) struct RectInstanceProps {
    pub(crate) rect_pos: Option<usize>,
    pub(crate) rect_size: Option<usize>,
}
impl RectInstanceProps {
    fn construct(instances: &[PropDef]) -> RectInstanceProps {
        let mut rect_pos = None;
        let mut rect_size = None;
        let mut slot = 0;
        for inst in instances {
            match inst.name.as_ref() {
                "rect_pos" => rect_pos = Some(slot),
                "rect_size" => rect_size = Some(slot),
                _ => (),
            }
            slot += inst.ty.size(); //sg.get_type_slots(&inst.ty);
        }
        RectInstanceProps { rect_pos, rect_size }
    }
}

/// Represents an "instance" GPU input in a [`Shader`].
///
/// TODO(JP): Merge this into [`NamedProp`].
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct InstanceProp {
    pub(crate) name: String,
    pub(crate) slots: usize,
}

/// Represents all "instance" GPU inputs in a [`Shader`].
///
/// TODO(JP): Merge this into [`NamedProps`].
#[derive(Debug, Default, Clone, PartialEq)]
pub(crate) struct InstanceProps {
    pub(crate) props: Vec<InstanceProp>,
    pub(crate) total_slots: usize,
}

/// Represents all "uniform" GPU inputs in a [`Shader`].
///
/// TODO(JP): Merge this into [`NamedProps`].
#[derive(Debug, Default, Clone, PartialEq)]
pub(crate) struct UniformProps {
    pub(crate) total_slots: usize,
}

/// A generic representation of any kind of [`Shader`] input (instance/uniform/geometry).
#[cfg(target_os = "windows")]
#[derive(Debug, Clone)]
pub(crate) struct NamedProp {
    pub(crate) offset: usize,
    pub(crate) slots: usize,
}

/// A generic representation of a list of [`Shader`] inputs (instance/uniform/geometry).
#[cfg(target_os = "windows")]
#[derive(Debug, Default, Clone)]
pub(crate) struct NamedProps {
    pub(crate) props: Vec<NamedProp>,
}

#[cfg(target_os = "windows")]
impl NamedProps {
    pub(crate) fn construct(in_props: &[PropDef]) -> NamedProps {
        let mut offset = 0;
        let out_props = in_props
            .iter()
            .map(|prop| {
                let slots = prop.ty.size();
                let prop = NamedProp { offset, slots };
                offset += slots;
                prop
            })
            .collect();
        NamedProps { props: out_props }
    }
}

impl InstanceProps {
    fn construct(in_props: &[PropDef]) -> InstanceProps {
        let mut offset = 0;
        let out_props = in_props
            .iter()
            .map(|prop| {
                let slots = prop.ty.size();
                let prop = InstanceProp { name: prop.name.clone(), slots };
                offset += slots;
                prop
            })
            .collect();
        InstanceProps { props: out_props, total_slots: offset }
    }
}

impl UniformProps {
    pub fn construct(in_props: &[PropDef]) -> UniformProps {
        UniformProps { total_slots: in_props.iter().map(|prop| prop.ty.size()).sum() }
    }
}

/// The actual shader information, which gets stored on [`Cx`]. Once compiled the
/// [`ShaderAst`] will be removed, and the [`CxPlatformShader`] (platform-specific
/// part of the compiled shader) gets set.
#[derive(Default)]
pub(crate) struct CxShader {
    pub(crate) name: String,
    pub(crate) gpu_geometry: Option<GpuGeometry>,
    pub(crate) platform: Option<CxPlatformShader>,
    pub(crate) mapping: CxShaderMapping,
    pub(crate) shader_ast: Option<ShaderAst>,
}

impl Cx {
    /// Get an individual [`Shader`] from a static [`Shader`].
    ///
    /// For more information on what [`LocationHash`] is used for here, see [`Shader`].
    pub(crate) fn get_shader_id(&mut self, shader: &'static Shader) -> usize {
        let shader_id = shader.shader_id.load(Ordering::Relaxed);
        if shader_id != Shader::UNCOMPILED_SHADER_ID {
            shader_id
        } else {
            // Use the last code fragment as the shader name.
            let main_code_fragment = shader.code_to_concatenate.last().expect("No code fragments found");
            match self.shader_ast_generator.generate_shader_ast(shader.code_to_concatenate) {
                Err(err) => panic!("{}", err.format_for_console(shader.code_to_concatenate)),
                Ok(shader_ast) => {
                    let gpu_geometry = shader.build_geom.map(|build_geom| GpuGeometry::new(self, (build_geom)()));

                    let shader_id = self.shaders.len();
                    self.shaders.push(CxShader {
                        name: main_code_fragment.name_line_col_at_offset(0),
                        gpu_geometry,
                        mapping: CxShaderMapping::from_shader_ast(shader_ast.clone()),
                        platform: None,
                        shader_ast: Some(shader_ast),
                    });
                    self.shader_recompile_ids.push(shader_id);

                    shader.shader_id.store(shader_id, Ordering::Relaxed);

                    shader_id
                }
            }
        }
    }
}
