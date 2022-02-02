//! Mac OS X Metal bindings.

use std::ffi::c_void;
use std::mem;
use std::os::raw::c_int;
use std::os::raw::c_ulong;
use std::ptr;
use std::ptr::NonNull;
use std::sync::Arc;
use std::sync::Condvar;
use std::sync::Mutex;

use crate::cx_apple::*;
use crate::cx_cocoa::*;
use crate::*;
use zaplib_objc_sys::msg_send;
use zaplib_objc_sys::runtime::YES;
use zaplib_shader_compiler::generate_metal;

impl Cx {
    fn render_view(
        &mut self,
        pass_id: usize,
        view_id: usize,
        scroll: Vec2,
        clip: (Vec2, Vec2),
        zbias: &mut f32,
        zbias_step: f32,
        encoder: id,
        gpu_read_guards: &mut Vec<MetalRwLockGpuReadGuard>,
        metal_cx: &MetalCx,
    ) {
        // tad ugly otherwise the borrow checker locks 'self' and we can't recur
        let draw_calls_len = self.views[view_id].draw_calls_len;
        //self.views[view_id].set_clipping_uniforms();
        self.views[view_id].parent_scroll = scroll;
        let local_scroll = self.views[view_id].snapped_scroll;
        let clip = self.views[view_id].intersect_clip(clip);

        for draw_call_id in 0..draw_calls_len {
            let sub_view_id = self.views[view_id].draw_calls[draw_call_id].sub_view_id;
            if sub_view_id != 0 {
                self.render_view(
                    pass_id,
                    sub_view_id,
                    Vec2 { x: local_scroll.x + scroll.x, y: local_scroll.y + scroll.y },
                    clip,
                    zbias,
                    zbias_step,
                    encoder,
                    gpu_read_guards,
                    metal_cx,
                );
            } else {
                let gpu_geometry_id = GpuGeometry::get_id(self, view_id, draw_call_id);

                let cxview = &mut self.views[view_id];
                //view.platform.uni_vw.update_with_f32_data(device, &view.uniforms);
                let draw_call = &mut cxview.draw_calls[draw_call_id];
                let sh = &self.shaders[draw_call.shader_id];
                let shp = sh.platform.as_ref().unwrap();

                if draw_call.instance_dirty {
                    draw_call.instance_dirty = false;
                    // update the instance buffer data
                    self.platform.bytes_written += draw_call.instances.len() * 4;
                    draw_call.platform.instance_buffer.cpu_write().update(metal_cx, &draw_call.instances);
                }

                // update the zbias uniform if we have it.
                draw_call.set_zbias(*zbias);
                draw_call.set_local_scroll(scroll, local_scroll);
                draw_call.set_clip(clip);
                *zbias += zbias_step;

                if draw_call.uniforms_dirty {
                    draw_call.uniforms_dirty = false;
                }

                // lets verify our instance_offset is not disaligned
                let instances = (draw_call.instances.len() / sh.mapping.instance_props.total_slots) as u64;
                if instances == 0 {
                    continue;
                }
                let render_pipeline_state = shp.render_pipeline_state.as_id();
                unsafe {
                    let () = msg_send![encoder, setRenderPipelineState: render_pipeline_state];
                }

                let geometry = &mut self.gpu_geometries[gpu_geometry_id];

                if geometry.dirty {
                    geometry.platform.vertex_buffer.cpu_write().update(metal_cx, geometry.geometry.vertices_f32_slice());
                    geometry.platform.index_buffer.cpu_write().update(metal_cx, geometry.geometry.indices_u32_slice());
                    geometry.dirty = false;
                }

                if let Some(inner) = geometry.platform.vertex_buffer.cpu_read().inner.as_ref() {
                    unsafe {
                        msg_send![
                            encoder,
                            setVertexBuffer: inner.buffer.as_id()
                            offset: 0
                            atIndex: 0
                        ]
                    }
                } else {
                    println!("Drawing error: vertex_buffer None")
                }

                if let Some(inner) = draw_call.platform.instance_buffer.cpu_read().inner.as_ref() {
                    unsafe {
                        msg_send![
                            encoder,
                            setVertexBuffer: inner.buffer.as_id()
                            offset: 0
                            atIndex: 1
                        ]
                    }
                } else {
                    println!("Drawing error: instance_buffer None")
                }

                let pass_uniforms = self.passes[pass_id].pass_uniforms.as_slice();
                let view_uniforms = cxview.view_uniforms.as_slice();
                let draw_uniforms = draw_call.draw_uniforms.as_slice();

                unsafe {
                    let () = msg_send![encoder, setVertexBytes:
                        pass_uniforms.as_ptr() as *const std::ffi::c_void length: (pass_uniforms.len() * 4) as u64 atIndex: 2u64];
                    let () = msg_send![encoder, setVertexBytes:
                        view_uniforms.as_ptr() as *const std::ffi::c_void length: (view_uniforms.len() * 4) as u64 atIndex: 3u64];
                    let () = msg_send![encoder, setVertexBytes:
                        draw_uniforms.as_ptr() as *const std::ffi::c_void length: (draw_uniforms.len() * 4) as u64 atIndex: 4u64];
                    let () = msg_send![encoder, setVertexBytes:
                        draw_call.user_uniforms.as_ptr() as *const std::ffi::c_void
                        length: (draw_call.user_uniforms.len() * 4) as u64 atIndex: 5u64];

                    let () = msg_send![encoder, setFragmentBytes:
                        pass_uniforms.as_ptr() as *const std::ffi::c_void length: (pass_uniforms.len() * 4) as u64 atIndex: 0u64];
                    let () = msg_send![encoder, setFragmentBytes:
                        view_uniforms.as_ptr() as *const std::ffi::c_void length: (view_uniforms.len() * 4) as u64 atIndex: 1u64];
                    let () = msg_send![encoder, setFragmentBytes:
                        draw_uniforms.as_ptr() as *const std::ffi::c_void length: (draw_uniforms.len() * 4) as u64 atIndex: 2u64];
                    let () = msg_send![encoder, setFragmentBytes:
                        draw_call.user_uniforms.as_ptr() as *const std::ffi::c_void
                        length: (draw_call.user_uniforms.len() * 4) as u64 atIndex: 3u64];
                }
                for (i, texture_id) in draw_call.textures_2d.iter().enumerate() {
                    let cxtexture = &mut self.textures[*texture_id as usize];
                    if cxtexture.update_image {
                        metal_cx.update_platform_texture_image2d(cxtexture);
                    }
                    if let Some(inner) = cxtexture.platform.inner.as_ref() {
                        let () = unsafe {
                            msg_send![
                                encoder,
                                setFragmentTexture: inner.texture.as_id()
                                atIndex: i as u64
                            ]
                        };
                        let () = unsafe {
                            msg_send![
                                encoder,
                                setVertexTexture: inner.texture.as_id()
                                atIndex: i as u64
                            ]
                        };
                    }
                }
                self.platform.draw_calls_done += 1;
                if let Some(inner) = geometry.platform.index_buffer.cpu_read().inner.as_ref() {
                    let () = unsafe {
                        msg_send![
                            encoder,
                            drawIndexedPrimitives: MTLPrimitiveType::Triangle
                            indexCount: geometry.geometry.indices_u32_slice().len() as u64
                            indexType: MTLIndexType::UInt32
                            indexBuffer: inner.buffer.as_id()
                            indexBufferOffset: 0
                            instanceCount: instances
                        ]
                    };
                } else {
                    println!("Drawing error: index_buffer None")
                }

                gpu_read_guards.push(draw_call.platform.instance_buffer.gpu_read());
                gpu_read_guards.push(geometry.platform.vertex_buffer.gpu_read());
                gpu_read_guards.push(geometry.platform.index_buffer.gpu_read());
            }
        }
        self.debug_draw_tree(view_id);
    }

