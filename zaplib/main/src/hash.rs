//! Hash functions and data structures.

/// Turn a string into a uniquely corresponding number. Useful for if you don't
/// need to refer back to the original string again. Also see
/// `zaplib_shader_compiler::ident::Interner` for if you do need access to original
/// strings.
/// TODO(JP): Might be good to replace all of this wholesale with an existing
/// "interner" library. See `zaplib_shader_compiler::ident::Interner` for more details.
#[derive(PartialEq, Copy, Clone, Hash, Eq, Debug, PartialOrd, Ord)]
struct StringHash<'a> {
    hash: u64,
    string: &'a str,
}
impl<'a> StringHash<'a> {
    /// Get a new [`StringHash`] for a given [`str`].
    const fn new(string: &'a str) -> Self {
        let bytes = string.as_bytes();
        let len = bytes.len();
        let mut hash = 1125899906842597u64;
        let mut i = 0;
        while i < len {
            hash = hash.wrapping_mul(31).wrapping_add(bytes[i] as u64);
            i += 1;
        }
        Self { hash, string }
    }
}

/// Represents a particular place in the code. Useful e.g. for shaders; see the
/// documentation of [`crate::Shader`].
///
/// TODO(JP): Having [`Default`] on [`LocationHash`] is not so great, because you
/// typically want to define it explicitly. Would be good to remove at some
/// point.
#[derive(Default, PartialEq, Copy, Clone, Hash, Eq, Debug, PartialOrd, Ord)]
pub struct LocationHash(pub u64);
impl LocationHash {
    /// Get a [`LocationHash`] for the given file/line/column. See also [`location_hash!`].
    pub const fn new(path: &str, line: u64, col: u64) -> Self {
        let val = StringHash::new(path).hash;
        LocationHash(val.wrapping_mul(31).wrapping_add(line).wrapping_mul(31).wrapping_add(col))
    }
}
