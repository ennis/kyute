////////////////////////////////////////////////////////////////////////////////////////////////////
// Length
////////////////////////////////////////////////////////////////////////////////////////////////////

use crate::LayoutConstraints;
use kyute_common::Angle;
use std::{
    fmt,
    ops::{Mul, Neg},
};

/// Length specification.
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
            Length::Px(v) | Length::Dip(v) | Length::Em(v) if v == 0.0 => {
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
    pub const fn zero() -> Length {
        Length::Dip(0.0)
    }

    /// Convert to dips.
    pub fn compute(self, constraints: &LayoutConstraints) -> f64 {
        match self {
            Length::Px(x) => x / constraints.scale_factor,
            Length::Dip(x) => x,
            Length::Em(x) => x * constraints.parent_font_size,
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
    pub const fn zero() -> LengthOrPercentage {
        LengthOrPercentage::Length(Length::zero())
    }
}

impl LengthOrPercentage {
    /// Convert to dips, given a scale factor and a parent length for proportional length specifications.
    pub fn compute(self, constraints: &LayoutConstraints, parent_length: f64) -> f64 {
        match self {
            LengthOrPercentage::Length(x) => x.compute(constraints),
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

/*impl LengthOrPercentage {
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
    pub fn zero() -> LengthOrPercentage {
        LengthOrPercentage::Length(Length::Dip(0.0))
    }

}

impl Neg for LengthOrPercentage {
    type Output = LengthOrPercentage;

    fn neg(self) -> Self::Output {
        match self {
            LengthOrPercentage::Length(l) => LengthOrPercentage::Length(-l),
            LengthOrPercentage::Length(l) => LengthOrPercentage::Length(-l),
            LengthOrPercentage::Length(l) => LengthOrPercentage::Length(-l),
            LengthOrPercentage::Percentage(p) => LengthOrPercentage::Percentage(-p),
        }
    }
}

/// Length scaling
impl Mul<LengthOrPercentage> for f64 {
    type Output = Length;
    fn mul(self, rhs: Length) -> Self::Output {
        rhs.scale(self)
    }
}

/// Length scaling
impl Mul<f64> for LengthOrPercentage {
    type Output = Length;
    fn mul(self, rhs: f64) -> Self::Output {
        self.scale(rhs)
    }
}

impl Default for LengthOrPercentage {
    fn default() -> Self {
        LengthOrPercentage::Length(Length::Dip(0.0))
    }
}

/// By default, a naked i32 represents a dip.
impl From<i32> for LengthOrPercentage {
    fn from(v: i32) -> Self {
        LengthOrPercentage::Length(Length::Dip(v as f64))
    }
}

/// By default, a naked f64 represents a dip.
impl From<f64> for LengthOrPercentage {
    fn from(v: f64) -> Self {
        LengthOrPercentage::Length(Length::Dip(v))
    }
}*/

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
    /// Interprets the value as a length in ems.
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
    fn percent(self) -> LengthOrPercentage {
        LengthOrPercentage::Percentage(self as f64 / 100.0)
    }
    fn em(self) -> Length {
        Length::Em(self as f64)
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
