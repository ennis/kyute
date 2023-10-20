//! Length specification
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
    /// Length relative to the font size of the parent element.
    Em(f64),
}

/// Parameters for resolving lengths to a length in dips.
#[derive(Copy, Clone, Default)]
pub struct LengthResolutionParams {
    /// scale factor for the target device on which the length is represented
    ///
    /// Used for the resolution of `Px` lengths.
    pub scale_factor: f64,
    /// Current font size in dips.
    ///
    /// Used for the resolution of `Em` sizes.
    pub font_size: f64,
    /// Parent container size in dips.
    ///
    /// Used for the resolution of `Percentage` sizes.
    pub container_size: f64,
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
    /// A length of zero.
    pub const ZERO: Length = Length::Dip(0.0);

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

    /// Resolves an em-length.
    pub fn resolve_em(self, font_size: f64) -> Self {
        match self {
            Length::Px(_) | Length::Dip(_) => self,
            Length::Em(em) => Length::Dip(em * font_size),
        }
    }

    /// Resolves the length to a length in DIPs.
    ///
    /// # Arguments
    /// * params resolution parameters, see [`LengthResolutionParams`]
    pub fn to_dips(self, params: &LengthResolutionParams) -> f64 {
        match self {
            Length::Px(x) => x / params.scale_factor,
            Length::Dip(x) => x,
            Length::Em(x) => x * params.font_size,
        }
    }

    /// Returns this length in dips if it is specified in dips, or `None`.
    pub fn as_dips(&self) -> Option<f64> {
        match *self {
            Length::Dip(v) => Some(v),
            _ => None,
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
    /// A length of zero.
    pub const ZERO: LengthOrPercentage = LengthOrPercentage::Length(Length::ZERO);
}

impl Default for LengthOrPercentage {
    fn default() -> Self {
        Self::ZERO
    }
}

impl LengthOrPercentage {
    /// Convert to dips, given a scale factor and a parent length for proportional length specifications.
    pub fn to_dips(self, params: &LengthResolutionParams) -> f64 {
        match self {
            LengthOrPercentage::Length(x) => x.to_dips(params),
            LengthOrPercentage::Percentage(x) => x * params.container_size,
        }
    }

    /// Resolves a percentage length to a concrete length.
    ///
    /// # Arguments
    ///
    /// * container_size parent container size in dips
    pub fn to_length(self, container_size: f64) -> Length {
        match self {
            LengthOrPercentage::Length(length) => length,
            LengthOrPercentage::Percentage(percent) => Length::Dip(percent * container_size),
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

/// Point-to-DIP conversion factor.
///
/// # Examples
///
/// ```rust
/// let size_in_points = 12.0;
/// let size_in_dips = size_in_points * PT_TO_DIP;
/// ```
pub const PT_TO_DIP: f64 = 4.0 / 3.0;

/// Inches-to-DIP conversion factor.
///
/// # Examples
///
/// ```rust
/// let size_in_inches = 2.5;
/// let size_in_dips = size_in_inches * IN_TO_DIP;
/// ```
pub const IN_TO_DIP: f64 = 96.0;

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
    /// Converts the specified value from degrees to radians. (i.e. `45.degrees()` will return `PI/4`).
    fn degrees(self) -> f64;
}

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
    fn degrees(self) -> f64 {
        self.to_radians() as f64
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
    fn degrees(self) -> f64 {
        self.to_radians()
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
    fn degrees(self) -> f64 {
        (self as f64).to_radians()
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
    fn degrees(self) -> f64 {
        (self as f64).to_radians()
    }
}
