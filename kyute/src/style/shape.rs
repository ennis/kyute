use crate::Length;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Shape {
    RoundedRect { radii: [Length; 4] },
}

impl Shape {
    pub const fn rectangle() -> Shape {
        Shape::RoundedRect {
            radii: [Length::zero(); 4],
        }
    }
}