    pub(crate) fn setup_render_pass_descriptor(
        &mut self,
        render_pass_descriptor: id,
        pass_id: usize,
        inherit_dpi_factor: f32,
        first_texture: Option<id>,
        metal_cx: &MetalCx,
    ) {
        let pass_size = self.passes[pass_id].pass_size;

        self.passes[pass_id].set_matrix(Vec2::default(), pass_size);
        self.passes[pass_id].paint_dirty = false;
        let dpi_factor = if let Some(override_dpi_factor) = self.passes[pass_id].override_dpi_factor {
            override_dpi_factor
        } else {
            inherit_dpi_factor
        };
        self.passes[pass_id].set_dpi_factor(dpi_factor);

        for (index, color_texture) in self.passes[pass_id].color_textures.iter().enumerate() {
            let color_attachments: id = unsafe { msg_send![render_pass_descriptor, colorAttachments] };
            let color_attachment: id = unsafe { msg_send![color_attachments, objectAtIndexedSubscript: 0] };
            // let color_attachment = render_pass_descriptor.color_attachments().object_at(0).unwrap();

            let is_initial;
            if index == 0 && first_texture.is_some() {
                let () = unsafe {
                    msg_send![
                        color_attachment,
                        setTexture: first_texture.unwrap()
                    ]
                };
                is_initial = true;
            } else {
                let cxtexture = &mut self.textures[color_texture.texture_id as usize];
                cxtexture.platform.update(metal_cx, AttachmentKind::Color, &cxtexture.desc, dpi_factor * pass_size);
                is_initial = !cxtexture.platform.inner.as_ref().unwrap().is_inited;

                if let Some(inner) = cxtexture.platform.inner.as_ref() {
                    let () = unsafe {
                        msg_send![
                            color_attachment,
                            setTexture: inner.texture.as_id()
                        ]
                    };
                } else {
                    println!("draw_pass_to_texture invalid render target");
                }
            }
            unsafe { msg_send![color_attachment, setStoreAction: MTLStoreAction::Store] }

            match color_texture.clear_color {
                ClearColor::InitWith(color) => {
                    if is_initial {
                        unsafe {
                            let () = msg_send![color_attachment, setLoadAction: MTLLoadAction::Clear];
                            let () = msg_send![color_attachment, setClearColor: MTLClearColor {
                                red: color.x as f64,
                                green: color.y as f64,
                                blue: color.z as f64,
                                alpha: color.w as f64
                            }];
                        }
                    } else {
                        unsafe {
                            let () = msg_send![color_attachment, setLoadAction: MTLLoadAction::Load];
                        }
                    }
                }
                ClearColor::ClearWith(color) => unsafe {
                    let () = msg_send![color_attachment, setLoadAction: MTLLoadAction::Clear];
                    let () = msg_send![color_attachment, setClearColor: MTLClearColor {
                        red: color.x as f64,
                        green: color.y as f64,
                        blue: color.z as f64,
                        alpha: color.w as f64
                    }];
                },
            }
        }
        // attach depth texture
        if let Some(depth_texture_id) = self.passes[pass_id].depth_texture {
            let cxtexture = &mut self.textures[depth_texture_id as usize];
            cxtexture.platform.update(metal_cx, AttachmentKind::Depth, &cxtexture.desc, dpi_factor * pass_size);
            let is_initial = !cxtexture.platform.inner.as_ref().unwrap().is_inited;

            let depth_attachment: id = unsafe { msg_send![render_pass_descriptor, depthAttachment] };

            if let Some(inner) = cxtexture.platform.inner.as_ref() {
                unsafe { msg_send![depth_attachment, setTexture: inner.texture.as_id()] }
            } else {
                println!("draw_pass_to_texture invalid render target");
            }
            let () = unsafe { msg_send![depth_attachment, setStoreAction: MTLStoreAction::Store] };

            match self.passes[pass_id].clear_depth {
                ClearDepth::InitWith(depth) => {
                    if is_initial {
                        let () = unsafe { msg_send![depth_attachment, setLoadAction: MTLLoadAction::Clear] };
                        let () = unsafe { msg_send![depth_attachment, setClearDepth: depth as f64] };
                    } else {
                        let () = unsafe { msg_send![depth_attachment, setLoadAction: MTLLoadAction::Load] };
                    }
                }
                ClearDepth::ClearWith(depth) => {
                    let () = unsafe { msg_send![depth_attachment, setLoadAction: MTLLoadAction::Clear] };
                    let () = unsafe { msg_send![depth_attachment, setClearDepth: depth as f64] };
                }
            }
            // create depth state
            if self.passes[pass_id].platform.mtl_depth_state.is_none() {
                let desc: id = unsafe { msg_send![class!(MTLDepthStencilDescriptor), new] };
                let () = unsafe { msg_send![desc, setDepthCompareFunction: MTLCompareFunction::LessEqual] };
                let () = unsafe { msg_send![desc, setDepthWriteEnabled: true] };
                let depth_stencil_state: id = unsafe { msg_send![metal_cx.device, newDepthStencilStateWithDescriptor: desc] };
                self.passes[pass_id].platform.mtl_depth_state = Some(depth_stencil_state);
            }
        }
    }

