use std::borrow::Cow;
use std::cmp::Ordering;
use std::fmt;
use std::fmt::Write;

use lasso::{Spur, ThreadedRodeo};
use once_cell::sync::Lazy;

static INTERNER: Lazy<ThreadedRodeo> = Lazy::new(|| ThreadedRodeo::new());

/// An "interned" string, using the [`Interner`] below. This is a data structure
/// that speeds up strings if you have a lot of them. See for more info:
/// <https://en.wikipedia.org/wiki/String_interning>
///
/// TODO(JP): We replaced this with Lasso, but we haven't looked into what impact
/// this has on the .wasm size; probably worth checking. For example, we could represent
/// them as offsets into the original shader code, which typically is static data anyway.
#[derive(Clone, Copy, Default, Eq, Hash, PartialEq)]
pub struct Ident(Spur);
impl Ident {
    pub(crate) fn new<'a, S>(string: S) -> Ident
    where
        S: Into<Cow<'a, str>>,
    {
        Self(INTERNER.get_or_intern(string.into()))
    }

    pub fn with<F, R>(self, f: F) -> R
    where
        F: FnOnce(&str) -> R,
    {
        f(INTERNER.resolve(&self.0))
    }
}

impl fmt::Debug for Ident {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.with(|string| write!(f, "{}", string))
    }
}

impl fmt::Display for Ident {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.with(|string| write!(f, "{}", string))
    }
}

impl Ord for Ident {
    fn cmp(&self, other: &Ident) -> Ordering {
        INTERNER.resolve(&self.0).cmp(INTERNER.resolve(&other.0))
    }
}

impl PartialOrd for Ident {
    fn partial_cmp(&self, other: &Ident) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Represents a path like `self::Something` or `Something::method`.
#[derive(Clone, Default, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub(crate) struct IdentPath {
    segs: [Ident; 2],
    len: usize,
}

impl IdentPath {
    pub(crate) fn from_ident(ident: Ident) -> Self {
        IdentPath { segs: [ident, Ident::default()], len: 1 }
    }

    pub(crate) fn from_two_idents(ident1: Ident, ident2: Ident) -> Self {
        IdentPath { segs: [ident1, ident2], len: 2 }
    }

    pub(crate) fn to_struct_fn_ident(&self) -> Ident {
        let mut s = String::new();
        for i in 0..self.len {
            if i != 0 {
                write!(s, "_").unwrap();
            }
            self.segs[i].with(|string| write!(s, "{}", string)).unwrap()
        }
        Ident::new(&s)
    }

    pub(crate) fn from_str(value: &str) -> Self {
        IdentPath { segs: [Ident::new(value), Ident::default()], len: 1 }
    }

    pub(crate) fn push(&mut self, ident: Ident) -> bool {
        if self.len >= 4 {
            return false;
        }
        self.segs[self.len] = ident;
        self.len += 1;
        true
    }

    pub(crate) fn from_two(one: Ident, two: Ident) -> Self {
        IdentPath { segs: [one, two], len: 2 }
    }

    pub(crate) fn get_single(&self) -> Option<Ident> {
        if self.len != 1 {
            return None;
        }
        Some(self.segs[0])
    }
}

impl fmt::Debug for IdentPath {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for i in 0..self.len {
            if i != 0 {
                write!(f, "::").unwrap();
            }
            self.segs[i].with(|string| write!(f, "{}", string)).unwrap()
        }
        Ok(())
    }
}

impl fmt::Display for IdentPath {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for i in 0..self.len {
            if i != 0 {
                write!(f, "::").unwrap();
            }
            self.segs[i].with(|string| write!(f, "{}", string)).unwrap()
        }
        Ok(())
    }
}
