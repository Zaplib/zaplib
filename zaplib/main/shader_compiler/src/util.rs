use std::fmt;

pub(crate) struct CommaSep<'a, T>(pub(crate) &'a [T]);

impl<'a, T> fmt::Display for CommaSep<'a, T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut sep = "";
        for item in self.0 {
            write!(f, "{}{}", sep, item)?;
            sep = ", ";
        }
        Ok(())
    }
}

pub(crate) struct PrettyPrintedFloat(pub(crate) f32);

impl fmt::Display for PrettyPrintedFloat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.0.abs().fract() < 0.00000001 {
            write!(f, "{}.0", self.0)
        } else {
            write!(f, "{}", self.0)
        }
    }
}
