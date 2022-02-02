#[derive(Copy, Clone, Debug)]
pub enum Axis {
    Horizontal,
    Vertical,
}

impl Default for Axis {
    fn default() -> Self {
        Axis::Horizontal
    }
}
