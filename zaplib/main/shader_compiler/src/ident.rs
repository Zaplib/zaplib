use std::borrow::Cow;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt;
use std::fmt::Write;

/// An "interned" string, using the [`Interner`] below. This is a data structure
/// that speeds up strings if you have a lot of them. See for more info:
/// <https://en.wikipedia.org/wiki/String_interning>
/// TODO(JP): We might want to replace this with a crate that is probably faster,
/// like "string-interner" or "lasso". That might also come in handy if we are
/// looking to do more concurrency stuff. See also:
/// <https://dev.to/cad97/string-interners-in-rust-797>
/// And see `zaplib::LocationHash`.
#[derive(Clone, Copy, Default, Eq, Hash, PartialEq)]
pub struct Ident(usize);
impl Ident {
    pub(crate) fn new<'a, S>(string: S) -> Ident
    where
        S: Into<Cow<'a, str>>,
    {
        let string = string.into();
        Interner::with(|interner| {
            Ident(if let Some(index) = interner.indices.get(string.as_ref()).cloned() {
                index
            } else {
                let string = string.into_owned();
                let string_index = interner.strings.len();
                interner.strings.push(string.clone());
                interner.indices.insert(string, string_index);
                string_index
            })
        })
    }

    pub fn with<F, R>(self, f: F) -> R
    where
        F: FnOnce(&str) -> R,
    {
        Interner::with(|interner| f(&interner.strings[self.0]))
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
        Interner::with(|interner| interner.strings[self.0].cmp(&interner.strings[other.0]))
    }
}

impl PartialOrd for Ident {
    fn partial_cmp(&self, other: &Ident) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// See comment for [`Ident`].
#[derive(Debug)]
struct Interner {
    strings: Vec<String>,
    indices: HashMap<String, usize>,
}

static mut INTERNER: *mut Interner = std::ptr::null_mut();

impl Interner {
    fn get_singleton() -> &'static mut Interner {
        unsafe {
            if INTERNER.is_null() {
                INTERNER = Box::into_raw(Box::new(Interner {
                    strings: {
                        let mut v = Vec::new();
                        v.push("".to_string());
                        v
                    },
                    indices: {
                        let mut h = HashMap::new();
                        h.insert("".to_string(), 0);
                        h
                    },
                }))
            }
            &mut *INTERNER
        }
    }

    fn with<F, R>(f: F) -> R
    where
        F: FnOnce(&mut Interner) -> R,
    {
        f(Interner::get_singleton())
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
