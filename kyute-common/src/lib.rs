#![feature(const_fn_floating_point_arithmetic)]
//! Basic types shared by kyute crates.

mod atom;
mod color;
pub mod counter;
mod data;

use std::{
    fmt,
    fmt::Formatter,
    ops::{Mul, Neg},
};

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

/// Values used to calculate the absolute value in DIPs of relative lengths.
pub struct RelativeLengthContext {
    /// Scale factor (pixel density ratio) of the target surface.   
    pub scale_factor: f64,
    /// Current font size in DIPs (for font-relative lengths)
    pub font_size: f64,
}

/// Represents a length, either in device-independent, absolute units (dips), device-dependent pixels,
/// or font-relative values (em).
/// TODO move into kyute? not sure it's used anywhere else
#[derive(Copy, Clone, PartialEq)]
#[cfg_attr(feature = "serializing", derive(serde::Deserialize))]
#[cfg_attr(feature = "serializing", serde(tag = "unit", content = "value"))]
pub enum Length {
    /// Actual screen pixels (the actual physical size depends on the density of the screen).
    #[cfg_attr(feature = "serializing", serde(rename = "px"))]
    Px(f64),
    /// Device-independent pixels (DIPs), close to 1/96th of an inch.
    #[cfg_attr(feature = "serializing", serde(rename = "dip"))]
    Dip(f64),
    /// Length relative to the current font size.
    Em(f64),
}

impl fmt::Debug for Length {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Length::Px(v) | Length::Dip(v) if v == 0.0 => {
                write!(f, "0")
            }
            Length::Px(v) => {
                write!(f, "{}px", v)
            }
            Length::Dip(v) => {
                write!(f, "{}dip", v)
            }
            Length::Em(v) => {
                write!(f, "{}em", v)
            }
        }
    }
}

impl Length {
    /// Scale the length by the given amount.
    pub fn scale(self, by: f64) -> Self {
        let mut v = self;
        match v {
            Length::Px(ref mut v) | Length::Dip(ref mut v) | Length::Em(ref mut v) => {
                *v *= by;
            }
        }
        v
    }

    /// Zero length.
    pub fn zero() -> Length {
        Length::Dip(0.0)
    }

    /// Convert to dips, given a scale factor and a parent length for proportional length specifications.
    pub fn to_dips(self, ctx: &RelativeLengthContext) -> f64 {
        match self {
            Length::Px(x) => x / ctx.scale_factor,
            Length::Dip(x) => x,
            Length::Em(x) => x * ctx.font_size,
        }
    }
}

impl Neg for Length {
    type Output = Length;

    fn neg(self) -> Self::Output {
        match self {
            Length::Px(v) => Length::Px(-v),
            Length::Dip(v) => Length::Dip(-v),
            Length::Em(v) => Length::Em(-v),
        }
    }
}

/// Length scaling
impl Mul<Length> for f64 {
    type Output = Length;
    fn mul(self, rhs: Length) -> Self::Output {
        rhs.scale(self)
    }
}

/// Length scaling
impl Mul<f64> for Length {
    type Output = Length;
    fn mul(self, rhs: f64) -> Self::Output {
        self.scale(rhs)
    }
}

impl Default for Length {
    fn default() -> Self {
        Length::Dip(0.0)
    }
}

/// By default, a naked i32 represents a dip.
impl From<i32> for Length {
    fn from(v: i32) -> Self {
        Length::Dip(v as f64)
    }
}

/// By default, a naked f64 represents a dip.
impl From<f64> for Length {
    fn from(v: f64) -> Self {
        Length::Dip(v)
    }
}

#[derive(Copy, Clone, PartialEq)]
#[cfg_attr(feature = "serializing", derive(serde::Deserialize))]
/// A length or a percentage.
pub enum LengthOrPercentage {
    /// Length.
    Length(Length),
    /// Percentage (normalized to the unit interval).
    Percentage(f64),
}

impl LengthOrPercentage {
    /// Convert to dips, given a scale factor and a parent length for proportional length specifications.
    pub fn to_dips(self, ctx: &RelativeLengthContext, parent_length: f64) -> f64 {
        match self {
            LengthOrPercentage::Length(x) => x.to_dips(ctx),
            LengthOrPercentage::Percentage(x) => x * parent_length,
        }
    }
}

impl fmt::Debug for LengthOrPercentage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LengthOrPercentage::Length(length) => fmt::Debug::fmt(length, f),
            LengthOrPercentage::Percentage(percentage) => write!(f, "{}%", percentage * 100.0),
        }
    }
}

impl From<Length> for LengthOrPercentage {
    fn from(length: Length) -> Self {
        LengthOrPercentage::Length(length)
    }
}

