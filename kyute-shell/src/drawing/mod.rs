pub mod brush;
pub mod context;
pub mod effect;
pub mod gradient;
pub mod path;

use crate::bindings::Windows::{
    Foundation::Numerics::Matrix3x2,
    Win32::Direct2D::{D2D1_COLOR_F, D2D_POINT_2F, D2D_RECT_F},
};
pub use brush::{Brush, IntoBrush};
pub use context::{
    Bitmap, CompositeMode, DrawContext, DrawTextOptions, InterpolationMode, PrimitiveBlend,
};
pub use gradient::{ColorInterpolationMode, ExtendMode, GradientStopCollection};
pub use path::PathGeometry;

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

/// Numeric types that can be converted to a size in DIPs.
pub trait IntoDip {
    fn into_dip(self, ctx: &DrawContext) -> DipLength;
}

/// Common graphics types
pub type Size = euclid::Size2D<f64, Dip>;
pub type Rect = euclid::Rect<f64, Dip>;
pub type Offset = euclid::Vector2D<f64, Dip>;
pub type Point = euclid::Point2D<f64, Dip>;
pub type Transform = euclid::Transform2D<f64, Dip, Dip>;
pub type Color = palette::Srgba;
pub type Length = DipLength;

pub trait RectExt {
    fn stroke_inset(self, width: f64) -> Self;
}

impl RectExt for Rect {
    fn stroke_inset(self, width: f64) -> Self {
        self.inflate(-width * 0.5, -width * 0.5)
    }
}

pub(crate) fn mk_color_f(color: Color) -> D2D1_COLOR_F {
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
}
