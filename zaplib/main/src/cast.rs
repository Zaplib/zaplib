//! Functions for semi-safely casting various things to each other.
//!
//! We mostly use this for slices of geometry and instance data, since you want
//! to be able to write simple Rust `struct`s to manage your shader data, which
//! we then read out as [`f32`] slices or [`Vec`]s.
//!
//! Given a `struct` like
//!
//! ```
//! #[repr(C)]
//! struct MyStruct(u32, u32);
//! ```
//!
//! the following casts are supported and safe:
//!
//! 1. `cast_slice::<MyStruct, u32>(..)`
//! 2. `cast_slice::<u32, MyStruct>(..)` if the slice has the right `len`
//! 3. `cast_slice::<MyStruct, u8>(..)`
//! 4. `cast_vec::<MyStruct, u32>(..)`
//! 5. `cast_vec::<u32, MyStruct>(..)` if the Vec has the right `len` and `capacity`
//!
//! The following casts cause a panic:
//! 1. `cast_slice::<u8, MyStruct>(..)` "from" alignment is not a multiple of "to" alignment
//! 2. `cast_slice::<MyStruct, u64>(..)` "from" alignment is not a multiple of "to" alignment
//! 3. `cast_vec::<MyStruct, u64>(..)` "from" alignment is different from "to" alignment
//! 4. `cast_vec::<MyStruct, u8>(..)` "from" alignment is different from "to" alignment
//!
//! The following casts are unsafe but DON'T cause a panic. Please just don't do them.
//! 1. `cast_slice::<u32, Option<u32>>` or any other data structure that can't be arbitrarily initialized
//! 2. `cast_slice::<u32, (u32, u32)>` doesn't use `#[repr(C)]`, so might behave unexpectedly
//! 3. `cast_vec` with a custom allocator (requires the unstable
//!    [allocator_api feature](https://doc.rust-lang.org/beta/unstable-book/library-features/allocator-api.html))
//!
//!
//! TODO(JP): We should check that you're casting to a struct that can be initialized with
//! arbitrary byte data. Ideally Rust should implement something like an
//! [`Arbitrary` trait](https://github.com/rust-lang/rfcs/issues/2626#issuecomment-633385808).
//! We currently use `'static + Copy` as a proxy for such a trait, since at least this disallows
//! anything that implements `Drop` or has (non-static) references. We should also update
//! places like [`crate::Area::get_slice`] when improving this.
//!
//! Even better could be to add a custom `derive` macro to make sure that a struct only contains
//! a certain type of scalar, e.g. `#[derive(OnlyScalars<f32>)]` for a typical geometry or instance
//! struct. Then we can prevent a bunch of runtime panics at compile time.
//!
//! We can also use something like <https://docs.rs/repr-trait/latest/repr_trait/trait.C.html>
//! to make sure the data is properly represented using `#[repr(C)]`.
//!
//! See:
//! - <https://users.rust-lang.org/t/why-does-vec-from-raw-parts-require-same-size-and-not-same-size-capacity/73036>
//! - <https://github.com/nabijaczleweli/safe-transmute-rs/issues/16#issuecomment-471066699>
//! - <https://rawcdn.githack.com/nabijaczleweli/safe-transmute-rs/doc/safe_transmute/fn.transmute_vec.html>
//! - <https://github.com/rust-lang/rust/blob/190feb65290d39d7ab6d44e994bd99188d339f16/src/libstd/sys/windows/alloc.rs#L46-L57>
//! - <https://github.com/rust-lang/rust/blob/b6f580acc0ce233d5c4d1f9680d354fded88b824/library/std/src/sys/common/alloc.rs#L30>

