use crate::ident::Ident;
use std::fmt;
use std::slice;

#[derive(Clone, Debug)]
pub(crate) struct Swizzle {
    indices: Vec<usize>,
}

#[allow(clippy::len_without_is_empty)]
impl Swizzle {
    pub(crate) fn parse(ident: Ident) -> Option<Swizzle> {
        let mut indices = Vec::new();
        ident.with(|string| {
            let mut chars = string.chars();
            let mut ch = chars.next().unwrap();
            match ch {
                'x' | 'y' | 'z' | 'w' => loop {
                    indices.push(match ch {
                        'x' => 0,
                        'y' => 1,
                        'z' => 2,
                        'w' => 3,
                        _ => return None,
                    });
                    ch = match chars.next() {
                        Some(ch) => ch,
                        None => break,
                    };
                },
                'r' | 'g' | 'b' | 'a' => loop {
                    indices.push(match ch {
                        'r' => 0,
                        'g' => 1,
                        'b' => 2,
                        'a' => 3,
                        _ => return None,
                    });
                    ch = match chars.next() {
                        Some(ch) => ch,
                        None => break,
                    };
                },
                _ => return None,
            }
            Some(Swizzle { indices })
        })
    }

    pub(crate) fn from_range(start: usize, end: usize) -> Swizzle {
        let mut indices = Vec::new();
        for index in start..end {
            indices.push(index)
        }
        Swizzle { indices }
    }

    pub(crate) fn len(&self) -> usize {
        self.indices.len()
    }

    pub(crate) fn iter(&self) -> Iter {
        Iter(self.indices.iter())
    }
}

impl fmt::Display for Swizzle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for index in &self.indices {
            write!(
                f,
                "{}",
                match index {
                    0 => "x",
                    1 => "y",
                    2 => "z",
                    3 => "w",
                    _ => panic!(),
                }
            )?;
        }
        Ok(())
    }
}
impl<'a> IntoIterator for &'a Swizzle {
    type Item = &'a usize;
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Iter<'a> {
        self.iter()
    }
}

pub(crate) struct Iter<'a>(slice::Iter<'a, usize>);

impl<'a> Iterator for Iter<'a> {
    type Item = &'a usize;

    fn next(&mut self) -> Option<&'a usize> {
        self.0.next()
    }
}
