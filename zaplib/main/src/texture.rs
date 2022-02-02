//! Managing GPU textures.

use crate::*;

/// A persistent reference to a GPU texture.
///
/// TODO(JP): The way this works is a bit inconsistent with other draw tree structures, like
/// [`View`], [`Area`], and so on. These never get reused. It might be nicer to allocate these
/// dynamically and clean them up if they don't get used in a draw tree.
#[derive(Debug, Default)]
pub struct Texture {
    pub(crate) handle: Option<TextureHandle>,
}

impl Texture {
    pub fn get_color(&mut self, cx: &mut Cx) -> TextureHandle {
        if let Some(handle) = self.handle {
            handle
        } else {
            let handle = TextureHandle {
                texture_id: {
                    cx.textures.push(CxTexture::default());
                    (cx.textures.len() - 1) as u32
                },
            };
            self.handle = Some(handle);
            handle
        }
    }

    pub fn get_depth(&mut self, cx: &mut Cx) -> TextureHandle {
        if let Some(handle) = self.handle {
            handle
        } else {
            let handle = TextureHandle {
                texture_id: {
                    cx.textures.push(CxTexture {
                        desc: TextureDesc { format: TextureFormat::Depth32Stencil8, ..TextureDesc::default() },
                        ..CxTexture::default()
                    });
                    (cx.textures.len() - 1) as u32
                },
            };
            self.handle = Some(handle);
            handle
        }
    }

    pub fn get_with_dimensions(&mut self, cx: &mut Cx, width: usize, height: usize) -> TextureHandle {
        if let Some(handle) = self.handle {
            handle
        } else {
            let handle = TextureHandle {
                texture_id: {
                    let cx_texture = CxTexture {
                        desc: TextureDesc { width: Some(width), height: Some(height), ..Default::default() },
                        image_u32: vec![0; width * height],
                        ..CxTexture::default()
                    };
                    cx.textures.push(cx_texture);
                    (cx.textures.len() - 1) as u32
                },
            };
            self.handle = Some(handle);
            handle
        }
    }

    pub fn unwrap_texture_handle(&self) -> TextureHandle {
        self.handle.unwrap()
    }
}

/// A pointer to a [`CxTexture`] (indexed in [`Cx::textures`] using [`TextureHandle::texture_id`]),
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct TextureHandle {
    pub(crate) texture_id: u32,
}

impl TextureHandle {
    pub fn get_image_mut<'a>(&self, cx: &'a mut Cx) -> &'a mut [u32] {
        let cx_texture = cx.textures.get_mut(self.texture_id as usize).unwrap();
        cx_texture.update_image = true;
        &mut cx_texture.image_u32
    }
}

// TODO(Paras): Standardize and test all platforms on RGBA.
// TODO(Paras): Make image_u32 updating work on Linux.
#[derive(Copy, Clone, PartialEq)]
pub(crate) enum TextureFormat {
    ImageRGBA,
    Depth32Stencil8,
}

#[derive(Clone, PartialEq)]
pub(crate) struct TextureDesc {
    pub(crate) format: TextureFormat,
    pub(crate) width: Option<usize>,
    pub(crate) height: Option<usize>,
    pub(crate) multisample: Option<usize>,
}

impl Default for TextureDesc {
    fn default() -> Self {
        TextureDesc { format: TextureFormat::ImageRGBA, width: None, height: None, multisample: None }
    }
}

/// Texture data, which you can render as an image in your shaders. See e.g. [`crate::ImageIns`].
#[derive(Default)]
pub(crate) struct CxTexture {
    pub(crate) desc: TextureDesc,
    pub(crate) image_u32: Vec<u32>,
    pub(crate) update_image: bool,
    // Not used on wasm
    #[cfg_attr(target_arch = "wasm32", allow(dead_code))]
    pub(crate) platform: CxPlatformTexture,
}