    /**
    TODO(JP): This is pretty inefficient -- we create a new buffer every time (which has overhead),
    and then don't wait for it to finish, which might cause the command queue to fill up with tons of buffers,
    which has a pretty bad effect on the user experience (everything becomes pretty laggy).

    We should really implement a triple-buffering scheme as mentioned here:
    <https://developer.apple.com/videos/play/wwdc2015/610/?time=1762>
    <https://developer.apple.com/library/archive/documentation/3DDrawing/>
    <Conceptual/MTLBestPracticesGuide/TripleBuffering.html>.

    We can do the draw/paint in a callback when the first buffer is done, instead of as part of our event loop.
    Or we can block but only if the CPU is too far ahead: <https://developer.apple.com/forums/thread/651581>
    */
    pub(crate) fn draw_pass_to_layer(
        &mut self,
        pass_id: usize,
        dpi_factor: f32,
        layer: id,
        metal_cx: &mut MetalCx,
        is_resizing: bool,
    ) {
        self.platform.bytes_written = 0;
        self.platform.draw_calls_done = 0;
        let view_id = self.passes[pass_id].main_view_id.unwrap();

        let pool: id = unsafe { msg_send![class!(NSAutoreleasePool), new] };

        //let command_buffer = command_queue.new_command_buffer();
        let drawable: id = unsafe { msg_send![layer, nextDrawable] };
        if drawable != nil {
            let render_pass_descriptor: id = unsafe { msg_send![class!(MTLRenderPassDescriptorInternal), renderPassDescriptor] };

            let texture: id = unsafe { msg_send![drawable, texture] };

            self.setup_render_pass_descriptor(render_pass_descriptor, pass_id, dpi_factor, Some(texture), metal_cx);

            let command_buffer: id = unsafe { msg_send![metal_cx.command_queue, commandBuffer] };
            let encoder: id = unsafe { msg_send![command_buffer, renderCommandEncoderWithDescriptor: render_pass_descriptor] };

            unsafe { msg_send![encoder, textureBarrier] }

            if let Some(depth_state) = self.passes[pass_id].platform.mtl_depth_state {
                let () = unsafe { msg_send![encoder, setDepthStencilState: depth_state] };
            }
            let mut zbias = 0.0;
            let zbias_step = self.passes[pass_id].zbias_step;

            let mut gpu_read_guards = Vec::new();
            self.render_view(
                pass_id,
                view_id,
                Vec2::default(),
                (Vec2 { x: -50000., y: -50000. }, Vec2 { x: 50000., y: 50000. }),
                &mut zbias,
                zbias_step,
                encoder,
                &mut gpu_read_guards,
                metal_cx,
            );

            let () = unsafe { msg_send![encoder, endEncoding] };
            if is_resizing {
                self.commit_command_buffer(command_buffer, gpu_read_guards);
                let () = unsafe { msg_send![command_buffer, waitUntilScheduled] };
                let () = unsafe { msg_send![drawable, present] };
            } else {
                let () = unsafe { msg_send![command_buffer, presentDrawable: drawable] };
                self.commit_command_buffer(command_buffer, gpu_read_guards);
            }
        }
        let () = unsafe { msg_send![pool, release] };
    }