/// Trait to interpret numeric values as units of measure.
pub trait UnitExt {
    /// Interprets the value as a length in device-independent pixels (1/96 inch).
    fn dip(self) -> Length;
    /// Interprets the value as a length in inches.
    fn inch(self) -> Length;
    /// Interprets the value as a length in physical pixels.
    fn px(self) -> Length;
    /// Interprets the value as a length in points (1/72 in, 96/72 dip (4/3))
    fn pt(self) -> Length;
    /// Interprets the value as a length proportional to the font size of the parent element (1em = current font size).
    fn em(self) -> Length;
    /// Interprets the value as a length expressed as a percentage of the parent element's length.
    ///
    /// The precise definition of "parent element" depends on the context in which the length is used.
    fn percent(self) -> LengthOrPercentage;
    /// Interprets the value as an angle expressed in degrees.
    fn degrees(self) -> Angle;
    /// Interprets the value as an angle expressed in radians.
    fn radians(self) -> Angle;
}

/// Point-to-DIP conversion factor.
///
/// # Examples
///
/// ```rust
/// use kyute_common::PT_TO_DIP;
/// let size_in_points = 12.0;
/// let size_in_dips = size_in_points * PT_TO_DIP;
/// ```
pub const PT_TO_DIP: f64 = 4.0 / 3.0;

/// Inches-to-DIP conversion factor.
///
/// # Examples
///
/// ```rust
/// use kyute_common::IN_TO_DIP;
/// let size_in_inches = 2.5;
/// let size_in_dips = size_in_inches * IN_TO_DIP;
/// ```
pub const IN_TO_DIP: f64 = 96.0;

impl UnitExt for f32 {
    fn dip(self) -> Length {
        Length::Dip(self as f64)
    }
    fn inch(self) -> Length {
        Length::Dip((self as f64) * IN_TO_DIP)
    }
    fn px(self) -> Length {
        Length::Px(self as f64)
    }
    fn pt(self) -> Length {
        Length::Dip((self as f64) * PT_TO_DIP)
    }
    fn em(self) -> Length {
        Length::Em(self as f64)
    }
    fn percent(self) -> LengthOrPercentage {
        LengthOrPercentage::Percentage(self as f64 / 100.0)
    }
    fn degrees(self) -> Angle {
        Angle::degrees(self as f64)
    }
    fn radians(self) -> Angle {
        Angle::radians(self as f64)
    }
}

impl UnitExt for f64 {
    fn dip(self) -> Length {
        Length::Dip(self)
    }
    fn inch(self) -> Length {
        Length::Dip(self * IN_TO_DIP)
    }
    fn px(self) -> Length {
        Length::Px(self)
    }
    fn pt(self) -> Length {
        Length::Dip(self * PT_TO_DIP)
    }
    fn em(self) -> Length {
        Length::Em(self)
    }
    fn percent(self) -> LengthOrPercentage {
        LengthOrPercentage::Percentage(self / 100.0)
    }
    fn degrees(self) -> Angle {
        Angle::degrees(self)
    }
    fn radians(self) -> Angle {
        Angle::radians(self)
    }
}

impl UnitExt for i32 {
    fn dip(self) -> Length {
        Length::Dip(self as f64)
    }
    fn inch(self) -> Length {
        Length::Dip((self as f64) * IN_TO_DIP)
    }
    fn px(self) -> Length {
        Length::Px(self as f64)
    }
    fn pt(self) -> Length {
        Length::Dip((self as f64) * PT_TO_DIP)
    }
    fn em(self) -> Length {
        Length::Em(self as f64)
    }
    fn percent(self) -> LengthOrPercentage {
        LengthOrPercentage::Percentage(self as f64 / 100.0)
    }
    fn degrees(self) -> Angle {
        Angle::degrees(self as f64)
    }
    fn radians(self) -> Angle {
        Angle::radians(self as f64)
    }
}

impl UnitExt for u32 {
    fn dip(self) -> Length {
        Length::Dip(self as f64)
    }
    fn inch(self) -> Length {
        Length::Dip((self as f64) * IN_TO_DIP)
    }
    fn px(self) -> Length {
        Length::Px(self as f64)
    }
    fn pt(self) -> Length {
        Length::Dip((self as f64) * PT_TO_DIP)
    }
    fn em(self) -> Length {
        Length::Em(self as f64)
    }
    fn percent(self) -> LengthOrPercentage {
        LengthOrPercentage::Percentage(self as f64 / 100.0)
    }
    fn degrees(self) -> Angle {
        Angle::degrees(self as f64)
    }
    fn radians(self) -> Angle {
        Angle::radians(self as f64)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// imbl reexports and data impl
////////////////////////////////////////////////////////////////////////////////////////////////////

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
