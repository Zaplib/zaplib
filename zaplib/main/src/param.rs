use std::sync::Arc;

// ZapParam types that can come back from JavaScript
// Keep in sync with ZapParamType in types.ts
// TODO(Paras): This could be cleaner as an enum, but casting between u32s and enums is a bit annoying.
#[cfg(any(target_arch = "wasm32", feature = "cef"))]
pub(crate) const ZAP_PARAM_STRING: u32 = 0;
#[cfg(any(target_arch = "wasm32", feature = "cef"))]
pub(crate) const ZAP_PARAM_READ_ONLY_UINT8_BUFFER: u32 = 1;
#[cfg(any(target_arch = "wasm32", feature = "cef"))]
pub(crate) const ZAP_PARAM_UINT8_BUFFER: u32 = 2;
#[cfg(any(target_arch = "wasm32", feature = "cef"))]
pub(crate) const ZAP_PARAM_FLOAT32_BUFFER: u32 = 3;
#[cfg(any(target_arch = "wasm32", feature = "cef"))]
pub(crate) const ZAP_PARAM_READ_ONLY_FLOAT32_BUFFER: u32 = 4;
#[cfg(any(target_arch = "wasm32", feature = "cef"))]
pub(crate) const ZAP_PARAM_UINT32_BUFFER: u32 = 5;
#[cfg(any(target_arch = "wasm32", feature = "cef"))]
pub(crate) const ZAP_PARAM_READ_ONLY_UINT32_BUFFER: u32 = 6;

#[derive(Clone, Debug, PartialEq)]
pub enum ZapParam {
    /// An arbitrary string supplied by the user (e.g. JSON encoded).
    /// TODO(Paras): I wish I could just put references here, since we end up cloning the string anyways when
    /// calling zerde. But then we have to declare many lifetimes - maybe worth it.
    String(String),
    /// Buffers to pass read-only memory from JS to Rust
    ReadOnlyU8Buffer(Arc<Vec<u8>>),
    ReadOnlyU32Buffer(Arc<Vec<u32>>),
    ReadOnlyF32Buffer(Arc<Vec<f32>>),
    /// Buffers to transfer ownership of memory from JS to Rust
    MutableU8Buffer(Vec<u8>),
    MutableF32Buffer(Vec<f32>),
    MutableU32Buffer(Vec<u32>),
}

impl ZapParam {
    /// Borrow contents of `ZapParam::String` as `&str`.
    pub fn as_str(&self) -> &str {
        match self {
            ZapParam::String(v) => v,
            _ => panic!("ZapParam is not a String"),
        }
    }
    /// Borrow contents of `ZapParam::MutableU8Buffer` or `ZapParam::ReadOnlyU8Buffer` as `&[u8]`.
    pub fn as_u8_slice(&self) -> &[u8] {
        match self {
            ZapParam::MutableU8Buffer(v) => v,
            ZapParam::ReadOnlyU8Buffer(v) => v,
            _ => panic!("{:?} is not a U8Buffer or ReadOnlyU8Buffer", self),
        }
    }
    /// Borrow contents of `ZapParam::MutableU32Buffer` or `ZapParam::ReadOnlyU32Buffer` as `&[u32]`.
    pub fn as_u32_slice(&self) -> &[u32] {
        match self {
            ZapParam::MutableU32Buffer(v) => v,
            ZapParam::ReadOnlyU32Buffer(v) => v,
            _ => panic!("{:?} is not a U32Buffer or ReadOnlyU32Buffer", self),
        }
    }
    /// Borrow contents of `ZapParam::MutableF32Buffer` or `ZapParam::ReadOnlyF32Buffer` as `&[f32]`.
    pub fn as_f32_slice(&self) -> &[f32] {
        match self {
            ZapParam::MutableF32Buffer(v) => v,
            ZapParam::ReadOnlyF32Buffer(v) => v,
            _ => panic!("{:?} is not a F32Buffer or ReadOnlyF32Buffer", self),
        }
    }
    /// Get contents of `ZapParam::ReadOnlyU8Buffer`, without having to consume it.
    pub fn as_arc_vec_u8(&self) -> Arc<Vec<u8>> {
        match self {
            ZapParam::ReadOnlyU8Buffer(v) => Arc::clone(v),
            _ => panic!("{:?} is not a ReadOnlyU8Buffer", self),
        }
    }
    /// Get contents of `ZapParam::ReadOnlyU32Buffer`, without having to consume it.
    pub fn as_arc_vec_u32(&self) -> Arc<Vec<u32>> {
        match self {
            ZapParam::ReadOnlyU32Buffer(v) => Arc::clone(v),
            _ => panic!("{:?} is not a ReadOnlyU32Buffer", self),
        }
    }
    /// Get contents of `ZapParam::ReadOnlyF32Buffer`, without having to consume it.
    pub fn as_arc_vec_f32(&self) -> Arc<Vec<f32>> {
        match self {
            ZapParam::ReadOnlyF32Buffer(v) => Arc::clone(v),
            _ => panic!("{:?} is not a ReadOnlyF32Buffer", self),
        }
    }
    /// Get contents of `ZapParam::String`, consuming it.
    pub fn into_string(self) -> String {
        match self {
            ZapParam::String(v) => v,
            _ => panic!("ZapParam is not a String"),
        }
    }
    /// Get contents of `ZapParam::MutableU8Buffer`, consuming it.
    pub fn into_vec_u8(self) -> Vec<u8> {
        match self {
            ZapParam::MutableU8Buffer(v) => v,
            _ => panic!("{:?} is not a U8Buffer", self),
        }
    }
    /// Get contents of `ZapParam::MutableU32Buffer`, consuming it.
    pub fn into_vec_u32(self) -> Vec<u32> {
        match self {
            ZapParam::MutableU32Buffer(v) => v,
            _ => panic!("{:?} is not a U32Buffer", self),
        }
    }
    /// Get contents of `ZapParam::MutableF32Buffer`, consuming it.
    pub fn into_vec_f32(self) -> Vec<f32> {
        match self {
            ZapParam::MutableF32Buffer(v) => v,
            _ => panic!("{:?} is not a F32Buffer", self),
        }
    }
}

pub trait IntoParam {
    fn into_param(self) -> ZapParam;
}

impl IntoParam for String {
    fn into_param(self) -> ZapParam {
        ZapParam::String(self)
    }
}
impl IntoParam for Vec<u8> {
    fn into_param(self) -> ZapParam {
        ZapParam::MutableU8Buffer(self)
    }
}
impl IntoParam for Vec<f32> {
    fn into_param(self) -> ZapParam {
        ZapParam::MutableF32Buffer(self)
    }
}
impl IntoParam for Arc<Vec<u8>> {
    fn into_param(self) -> ZapParam {
        ZapParam::ReadOnlyU8Buffer(self)
    }
}
impl IntoParam for Arc<Vec<f32>> {
    fn into_param(self) -> ZapParam {
        ZapParam::ReadOnlyF32Buffer(self)
    }
}
impl IntoParam for Arc<Vec<u32>> {
    fn into_param(self) -> ZapParam {
        ZapParam::ReadOnlyU32Buffer(self)
    }
}
impl IntoParam for Vec<u32> {
    fn into_param(self) -> ZapParam {
        ZapParam::MutableU32Buffer(self)
    }
}