    pub(crate) fn draw_pass_to_texture(&mut self, pass_id: usize, dpi_factor: f32, metal_cx: &MetalCx) {
        let view_id = self.passes[pass_id].main_view_id.unwrap();

        let pool: id = unsafe { msg_send![class!(NSAutoreleasePool), new] };
        let render_pass_descriptor: id = unsafe { msg_send![class!(MTLRenderPassDescriptorInternal), renderPassDescriptor] };

        self.setup_render_pass_descriptor(render_pass_descriptor, pass_id, dpi_factor, None, metal_cx);

        let command_buffer: id = unsafe { msg_send![metal_cx.command_queue, commandBuffer] };
        let encoder: id = unsafe { msg_send![command_buffer, renderCommandEncoderWithDescriptor: render_pass_descriptor] };

        if let Some(depth_state) = self.passes[pass_id].platform.mtl_depth_state {
            let () = unsafe { msg_send![encoder, setDepthStencilState: depth_state] };
        }

        let mut zbias = 0.0;
        let zbias_step = self.passes[pass_id].zbias_step;
        let mut gpu_read_guards = Vec::new();
        self.render_view(
            pass_id,
            view_id,
            Vec2::default(),
            (Vec2 { x: -50000., y: -50000. }, Vec2 { x: 50000., y: 50000. }),
            &mut zbias,
            zbias_step,
            encoder,
            &mut gpu_read_guards,
            metal_cx,
        );
        let () = unsafe { msg_send![encoder, textureBarrier] };
        let () = unsafe { msg_send![encoder, endEncoding] };
        self.commit_command_buffer(command_buffer, gpu_read_guards);
        let () = unsafe { msg_send![pool, release] };
    }

