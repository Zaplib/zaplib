use crate::geometry::{LinearTransformation, Point, Transform, Transformation, Vector};

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub struct AffineTransformation {
    pub(crate) xy: LinearTransformation,
    pub(crate) z: Vector,
}

impl AffineTransformation {
    pub(crate) fn new(xy: LinearTransformation, z: Vector) -> AffineTransformation {
        AffineTransformation { xy, z }
    }

    #[must_use]
    pub fn identity() -> AffineTransformation {
        AffineTransformation::new(LinearTransformation::identity(), Vector::zero())
    }

    #[must_use]
    pub fn scaling(v: Vector) -> AffineTransformation {
        AffineTransformation::new(LinearTransformation::scaling(v), Vector::zero())
    }

    #[must_use]
    pub fn uniform_scaling(k: f32) -> AffineTransformation {
        AffineTransformation::new(LinearTransformation::uniform_scaling(k), Vector::zero())
    }

    #[must_use]
    pub fn translation(v: Vector) -> AffineTransformation {
        AffineTransformation::new(LinearTransformation::identity(), v)
    }

    #[must_use]
    pub fn scale(self, v: Vector) -> AffineTransformation {
        AffineTransformation::new(self.xy.scale(v), self.z.scale(v))
    }

    #[must_use]
    pub fn uniform_scale(self, k: f32) -> AffineTransformation {
        AffineTransformation::new(self.xy.uniform_scale(k), self.z * k)
    }

    #[must_use]
    pub fn translate(self, v: Vector) -> AffineTransformation {
        AffineTransformation::new(self.xy, self.z + v)
    }
}

impl Transformation for AffineTransformation {
    fn transform_point(&self, p: Point) -> Point {
        p.transform(&self.xy) + self.z
    }

    fn transform_vector(&self, v: Vector) -> Vector {
        v.transform(&self.xy)
    }
}
