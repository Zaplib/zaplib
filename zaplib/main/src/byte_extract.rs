//! Read the desired data type in as few instructions as possible. This is technically
//! unsafe if you're reading past the end of a buffer, but we're not going to worry
//! about that for performance. This saves a relatively expensive boundary
//! check when using safe functions like `from_le_bytes` with `try_into().unwrap()`.
//!
//! TODO(JP): see if we can implement this with generics; might be tricky since you
//! can only specialize on traits. Or might be able to use code generation here?

#[inline]
pub fn get_i8_le(data: &[u8], offset: usize) -> i8 {
    unsafe { *(data.as_ptr().add(offset) as *const i8) }
}
#[inline]
pub fn get_u8_le(data: &[u8], offset: usize) -> u8 {
    unsafe { *(data.as_ptr().add(offset) as *const u8) }
}
#[inline]
pub fn get_i16_le(data: &[u8], offset: usize) -> i16 {
    unsafe { i16::from_le(*(data.as_ptr().add(offset) as *const i16)) }
}
#[inline]
pub fn get_u16_le(data: &[u8], offset: usize) -> u16 {
    unsafe { u16::from_le(*(data.as_ptr().add(offset) as *const u16)) }
}
#[inline]
pub fn get_i32_le(data: &[u8], offset: usize) -> i32 {
    unsafe { i32::from_le(*(data.as_ptr().add(offset) as *const i32)) }
}
#[inline]
pub fn get_u32_le(data: &[u8], offset: usize) -> u32 {
    unsafe { u32::from_le(*(data.as_ptr().add(offset) as *const u32)) }
}
#[inline]
pub fn get_i64_le(data: &[u8], offset: usize) -> i64 {
    unsafe { i64::from_le(*(data.as_ptr().add(offset) as *const i64)) }
}
#[inline]
pub fn get_u64_le(data: &[u8], offset: usize) -> u64 {
    unsafe { u64::from_le(*(data.as_ptr().add(offset) as *const u64)) }
}
#[inline]
pub fn get_f32_le(data: &[u8], offset: usize) -> f32 {
    unsafe { f32::from_bits(u32::from_le(*(data.as_ptr().add(offset) as *const u32))) }
}
#[inline]
pub fn get_f64_le(data: &[u8], offset: usize) -> f64 {
    unsafe { f64::from_bits(u64::from_le(*(data.as_ptr().add(offset) as *const u64))) }
}

// Cast to f32; common for 3d rendering.
#[inline]
pub fn get_i8_le_as_f32(data: &[u8], offset: usize) -> f32 {
    get_i8_le(data, offset) as f32
}
#[inline]
pub fn get_u8_le_as_f32(data: &[u8], offset: usize) -> f32 {
    get_u8_le(data, offset) as f32
}
#[inline]
pub fn get_i16_le_as_f32(data: &[u8], offset: usize) -> f32 {
    get_i16_le(data, offset) as f32
}
#[inline]
pub fn get_u16_le_as_f32(data: &[u8], offset: usize) -> f32 {
    get_u16_le(data, offset) as f32
}
#[inline]
pub fn get_i32_le_as_f32(data: &[u8], offset: usize) -> f32 {
    get_i32_le(data, offset) as f32
}
#[inline]
pub fn get_u32_le_as_f32(data: &[u8], offset: usize) -> f32 {
    get_u32_le(data, offset) as f32
}
#[inline]
pub fn get_i64_le_as_f32(data: &[u8], offset: usize) -> f32 {
    get_i64_le(data, offset) as f32
}
#[inline]
pub fn get_u64_le_as_f32(data: &[u8], offset: usize) -> f32 {
    get_u64_le(data, offset) as f32
}
#[inline]
pub fn get_f32_le_as_f32(data: &[u8], offset: usize) -> f32 {
    get_f32_le(data, offset) as f32
}
#[inline]
pub fn get_f64_le_as_f32(data: &[u8], offset: usize) -> f32 {
    get_f64_le(data, offset) as f32
}