    fn commit_command_buffer(&mut self, command_buffer: id, gpu_read_guards: Vec<MetalRwLockGpuReadGuard>) {
        #[repr(C)]
        struct BlockDescriptor {
            reserved: c_ulong,
            size: c_ulong,
            copy_helper: extern "C" fn(*mut c_void, *const c_void),
            dispose_helper: extern "C" fn(*mut c_void),
        }

        static DESCRIPTOR: BlockDescriptor =
            BlockDescriptor { reserved: 0, size: mem::size_of::<BlockLiteral>() as c_ulong, copy_helper, dispose_helper };

        extern "C" fn copy_helper(dst: *mut c_void, src: *const c_void) {
            unsafe {
                ptr::write(&mut (*(dst as *mut BlockLiteral)).inner as *mut _, (*(src as *const BlockLiteral)).inner.clone());
            }
        }

        extern "C" fn dispose_helper(src: *mut c_void) {
            unsafe {
                ptr::drop_in_place(src as *mut BlockLiteral);
            }
        }

        #[repr(C)]
        struct BlockLiteral {
            isa: *const c_void,
            flags: c_int,
            reserved: c_int,
            invoke: extern "C" fn(*mut BlockLiteral, id),
            descriptor: *const BlockDescriptor,
            inner: Arc<BlockLiteralInner>,
        }

        #[repr(C)]
        struct BlockLiteralInner {
            gpu_read_guards: Mutex<Option<Vec<MetalRwLockGpuReadGuard>>>,
        }

        let literal = BlockLiteral {
            isa: unsafe { _NSConcreteStackBlock.as_ptr() as *const c_void },
            flags: 1 << 25,
            reserved: 0,
            invoke,
            descriptor: &DESCRIPTOR,
            inner: Arc::new(BlockLiteralInner { gpu_read_guards: Mutex::new(Some(gpu_read_guards)) }),
        };

        extern "C" fn invoke(literal: *mut BlockLiteral, _command_buffer: id) {
            let literal = unsafe { &mut *literal };
            drop(literal.inner.gpu_read_guards.lock().unwrap().take().unwrap());
        }

        let () = unsafe { msg_send![command_buffer, addCompletedHandler: &literal] };
        let () = unsafe { msg_send![command_buffer, commit] };
    }
}

pub(crate) struct MetalCx {
    pub(crate) device: id,
    pub(crate) command_queue: id,
}

#[derive(Clone)]
pub(crate) struct MetalWindow {
    pub(crate) window_id: usize,
    pub(crate) first_draw: bool,
    pub(crate) window_geom: WindowGeom,
    pub(crate) cal_size: Vec2,
    pub(crate) ca_layer: id,
    pub(crate) cocoa_window: CocoaWindow,
    pub(crate) is_resizing: bool,
}

impl MetalWindow {
    pub(crate) fn new(
        window_id: usize,
        metal_cx: &MetalCx,
        cocoa_app: &mut CocoaApp,
        inner_size: Vec2,
        position: Option<Vec2>,
        title: &str,
        add_drop_target_for_app_open_files: bool,
    ) -> MetalWindow {
        let ca_layer: id = unsafe { msg_send![class!(CAMetalLayer), new] };

        let mut cocoa_window = CocoaWindow::new(cocoa_app, window_id);

        cocoa_window.init(title, inner_size, position, add_drop_target_for_app_open_files);

        unsafe {
            let () = msg_send![ca_layer, setDevice: metal_cx.device];
            let () = msg_send![ca_layer, setPixelFormat: MTLPixelFormat::BGRA8Unorm];
            let () = msg_send![ca_layer, setPresentsWithTransaction: NO];
            let () = msg_send![ca_layer, setMaximumDrawableCount: 3];
            let () = msg_send![ca_layer, setDisplaySyncEnabled: NO];
            let () = msg_send![ca_layer, setNeedsDisplayOnBoundsChange: YES];
            let () = msg_send![ca_layer, setAutoresizingMask: (1 << 4) | (1 << 1)];
            let () = msg_send![ca_layer, setAllowsNextDrawableTimeout: NO];
            let () = msg_send![ca_layer, setDelegate: cocoa_window.view];
            let () = msg_send![ca_layer, setOpaque: NO];
            let () = msg_send![ca_layer, setBackgroundColor: CGColorCreateSRGB(0.0, 0.0, 0.0, 0.0)];

            let view = cocoa_window.view;
            let () = msg_send![view, setWantsBestResolutionOpenGLSurface: YES];
            let () = msg_send![view, setWantsLayer: YES];
            let () = msg_send![view, setLayerContentsPlacement: 11];
            let () = msg_send![view, setLayer: ca_layer];
        }

        MetalWindow {
            is_resizing: false,
            first_draw: true,
            window_id,
            cal_size: Vec2::default(),
            ca_layer,
            window_geom: cocoa_window.get_window_geom(),
            cocoa_window,
        }
    }

    pub(crate) fn start_resize(&mut self) {
        self.is_resizing = true;
        let () = unsafe { msg_send![self.ca_layer, setPresentsWithTransaction: YES] };
    }

    pub(crate) fn stop_resize(&mut self) {
        self.is_resizing = false;
        let () = unsafe { msg_send![self.ca_layer, setPresentsWithTransaction: NO] };
    }

    pub(crate) fn resize_core_animation_layer(&mut self, _metal_cx: &MetalCx) -> bool {
        let cal_size = Vec2 {
            x: self.window_geom.inner_size.x * self.window_geom.dpi_factor,
            y: self.window_geom.inner_size.y * self.window_geom.dpi_factor,
        };
        if self.cal_size != cal_size {
            self.cal_size = cal_size;
            unsafe {
                let () = msg_send![self.ca_layer, setDrawableSize: CGSize {width: cal_size.x as f64, height: cal_size.y as f64}];
                let () = msg_send![self.ca_layer, setContentsScale: self.window_geom.dpi_factor as f64];
            }
            true
        } else {
            false
        }
    }
}

