pub mod brush;
pub mod context;

pub use brush::Brush;
pub use brush::IntoBrush;
pub use brush::LinearGradientBrush;
pub use brush::RadialGradientBrush;
pub use brush::SolidColorBrush;
pub use context::DrawContext;
pub use context::DrawTextOptions;

use winapi::um::d2d1::{D2D1_COLOR_F, D2D1_MATRIX_3X2_F, D2D1_POINT_2F, D2D1_RECT_F};

/// Common graphics types
pub type Size = euclid::default::Size2D<f64>;
pub type Rect = euclid::default::Rect<f64>;
pub type Offset = euclid::default::Vector2D<f64>;
pub type Point = euclid::default::Point2D<f64>;
pub type Transform = euclid::default::Transform2D<f64>;
pub type Color = palette::Srgba;

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

pub(crate) fn mk_point_f(point: Point) -> D2D1_POINT_2F {
    D2D1_POINT_2F {
        x: point.x as f32,
        y: point.y as f32,
    }
}

pub(crate) fn mk_rect_f(rect: Rect) -> D2D1_RECT_F {
    let ((l, t), (r, b)) = (rect.min().to_tuple(), rect.max().to_tuple());
    D2D1_RECT_F {
        left: l as f32,
        top: t as f32,
        right: r as f32,
        bottom: b as f32,
    }
}

pub(crate) fn mk_matrix_3x2(t: &Transform) -> D2D1_MATRIX_3X2_F {
    D2D1_MATRIX_3X2_F {
        matrix: [
            [t.m11 as f32, t.m12 as f32],
            [t.m21 as f32, t.m22 as f32],
            [t.m31 as f32, t.m32 as f32],
        ],
    }
}
