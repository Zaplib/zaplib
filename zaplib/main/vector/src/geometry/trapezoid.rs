/// A trapezoid in 2-dimensional Euclidian space.
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub struct Trapezoid {
    pub(crate) xs: [f32; 2],
    pub(crate) ys: [f32; 4],
}