#[derive(Clone, Default)]
pub(crate) struct CxPlatformView {}

#[derive(Default, Clone)]
pub(crate) struct CxPlatformPass {
    pub(crate) mtl_depth_state: Option<id>,
}

impl Cx {
    pub(crate) fn mtl_compile_shaders(&mut self, metal_cx: &MetalCx) {
        for shader_id in self.shader_recompile_ids.drain(..) {
            let shader = unsafe { self.shaders.get_unchecked_mut(shader_id) };
            let shader_ast = shader.shader_ast.as_ref().unwrap();
            let mtlsl = generate_metal::generate_shader(shader_ast);
            shader.platform = Some(CxPlatformShader::new(metal_cx, mtlsl));
            shader.shader_ast = None;
        }
    }
}

impl MetalCx {
    pub(crate) fn new() -> MetalCx {
        /*
        let devices = get_all_metal_devices();
        for device in devices {
            let is_low_power: BOOL = unsafe {msg_send![device, isLowPower]};
            let command_queue: id = unsafe {msg_send![device, newCommandQueue]};
            if is_low_power == YES {
                return MetalCx {
                    command_queue: command_queue,
                    device: device
                }
            }
        }
        */
        let device = get_default_metal_device().expect("Cannot get default metal device");
        MetalCx { command_queue: unsafe { msg_send![device, newCommandQueue] }, device }
    }

    pub(crate) fn update_platform_texture_image2d(&self, cxtexture: &mut CxTexture) {
        if cxtexture.desc.width.is_none() || cxtexture.desc.height.is_none() {
            println!("update_platform_texture_image2d without width/height");
            return;
        }

        let width = cxtexture.desc.width.unwrap() as u64;
        let height = cxtexture.desc.height.unwrap() as u64;

        let mut desc_changed = true;
        if let Some(inner) = &cxtexture.platform.inner {
            desc_changed = inner.format != cxtexture.desc.format
                || inner.width != width
                || inner.height != height
                || inner.multisample != cxtexture.desc.multisample;
        }

        // allocate new texture if descriptor change
        if desc_changed {
            let descriptor = RcObjcId::from_owned(NonNull::new(unsafe { msg_send![class!(MTLTextureDescriptor), new] }).unwrap());
            let texture = RcObjcId::from_owned(
                NonNull::new(unsafe {
                    let _: () = msg_send![descriptor.as_id(), setTextureType: MTLTextureType::D2];
                    let _: () = msg_send![descriptor.as_id(), setWidth: width as u64];
                    let _: () = msg_send![descriptor.as_id(), setHeight: height as u64];
                    let _: () = msg_send![descriptor.as_id(), setStorageMode: MTLStorageMode::Managed];
                    let _: () = msg_send![descriptor.as_id(), setUsage: MTLTextureUsage::RenderTarget];
                    match cxtexture.desc.format {
                        TextureFormat::ImageRGBA => {
                            let _: () = msg_send![descriptor.as_id(), setPixelFormat: MTLPixelFormat::RGBA8Unorm];
                        }
                        _ => {
                            panic!("update_platform_texture_image2d with unsupported format");
                        }
                    }
                    msg_send![self.device, newTextureWithDescriptor: descriptor]
                })
                .unwrap(),
            );

            cxtexture.platform.inner = Some(CxPlatformTextureInner {
                is_inited: false,
                width,
                height,
                format: cxtexture.desc.format,
                multisample: cxtexture.desc.multisample,
                texture,
            });
        }

        // always allocate new image
        let inner = cxtexture.platform.inner.as_ref().unwrap();
        match cxtexture.desc.format {
            TextureFormat::ImageRGBA => {
                if cxtexture.image_u32.len() as u64 != width * height {
                    panic!("update_platform_texture_image2d with wrong buffer_u32 size!");
                }
                let region = MTLRegion {
                    origin: MTLOrigin { x: 0, y: 0, z: 0 },
                    size: MTLSize { width: width as u64, height: height as u64, depth: 1 },
                };
                let mtl_texture = inner.texture.as_id();
                let () = unsafe {
                    msg_send![
                        mtl_texture,
                        replaceRegion: region
                        mipmapLevel: 0
                        withBytes: cxtexture.image_u32.as_ptr() as *const std::ffi::c_void
                        bytesPerRow: (width as usize * std::mem::size_of::<u32>()) as u64
                    ]
                };
            }
            _ => {
                println!("update_platform_texture_image2d with unsupported format");
                return;
            }
        }
        cxtexture.update_image = false;
    }
}

