//! Functions for (semi-)safely casting various things to each other.
//!
//! We mostly use this for slices of geometry and instance data, since you want
//! to be able to write simple Rust `struct`s to manage your shader data, which
//! we then read out as [`f32`] slices.
//!
//! TODO(JP): This is semi-safe, since we currently don't enforce that the
//! structs that get passed in indeed only contain f32 data. We might want to
//! add a derive-macro that throws an error if it's not valid data, and tags it
//! with a trait if it is valid.
//!
//! TODO(JP): We currently cannot safely cast [`Vec`]s of data, only slices. The
//! reason for this is that when the new, casted [`Vec`] gets dropped, then
//! [`std::alloc::GlobalAlloc::dealloc`] function will get called with a different
//! [`std::alloc::Layout`], which can cause memory corruption. We might want to
//! consider writing our own allocator that doesn't care about different layouts.
//! This is maybe a bit crazy, but might enable simpler code in a bunch of places!
//!
//! See:
//! - <https://github.com/nabijaczleweli/safe-transmute-rs/issues/16#issuecomment-471066699>
//! - <https://rawcdn.githack.com/nabijaczleweli/safe-transmute-rs/doc/safe_transmute/fn.transmute_vec.html>
//! - <https://github.com/rust-lang/rust/blob/190feb65290d39d7ab6d44e994bd99188d339f16/src/libstd/sys/windows/alloc.rs#L46-L57>
//! - <https://github.com/rust-lang/rust/blob/b6f580acc0ce233d5c4d1f9680d354fded88b824/library/std/src/sys/common/alloc.rs#L30>

/// Anything that can be represented as an [`f32`]-slice. For use in geometry and instance buffers.
///
/// See module documentation for more details.
pub(crate) trait AsF32Slice {
    fn as_f32_slice(&self) -> &[f32];
}
impl<T> AsF32Slice for [T] {
    fn as_f32_slice(&self) -> &[f32] {
        assert_eq!(std::mem::size_of::<T>() % std::mem::size_of::<f32>(), 0);
        assert_eq!(std::mem::align_of::<T>() % std::mem::align_of::<f32>(), 0);
        unsafe {
            std::slice::from_raw_parts(
                self.as_ptr() as *const _ as *const f32,
                self.len() * (std::mem::size_of::<T>() / std::mem::size_of::<f32>()),
            )
        }
    }
}
impl<T> AsF32Slice for Vec<T> {
    fn as_f32_slice(&self) -> &[f32] {
        self.as_slice().as_f32_slice()
    }
}

/// Anything that can be represented as an u32-slice. See module documentation for more details.
///
/// Keeping this separate from [`AsF32Slice`] since we just want to support very specific types here for now,
/// and in the future we might want to automatically check if the type that [`AsF32Slice`] is applied to actually
/// consists only of [`f32`]s.
pub(crate) trait AsU32Slice {
    fn as_u32_slice(&self) -> &[u32];
}
impl AsU32Slice for Vec<[u32; 3]> {
    fn as_u32_slice(&self) -> &[u32] {
        assert_eq!(std::mem::size_of::<[u32; 3]>() % std::mem::size_of::<u32>(), 0);
        assert_eq!(std::mem::align_of::<[u32; 3]>() % std::mem::align_of::<u32>(), 0);
        unsafe {
            std::slice::from_raw_parts(
                self.as_ptr() as *const _ as *const u32,
                self.len() * (std::mem::size_of::<[u32; 3]>() / std::mem::size_of::<u32>()),
            )
        }
    }
}
