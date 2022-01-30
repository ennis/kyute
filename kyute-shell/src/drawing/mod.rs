mod color;
mod path;

pub use color::Color;
use float_cmp::ApproxEqUlps;
pub use path::{Path, PathSegment};

/// The DIP (device-independent pixel) unit.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Dip;

/// Device pixel unit (device-dependent)
pub struct Px;

/// A length in Dips.
pub type DipLength = euclid::Length<f64, Dip>;

/// A length in device pixels.
pub type PxLength = euclid::Length<f64, Px>;

/// One DIP.
pub const DIP: DipLength = DipLength::new(1.0);

/// One device pixel.
pub const PX: DipLength = DipLength::new(1.0);

/*/// Numeric types that can be converted to a size in DIPs.
pub trait IntoDip {
    fn into_dip(self, ctx: &DrawContext) -> DipLength;
}*/

/// Common graphics types
pub type Size = euclid::Size2D<f64, Dip>;
pub type PhysicalSize = euclid::Size2D<f64, Px>;
pub type Rect = euclid::Rect<f64, Dip>;
pub type Offset = euclid::Vector2D<f64, Dip>;
pub type Point = euclid::Point2D<f64, Dip>;
pub type PhysicalPoint = euclid::Point2D<f64, Px>;
pub type Transform = euclid::Transform2D<f64, Dip, Dip>;
pub type Length = DipLength;

pub trait RoundToPixel {
    fn round_to_pixel(&self, scale_factor: f64) -> Self;
}

// Lengths: round up/down to pixel boundary; round to nearest
// Rects: round inside/outside
// Points/Vectors/Offsets: round to nearest pixel boundary

impl RoundToPixel for f64 {
    fn round_to_pixel(&self, scale_factor: f64) -> Self {
        (*self * scale_factor).round() * (1.0 / scale_factor)
    }
}

impl RoundToPixel for Offset {
    fn round_to_pixel(&self, scale_factor: f64) -> Self {
        (*self * scale_factor).round() * (1.0 / scale_factor)
    }
}

impl RoundToPixel for Rect {
    fn round_to_pixel(&self, scale_factor: f64) -> Rect {
        (*self * scale_factor).round() * (1.0 / scale_factor)
    }
}

pub trait RectExt {
    fn stroke_inset(self, width: f64) -> Self;
    fn top_left(&self) -> Point;
    fn top_right(&self) -> Point;
    fn bottom_left(&self) -> Point;
    fn bottom_right(&self) -> Point;
}

impl RectExt for Rect {
    fn stroke_inset(self, width: f64) -> Self {
        self.inflate(-width * 0.5, -width * 0.5)
    }
    fn top_left(&self) -> Point {
        Point::new(self.origin.x, self.origin.y)
    }
    fn top_right(&self) -> Point {
        Point::new(self.origin.x + self.size.width, self.origin.y)
    }
    fn bottom_left(&self) -> Point {
        Point::new(self.origin.x, self.origin.y + self.size.height)
    }
    fn bottom_right(&self) -> Point {
        Point::new(
            self.origin.x + self.size.width,
            self.origin.y + self.size.height,
        )
    }
}

/*pub trait FuzzyEq<Rhs: ?Sized = Self> {
    fn fuzzy_eq(&self, other: &Rhs) -> bool;

    #[inline]
    fn fuzzy_ne(&self, other: &Rhs) -> bool {
        !self.fuzzy_eq(other)
    }
}


pub trait FuzzyZero {
    fn is_fuzzy_zero(self) -> bool;
}

impl FuzzyZero for f32 {
    fn is_fuzzy_zero(self) -> bool {
        self.approx_eq_ulps(&0.0, 2)
    }
}

impl FuzzyZero for f64 {
    fn is_fuzzy_zero(self) -> bool {
        self.approx_eq_ulps(&0.0, 2)
    }
}*/

pub trait ToSkia {
    type Target;
    fn to_skia(&self) -> Self::Target;
}

pub trait FromSkia {
    type Source;
    fn from_skia(value: Self::Source) -> Self;
}

impl ToSkia for Rect {
    type Target = skia_safe::Rect;

    fn to_skia(&self) -> Self::Target {
        skia_safe::Rect {
            left: self.origin.x as f32,
            top: self.origin.y as f32,
            right: (self.origin.x + self.size.width) as f32,
            bottom: (self.origin.y + self.size.height) as f32,
        }
    }
}

impl FromSkia for Rect {
    type Source = skia_safe::Rect;

    fn from_skia(value: Self::Source) -> Self {
        Rect {
            origin: Point::new(value.left as f64, value.top as f64),
            size: Size::new(
                (value.right - value.left) as f64,
                (value.bottom - value.top) as f64,
            ),
        }
    }
}

impl ToSkia for Point {
    type Target = skia_safe::Point;

    fn to_skia(&self) -> Self::Target {
        skia_safe::Point {
            x: self.x as f32,
            y: self.y as f32,
        }
    }
}

impl ToSkia for Offset {
    type Target = skia_safe::Vector;

    fn to_skia(&self) -> Self::Target {
        skia_safe::Vector {
            x: self.x as f32,
            y: self.y as f32,
        }
    }
}

impl FromSkia for Point {
    type Source = skia_safe::Point;

    fn from_skia(value: Self::Source) -> Self {
        Point::new(value.x as f64, value.y as f64)
    }
}

/*pub(crate) fn mk_color_f(color: Color) -> D2D1_COLOR_F {
    let (r, g, b, a) = color.into_components();
    D2D1_COLOR_F { r, g, b, a }
}

pub(crate) fn mk_point_f(point: Point) -> D2D_POINT_2F {
    D2D_POINT_2F {
        x: point.x as f32,
        y: point.y as f32,
    }
}

pub(crate) fn mk_rect_f(rect: Rect) -> D2D_RECT_F {
    let ((l, t), (r, b)) = (rect.min().to_tuple(), rect.max().to_tuple());
    D2D_RECT_F {
        left: l as f32,
        top: t as f32,
        right: r as f32,
        bottom: b as f32,
    }
}

pub(crate) fn mk_matrix_3x2(t: &Transform) -> Matrix3x2 {
    Matrix3x2 {
        M11: t.m11 as f32,
        M12: t.m12 as f32,
        M21: t.m21 as f32,
        M22: t.m22 as f32,
        M31: t.m31 as f32,
        M32: t.m32 as f32,
    }
}*/