pub(crate) struct CxPlatformShader {
    render_pipeline_state: RcObjcId,
}

impl CxPlatformShader {
    pub(crate) fn new(metal_cx: &MetalCx, mtlsl: String) -> Self {
        let options = RcObjcId::from_owned(unsafe { msg_send![class!(MTLCompileOptions), new] });
        unsafe {
            let _: () = msg_send![options.as_id(), setFastMathEnabled: YES];
        };

        let mut error: id = nil;

        let library = RcObjcId::from_owned(
            match NonNull::new(unsafe {
                msg_send![
                    metal_cx.device,
                    newLibraryWithSource: str_to_nsstring(&mtlsl)
                    options: options
                    error: &mut error
                ]
            }) {
                Some(library) => library,
                None => {
                    let description: id = unsafe { msg_send![error, localizedDescription] };
                    panic!("{}", nsstring_to_string(description));
                }
            },
        );

        let descriptor =
            RcObjcId::from_owned(NonNull::new(unsafe { msg_send![class!(MTLRenderPipelineDescriptor), new] }).unwrap());

        let vertex_function = RcObjcId::from_owned(
            NonNull::new(unsafe { msg_send![library.as_id(), newFunctionWithName: str_to_nsstring("mpsc_vertex_main")] })
                .unwrap(),
        );

        let fragment_function = RcObjcId::from_owned(
            NonNull::new(unsafe { msg_send![library.as_id(), newFunctionWithName: str_to_nsstring("mpsc_fragment_main")] })
                .unwrap(),
        );

        let render_pipeline_state = RcObjcId::from_owned(
            NonNull::new(unsafe {
                let _: () = msg_send![descriptor.as_id(), setVertexFunction: vertex_function];
                let _: () = msg_send![descriptor.as_id(), setFragmentFunction: fragment_function];

                let color_attachments: id = msg_send![descriptor.as_id(), colorAttachments];
                let color_attachment: id = msg_send![color_attachments, objectAtIndexedSubscript: 0];
                let () = msg_send![color_attachment, setPixelFormat: MTLPixelFormat::BGRA8Unorm];
                let () = msg_send![color_attachment, setBlendingEnabled: YES];
                let () = msg_send![color_attachment, setRgbBlendOperation: MTLBlendOperation::Add];
                let () = msg_send![color_attachment, setAlphaBlendOperation: MTLBlendOperation::Add];
                let () = msg_send![color_attachment, setSourceRGBBlendFactor: MTLBlendFactor::One];
                let () = msg_send![color_attachment, setSourceAlphaBlendFactor: MTLBlendFactor::One];
                let () = msg_send![color_attachment, setDestinationRGBBlendFactor: MTLBlendFactor::OneMinusSourceAlpha];
                let () = msg_send![color_attachment, setDestinationAlphaBlendFactor: MTLBlendFactor::OneMinusSourceAlpha];

                let () = msg_send![descriptor.as_id(), setDepthAttachmentPixelFormat: MTLPixelFormat::Depth32Float_Stencil8];

                let mut error: id = nil;
                msg_send![
                    metal_cx.device,
                    newRenderPipelineStateWithDescriptor: descriptor
                    error: &mut error
                ]
            })
            .unwrap(),
        );

        Self { render_pipeline_state }
    }
}

#[derive(Default)]
pub(crate) struct CxPlatformDrawCall {
    //pub(crate) uni_dr: MetalBuffer,
    instance_buffer: MetalRwLock<MetalBuffer>,
}

#[derive(Default)]
pub(crate) struct CxPlatformGpuGeometry {
    vertex_buffer: MetalRwLock<MetalBuffer>,
    index_buffer: MetalRwLock<MetalBuffer>,
}

#[derive(Default)]
struct MetalBuffer {
    inner: Option<MetalBufferInner>,
}

impl MetalBuffer {
    fn update<T>(&mut self, metal_cx: &MetalCx, data: &[T]) {
        let len = data.len() * std::mem::size_of::<T>();
        if len == 0 {
            self.inner = None;
            return;
        }
        if self.inner.as_ref().map_or(0, |inner| inner.len) < len {
            self.inner = Some(MetalBufferInner {
                len,
                buffer: RcObjcId::from_owned(
                    NonNull::new(unsafe {
                        msg_send![
                            metal_cx.device,
                            newBufferWithLength: len as u64
                            options: nil
                        ]
                    })
                    .unwrap(),
                ),
            });
        }
        let inner = self.inner.as_ref().unwrap();
        unsafe {
            let contents: *mut u8 = msg_send![inner.buffer.as_id(), contents];
            std::ptr::copy(data.as_ptr() as *const u8, contents, len);
            let _: () = msg_send![
                inner.buffer.as_id(),
                didModifyRange: NSRange {
                    location: 0,
                    length: len as u64
                }
            ];
        }
    }
}

