//! Parametric SVG reader.
//!

use kyute_common::Transform;

#[derive(Copy, Clone, PartialEq, Debug)]
#[repr(transparent)]
pub struct Scalar(u32);

impl Scalar {
    pub fn resolve(&self, params: &Parameters) -> f32 {}
}

#[derive(Copy, Clone, PartialEq, Debug)]
#[repr(C)]
pub struct MoveTo {
    pub x: Scalar,
    pub y: Scalar,
}

#[derive(Copy, Clone, PartialEq, Debug)]
#[repr(C)]
pub struct LineTo {
    pub x: Scalar,
    pub y: Scalar,
}

#[derive(Copy, Clone, PartialEq, Debug)]
#[repr(C)]
pub struct Cubic {
    pub x1: Scalar,
    pub y1: Scalar,

    pub x2: Scalar,
    pub y2: Scalar,

    pub x: Scalar,
    pub y: Scalar,
}

#[derive(Copy, Clone, PartialEq, Debug)]
#[repr(C)]
pub struct Quadratic {
    pub x1: Scalar,
    pub y1: Scalar,

    pub x: Scalar,
    pub y: Scalar,
}

#[derive(Copy, Clone, PartialEq, Debug)]
#[repr(C)]
pub struct Arc {
    pub rx: Scalar,
    pub ry: Scalar,

    pub x: Scalar,
    pub y: Scalar,
}

pub enum PathSegmentType {
    MoveTo,
    LineTo,
    Cubic,
    Quadratic,
    EllipticalArc,
    Close,
}

#[derive(Copy, Clone, PartialEq, Debug)]
#[repr(C)]
pub struct PaintCode(u32, u32);

#[derive(Copy, Clone, PartialEq, Debug)]
#[repr(C)]
pub struct RawComposition {
    transform: u32,
    shape: u32,
    paint: PaintCode,
}

#[derive(Copy, Clone, PartialEq, Debug)]
#[repr(C)]
struct Header {
    param_count: u32,
    param_start: u32,
    paint_count: u32,
    paint_start: u32,
    shapes_count: u32,
    shapes_start: u32,
    comp_count: u32,
    comp_start: u32,
}

pub struct Shape {}

pub struct MiniVG {
    data: Vec<u32>,
}

impl MiniVG {
    fn header(&self) -> &Header {
        unsafe { &*(self.data.as_ptr() as *const Header) }
    }

    fn shapes(&self) -> &[Shape] {
        let offset = self.header().shapes_count;
        let count = self.header().shapes_start as usize;
        unsafe { std::slice::from_raw_parts(self.data.as_ptr().add(offset as usize) as *const Shape, count) }
    }

    fn compositions(&self) -> &[RawComposition] {
        let offset = self.header().comp_start;
        let count = self.header().comp_count as usize;
        unsafe { std::slice::from_raw_parts(self.data.as_ptr().add(offset as usize) as *const RawComposition, count) }
    }
}