/// Cast a slice from one type to another.
///
/// See [`crate::cast`] for more information.
pub fn cast_slice<FROM: 'static + Copy, TO: 'static + Copy>(slice: &[FROM]) -> &[TO] {
    assert_eq!(
        std::mem::align_of::<FROM>() % std::mem::align_of::<TO>(),
        0,
        "cast_slice: Alignment of FROM must be equal to -- or a multiple of -- the alignment of TO"
    );
    if std::mem::size_of::<FROM>() >= std::mem::size_of::<TO>() {
        // E.g. going from `(f32,f32,f32)` to `f32`.
        assert_eq!(
            std::mem::size_of::<FROM>() % std::mem::size_of::<TO>(),
            0,
            "cast_slice: Size of FROM must be equal to -- or a multiple of -- the size of TO"
        );
        unsafe {
            std::slice::from_raw_parts(
                slice.as_ptr() as *const TO,
                slice.len() * (std::mem::size_of::<FROM>() / std::mem::size_of::<TO>()),
            )
        }
    } else {
        // E.g. going from `f32` to `(f32,f32,f32)`.
        assert_eq!(
            std::mem::size_of::<TO>() % std::mem::size_of::<FROM>(),
            0,
            "cast_slice: Size of TO must be equal to -- or a multiple of -- the size of FROM"
        );
        let factor = std::mem::size_of::<TO>() / std::mem::size_of::<FROM>();
        assert_eq!(slice.len() % factor, 0, "cast_slice: slice.len() must be able to precisely fit TO without a remainder");
        unsafe { std::slice::from_raw_parts(slice.as_ptr() as *const TO, slice.len() / factor) }
    }
}

/// Cast a [`Vec`] from one type to another.
///
/// See [`crate::cast`] for more information
pub fn cast_vec<FROM: 'static + Copy, TO: 'static + Copy>(mut vec: Vec<FROM>) -> Vec<TO> {
    // We have to be careful when casting [`Vec`]s. The
    // reason for this is that when the new, casted [`Vec`] gets dropped, then
    // [`std::alloc::GlobalAlloc::dealloc`] function will get called with a different
    // [`std::alloc::Layout`], which can cause memory corruption. This means that we
    // need the exact same `align`, and we need to make sure that `size*capacity` also
    // remains the same.
    //
    // We might want to consider writing our own allocator that doesn't care about
    // different layouts. This is maybe a bit crazy, but might enable simpler code
    // in some places. However, for now these casts will suffice.
    // See https://github.com/Zaplib/zaplib/issues/103
    assert_eq!(std::mem::align_of::<FROM>(), std::mem::align_of::<TO>(), "cast_vec: Alignments of both types must be the same");

    let new_vec = if std::mem::size_of::<FROM>() >= std::mem::size_of::<TO>() {
        // E.g. going from `(f32,f32,f32)` to `f32`.
        assert_eq!(
            std::mem::size_of::<FROM>() % std::mem::size_of::<TO>(),
            0,
            "cast_vec: Size of FROM must be equal to -- or a multiple of -- the size of TO"
        );
        let factor = std::mem::size_of::<FROM>() / std::mem::size_of::<TO>();
        unsafe { Vec::from_raw_parts(vec.as_mut_ptr() as *mut TO, vec.len() * factor, vec.capacity() * factor) }
    } else {
        // E.g. going from `f32` to `(f32,f32,f32)`.
        assert_eq!(
            std::mem::size_of::<TO>() % std::mem::size_of::<FROM>(),
            0,
            "cast_vec: Size of TO must be equal to -- or a multiple of -- the size of FROM"
        );
        let factor = std::mem::size_of::<TO>() / std::mem::size_of::<FROM>();
        assert_eq!(vec.len() % factor, 0, "cast_vec: vec.len() must be able to precisely fit TO without a remainder");
        assert_eq!(vec.capacity() % factor, 0, "cast_vec: vec.capacity() must be able to precisely fit TO without a remainder");
        unsafe { Vec::from_raw_parts(vec.as_mut_ptr() as *mut TO, vec.len() / factor, vec.capacity() / factor) }
    };

    std::mem::forget(vec);
    new_vec
}

#[cfg(test)]
mod tests {
    use crate::cast::*;

    #[test]
    fn test_casts() {
        #[derive(Clone, Copy, PartialEq, Debug)]
        #[repr(C)]
        struct MyStruct(u32, u32);
        let vec = vec![MyStruct(1, 2), MyStruct(3, 4)];
        assert_eq!(cast_slice::<MyStruct, u32>(&vec), &[1, 2, 3, 4]);
        assert_eq!(cast_slice::<u32, MyStruct>(cast_slice::<MyStruct, u32>(&vec)), &vec);
        assert_eq!(
            cast_slice::<MyStruct, u8>(&vec),
            cast_slice::<[u8; 4], u8>(&[1u32.to_ne_bytes(), 2u32.to_ne_bytes(), 3u32.to_ne_bytes(), 4u32.to_ne_bytes()])
        );

        assert_eq!(cast_vec::<MyStruct, u32>(vec.clone()), vec![1, 2, 3, 4]);
        assert_eq!(cast_vec::<u32, MyStruct>(cast_vec::<MyStruct, u32>(vec.clone())), vec);
    }
}