struct MetalBufferInner {
    len: usize,
    buffer: RcObjcId,
}

#[derive(Default)]
pub(crate) struct CxPlatformTexture {
    inner: Option<CxPlatformTextureInner>,
}

impl CxPlatformTexture {
    fn update(&mut self, metal_cx: &MetalCx, attachment_kind: AttachmentKind, desc: &TextureDesc, default_size: Vec2) {
        let width = desc.width.unwrap_or(default_size.x as usize) as u64;
        let height = desc.height.unwrap_or(default_size.y as usize) as u64;

        let inited = self.inner.as_mut().map_or(false, |inner| {
            if inner.width != width {
                return false;
            }
            if inner.height != height {
                return false;
            }
            if inner.format != desc.format {
                return false;
            }
            if inner.multisample != desc.multisample {
                return false;
            }
            inner.is_inited = true;
            true
        });
        if inited {
            return;
        }

        let descriptor = RcObjcId::from_owned(NonNull::new(unsafe { msg_send![class!(MTLTextureDescriptor), new] }).unwrap());
        let texture = RcObjcId::from_owned(
            NonNull::new(unsafe {
                let _: () = msg_send![descriptor.as_id(), setTextureType: MTLTextureType::D2];
                let _: () = msg_send![descriptor.as_id(), setWidth: width as u64];
                let _: () = msg_send![descriptor.as_id(), setHeight: height as u64];
                let _: () = msg_send![descriptor.as_id(), setDepth: 1u64];
                let _: () = msg_send![descriptor.as_id(), setStorageMode: MTLStorageMode::Private];
                let _: () = msg_send![descriptor.as_id(), setUsage: MTLTextureUsage::RenderTarget];
                match attachment_kind {
                    AttachmentKind::Color => match desc.format {
                        TextureFormat::ImageRGBA => {
                            let _: () = msg_send![descriptor.as_id(), setPixelFormat: MTLPixelFormat::RGBA8Unorm];
                        }
                        _ => panic!(),
                    },
                    AttachmentKind::Depth => match desc.format {
                        TextureFormat::Depth32Stencil8 => {
                            let _: () = msg_send![descriptor.as_id(), setPixelFormat: MTLPixelFormat::Depth32Float_Stencil8];
                        }
                        _ => panic!(),
                    },
                }
                msg_send![metal_cx.device, newTextureWithDescriptor: descriptor]
            })
            .unwrap(),
        );

        self.inner = Some(CxPlatformTextureInner {
            is_inited: false,
            width,
            height,
            format: desc.format,
            multisample: desc.multisample,
            texture,
        });
    }
}

struct CxPlatformTextureInner {
    is_inited: bool,
    width: u64,
    height: u64,
    format: TextureFormat,
    multisample: Option<usize>,
    texture: RcObjcId,
}

enum AttachmentKind {
    Color,
    Depth,
}

/// TODO(JP): Can we use a regular [`std::sync::RwLock`] here instead?
#[derive(Default)]
struct MetalRwLock<T> {
    inner: Arc<MetalRwLockInner>,
    value: T,
}

impl<T> MetalRwLock<T> {
    fn cpu_read(&self) -> &T {
        &self.value
    }

    fn gpu_read(&self) -> MetalRwLockGpuReadGuard {
        #[allow(clippy::mutex_atomic)]
        let mut reader_count = self.inner.reader_count.lock().unwrap();
        *reader_count += 1;
        MetalRwLockGpuReadGuard { inner: self.inner.clone() }
    }

    fn cpu_write(&mut self) -> &mut T {
        #[allow(clippy::mutex_atomic)]
        let mut reader_count = self.inner.reader_count.lock().unwrap();
        while *reader_count != 0 {
            reader_count = self.inner.condvar.wait(reader_count).unwrap();
        }
        &mut self.value
    }
}

#[derive(Default)]
#[allow(clippy::mutex_atomic)]
// This is a false positive since Clippy is unable to detect us
// waiting on the Mutex in `cpu_write`
struct MetalRwLockInner {
    reader_count: Mutex<usize>,
    condvar: Condvar,
}

struct MetalRwLockGpuReadGuard {
    inner: Arc<MetalRwLockInner>,
}

impl Drop for MetalRwLockGpuReadGuard {
    fn drop(&mut self) {
        #[allow(clippy::mutex_atomic)]
        let mut reader_count = self.inner.reader_count.lock().unwrap();
        *reader_count -= 1;
        if *reader_count == 0 {
            self.inner.condvar.notify_one();
        }
    }
}
