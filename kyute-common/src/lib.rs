#![feature(const_fn_floating_point_arithmetic)]
//! Basic types shared by kyute crates.

mod atom;
mod color;
pub mod counter;
mod data;

pub use crate::{
    atom::{make_unique_atom, Atom},
    color::Color,
    data::Data,
};
pub use kyute_common_macros::Data;

/// The DIP (device-independent pixel) unit.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Dip;

/// Device pixel unit (device-dependent)
pub struct Px;

/// A length in dips.
pub type DipLength = euclid::Length<f64, Dip>;

/// A length in device pixels.
pub type PxLength = euclid::Length<f64, Px>;

/// Angle.
pub type Angle = euclid::Angle<f64>;

/// One DIP.
pub const DIP: DipLength = DipLength::new(1.0);

/// One device pixel.
pub const PX: DipLength = DipLength::new(1.0);

/// 2D size in dips.
pub type Size = euclid::Size2D<f64, Dip>;
/// 2D integer size in physical pixels.
pub type SizeI = euclid::Size2D<i32, Px>;
pub type PhysicalSize = euclid::Size2D<f64, Px>;

/// Rectangle in dips
pub type Rect = euclid::Rect<f64, Dip>;
pub type RectI = euclid::Rect<i32, Px>;
/// Offset in dips.
pub type Offset = euclid::Vector2D<f64, Dip>;
/// Point in dips.
pub type Point = euclid::Point2D<f64, Dip>;
pub type PointI = euclid::Point2D<i32, Px>;
/// Point in physical pixel coordinates.
pub type PhysicalPoint = euclid::Point2D<f64, Px>;
/// Transform in dips.
//pub type Transform<Src, Dst> = euclid::Transform2D<f64, Src, Dst>;
pub type Transform = euclid::Transform2D<f64, Dip, Dip>;
pub type UnknownUnit = euclid::UnknownUnit;
/// Side offsets (top,left,right,bottom lengths) in dips
pub type SideOffsets = euclid::SideOffsets2D<f64, Dip>;

pub type DipToPx = euclid::Scale<f64, Dip, Px>;
pub type PxToDip = euclid::Scale<f64, Px, Dip>;

/// Trait for graphics types that support being rounded to the nearest pixel.
///
/// It is implemented for:
/// - Lengths: round up/down to pixel boundary; round to nearest
/// - Rects: round inside/outside
/// - Points/Vectors/Offsets: round to nearest pixel boundary
pub trait RoundToPixel {
    fn round_to_pixel(&self, scale_factor: f64) -> Self;
}

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

impl RoundToPixel for Size {
    fn round_to_pixel(&self, scale_factor: f64) -> Self {
        (*self * scale_factor).ceil() * (1.0 / scale_factor)
    }
}

impl RoundToPixel for Rect {
    fn round_to_pixel(&self, scale_factor: f64) -> Rect {
        (*self * scale_factor).round() * (1.0 / scale_factor)
    }
}

/// Additional methods for rectangles.
pub trait RectExt {
    /// Insets the rectangle so that it is centered on the inner border stroke of the specified
    /// width.
    ///
    /// The returned Rect can be used to draw border lines inside the original Rect.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let rect = Rect::new(Point::origin(), Size::new(10.0, 10.0));
    /// // Rect centered on the 1dip-wide border stroke inside `rect`.
    /// let border_stroke_rect = rect.stroke_inset(1.0);
    ///
    /// // use the adjusted rect to draw a border of width 1 inside the `rect`.
    /// canvas.draw_rect(border_stroke_rect, 1.0);
    /// ```
    fn stroke_inset(self, width: f64) -> Self;
    /// Returns the top-left corner of the rectangle (assumes lower y is up).
    fn top_left(&self) -> Point;
    /// Returns the top-right corner of the rectangle (assumes lower y is up).
    fn top_right(&self) -> Point;
    /// Returns the bottom-left corner of the rectangle (assumes lower y is up).
    fn bottom_left(&self) -> Point;
    /// Returns the bottom-right corner of the rectangle (assumes lower y is up).
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
        Point::new(self.origin.x + self.size.width, self.origin.y + self.size.height)
    }
}

#[cfg(feature = "imbl")]
pub use imbl;

// Taken from druid
#[cfg(feature = "imbl")]
impl<A: Data> Data for imbl::Vector<A> {
    fn same(&self, other: &Self) -> bool {
        // if a vec is small enough that it doesn't require an allocation
        // it is 'inline'; in this case a pointer comparison is meaningless.
        if self.is_inline() {
            self.len() == other.len() && self.iter().zip(other.iter()).all(|(a, b)| a.same(b))
        } else {
            self.ptr_eq(other)
        }
    }
}

#[cfg(feature = "imbl")]
impl<K: Clone + 'static, V: Data, S: 'static> Data for imbl::HashMap<K, V, S> {
    fn same(&self, other: &Self) -> bool {
        self.ptr_eq(&other)
    }
}

#[cfg(feature = "imbl")]
impl<A: Data, S: 'static> Data for imbl::HashSet<A, S> {
    fn same(&self, other: &Self) -> bool {
        self.ptr_eq(&other)
    }
}

#[cfg(feature = "imbl")]
impl<K: Clone + 'static, V: Data> Data for imbl::OrdMap<K, V> {
    fn same(&self, other: &Self) -> bool {
        self.ptr_eq(&other)
    }
}

#[cfg(feature = "imbl")]
impl<A: Data> Data for imbl::OrdSet<A> {
    fn same(&self, other: &Self) -> bool {
        self.ptr_eq(&other)
    }
}
