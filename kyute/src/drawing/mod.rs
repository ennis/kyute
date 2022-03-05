//! Drawing-related wrappers and helpers for use with skia.
mod image;
mod svg_path;

pub use image::{Image, ImageCache, IMAGE_CACHE};
pub(crate) use svg_path::svg_path_to_skia;

use crate::{Color, Offset, Point, Rect, Size};
use kyute_common::Transform;
use skia_safe as sk;

/// Types that can be converted to their skia equivalent.
pub trait ToSkia {
    type Target;
    fn to_skia(&self) -> Self::Target;
}

/// Types that can be converted from their skia equivalent.
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
            size: Size::new((value.right - value.left) as f64, (value.bottom - value.top) as f64),
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

impl ToSkia for Color {
    type Target = sk::Color4f;

    fn to_skia(&self) -> Self::Target {
        let (r, g, b, a) = self.to_rgba();
        skia_safe::Color4f { r, g, b, a }
    }
}

impl FromSkia for Color {
    type Source = skia_safe::Color4f;

    fn from_skia(value: Self::Source) -> Self {
        Color::new(value.r as f64, value.g as f64, value.b as f64, value.a as f64)
    }
}

impl ToSkia for Transform {
    type Target = sk::Matrix;

    fn to_skia(&self) -> Self::Target {
        sk::Matrix::new_all(
            self.m11 as sk::scalar,
            self.m21 as sk::scalar,
            self.m31 as sk::scalar,
            self.m12 as sk::scalar,
            self.m22 as sk::scalar,
            self.m32 as sk::scalar,
            0.0,
            0.0,
            1.0,
        )
    }
}
