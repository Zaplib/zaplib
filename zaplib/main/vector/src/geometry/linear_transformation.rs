use crate::geometry::{Point, Transformation, Vector};

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub(crate) struct LinearTransformation {
    pub(crate) x: Vector,
    pub(crate) y: Vector,
}

impl LinearTransformation {
    pub(crate) fn new(x: Vector, y: Vector) -> LinearTransformation {
        LinearTransformation { x, y }
    }

    pub(crate) fn identity() -> LinearTransformation {
        LinearTransformation::new(Vector::new(1.0, 0.0), Vector::new(0.0, 1.0))
    }

    pub(crate) fn scaling(v: Vector) -> LinearTransformation {
        LinearTransformation::new(Vector::new(v.x, 0.0), Vector::new(0.0, v.y))
    }

    pub(crate) fn uniform_scaling(k: f32) -> LinearTransformation {
        LinearTransformation::scaling(Vector::new(k, k))
    }

    pub(crate) fn scale(self, v: Vector) -> LinearTransformation {
        LinearTransformation::new(self.x * v.x, self.y * v.y)
    }

    pub(crate) fn uniform_scale(self, k: f32) -> LinearTransformation {
        LinearTransformation::new(self.x * k, self.y * k)
    }

    // pub(crate) fn compose(self, other: LinearTransformation) -> LinearTransformation {
    //     LinearTransformation::new(self.transform_vector(other.x), self.transform_vector(other.y))
    // }
}

impl Transformation for LinearTransformation {
    fn transform_point(&self, p: Point) -> Point {
        (self.x * p.x + self.y * p.y).to_point()
    }

    fn transform_vector(&self, v: Vector) -> Vector {
        self.x * v.x + self.y * v.y
    }
}
