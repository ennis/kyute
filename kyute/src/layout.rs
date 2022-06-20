//! Types and functions used for layouting widgets.
use crate::{Data, Offset, Point, Rect, SideOffsets, Size};
use kyute_common::Angle;
use std::{
    fmt,
    hash::{Hash, Hasher},
    ops::{Bound, Mul, Neg, RangeBounds},
};

////////////////////////////////////////////////////////////////////////////////////////////////////
// Length
////////////////////////////////////////////////////////////////////////////////////////////////////

/// Context used to compute the final value of a Length.
pub struct LengthCtx {
    /// Font size of the current element in DIPs.
    pub font_size: f64,

    /// Size in DIPs of the containing block size (width or height, depending on the value being resolved).
    ///
    /// Used to resolve percentage lengths.
    pub containing_block_size: f64,

    /// Target scale factor.
    pub scale_factor: f64,
}

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
    /// Length relative to the parent element.
    Proportional(f64),
}

impl fmt::Debug for Length {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Length::Px(v) | Length::Dip(v) | Length::Proportional(v) if v == 0.0 => {
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
            Length::Proportional(v) => {
                write!(f, "{}%", v * 100.0)
            }
        }
    }
}

impl Length {
    /// Scale the length by the given amount.
    pub fn scale(self, by: f64) -> Self {
        let mut v = self;
        match v {
            Length::Px(ref mut v)
            | Length::Dip(ref mut v)
            | Length::Em(ref mut v)
            | Length::Proportional(ref mut v) => {
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
    pub fn to_dips(self, ctx: &LengthCtx) -> f64 {
        match self {
            Length::Px(x) => x / ctx.scale_factor,
            Length::Dip(x) => x,
            Length::Em(x) => x * ctx.font_size,
            Length::Proportional(x) => x * ctx.containing_block_size,
        }
    }
}

impl Neg for Length {
    type Output = Length;

    fn neg(self) -> Self::Output {
        match self {
            Length::Px(v) => Length::Px(-v),
            Length::Dip(v) => Length::Dip(-v),
            Length::Proportional(v) => Length::Proportional(-v),
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
    fn percent(self) -> Length;
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
    fn percent(self) -> Length {
        Length::Proportional(self as f64 / 100.0)
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
    fn percent(self) -> Length {
        Length::Proportional(self / 100.0)
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
    fn percent(self) -> Length {
        Length::Proportional(self as f64 / 100.0)
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
    fn percent(self) -> Length {
        Length::Proportional(self as f64 / 100.0)
    }
    fn degrees(self) -> Angle {
        Angle::degrees(self as f64)
    }
    fn radians(self) -> Angle {
        Angle::radians(self as f64)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// LayoutConstraints
////////////////////////////////////////////////////////////////////////////////////////////////////

/// Layout constraints passed down to child widgets
#[derive(Copy, Clone, Debug)]
pub struct LayoutConstraints {
    /// Parent font size.
    pub parent_font_size: f64,
    /// Scale factor.
    pub scale_factor: f64,
    /// Minimum allowed size.
    pub min: Size,
    /// Maximum allowed size (can be infinite).
    pub max: Size,
}

// required because we also have a custom hash impl
// (https://rust-lang.github.io/rust-clippy/master/index.html#derive_hash_xor_eq)
impl PartialEq for LayoutConstraints {
    fn eq(&self, other: &Self) -> bool {
        self.min.width.to_bits() == other.min.width.to_bits()
            && self.min.height.to_bits() == other.min.height.to_bits()
            && self.max.width.to_bits() == other.max.width.to_bits()
            && self.max.height.to_bits() == other.max.height.to_bits()
            && self.scale_factor.to_bits() == other.scale_factor.to_bits()
            && self.parent_font_size.to_bits() == other.parent_font_size.to_bits()
    }
}

impl Hash for LayoutConstraints {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.parent_font_size.to_bits().hash(state);
        self.scale_factor.to_bits().hash(state);
        self.min.width.to_bits().hash(state);
        self.min.height.to_bits().hash(state);
        self.max.width.to_bits().hash(state);
        self.max.height.to_bits().hash(state);
    }
}

impl Data for LayoutConstraints {
    fn same(&self, other: &Self) -> bool {
        self == other
    }
}

impl LayoutConstraints {
    pub fn deflate(&self, insets: SideOffsets) -> LayoutConstraints {
        let max_w = self.max.width - insets.horizontal();
        let max_h = self.max.height - insets.vertical();

        LayoutConstraints {
            max: Size::new(max_w, max_h).max(self.min),
            ..*self
        }
    }

    pub fn finite_max_width(&self) -> Option<f64> {
        if self.max.width.is_finite() {
            Some(self.max.width)
        } else {
            None
        }
    }

    pub fn finite_max_height(&self) -> Option<f64> {
        if self.max.height.is_finite() {
            Some(self.max.height)
        } else {
            None
        }
    }

    fn resolve_length(&self, length: Length, max_length: f64) -> f64 {
        match length {
            Length::Px(px) => px / self.scale_factor,
            Length::Dip(dip) => dip,
            Length::Em(em) => em * self.parent_font_size,
            Length::Proportional(x) => x * max_length,
        }
    }

    pub fn resolve_width(&self, width: Length) -> f64 {
        self.resolve_length(width, self.max.width)
    }

    pub fn resolve_height(&self, height: Length) -> f64 {
        self.resolve_length(height, self.max.height)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// BoxConstraints
////////////////////////////////////////////////////////////////////////////////////////////////////

/// Box constraints.
#[derive(Copy, Clone)]
pub struct BoxConstraints {
    pub min: Size,
    pub max: Size,
}

impl Default for BoxConstraints {
    fn default() -> Self {
        BoxConstraints {
            min: Size::zero(),
            max: Size::new(f64::INFINITY, f64::INFINITY),
        }
    }
}

impl Data for BoxConstraints {
    fn same(&self, other: &Self) -> bool {
        self == other
    }
}

// required because we also have a custom hash impl
// (https://rust-lang.github.io/rust-clippy/master/index.html#derive_hash_xor_eq)
impl PartialEq for BoxConstraints {
    fn eq(&self, other: &Self) -> bool {
        self.min.width.to_bits() == other.min.width.to_bits()
            && self.min.height.to_bits() == other.min.height.to_bits()
            && self.max.width.to_bits() == other.max.width.to_bits()
            && self.max.height.to_bits() == other.max.height.to_bits()
    }
}

impl Hash for BoxConstraints {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.min.width.to_bits().hash(state);
        self.min.height.to_bits().hash(state);
        self.max.width.to_bits().hash(state);
        self.max.height.to_bits().hash(state);
    }
}

impl BoxConstraints {
    pub fn new(width: impl RangeBounds<f64>, height: impl RangeBounds<f64>) -> BoxConstraints {
        let min_width = match width.start_bound() {
            Bound::Unbounded => 0.0,
            Bound::Excluded(&x) => x,
            Bound::Included(&x) => x,
        };
        let max_width = match width.end_bound() {
            Bound::Unbounded => f64::INFINITY,
            Bound::Excluded(&x) => x,
            Bound::Included(&x) => x,
        };
        let min_height = match height.start_bound() {
            Bound::Unbounded => 0.0,
            Bound::Excluded(&x) => x,
            Bound::Included(&x) => x,
        };
        let max_height = match height.end_bound() {
            Bound::Unbounded => f64::INFINITY,
            Bound::Excluded(&x) => x,
            Bound::Included(&x) => x,
        };
        BoxConstraints {
            min: Size::new(min_width, min_height),
            max: Size::new(max_width, max_height),
        }
    }

    pub fn loose(size: Size) -> BoxConstraints {
        BoxConstraints {
            min: Size::zero(),
            max: size,
        }
    }

    pub fn loosen(&self) -> BoxConstraints {
        BoxConstraints {
            min: Size::zero(),
            max: self.max,
        }
    }

    pub fn tighten(&self) -> BoxConstraints {
        let w = if self.max.width.is_finite() {
            self.max.width
        } else {
            self.min.width
        };
        let h = if self.max.height.is_finite() {
            self.max.height
        } else {
            self.min.height
        };
        BoxConstraints {
            min: Size::new(w, h),
            max: self.max,
        }
    }

    pub fn tight(size: Size) -> BoxConstraints {
        BoxConstraints { min: size, max: size }
    }

    pub fn enforce(&self, other: BoxConstraints) -> BoxConstraints {
        BoxConstraints {
            min: self.min.max(other.min),
            max: self.max.min(other.max),
        }
    }

    pub fn deflate(&self, insets: SideOffsets) -> BoxConstraints {
        let max_w = self.max.width - insets.horizontal();
        let max_h = self.max.height - insets.vertical();

        BoxConstraints {
            min: self.min,
            max: Size::new(max_w, max_h).max(self.min),
        }
    }

    /// Returns the smallest size that satisfies the constraints.
    ///
    /// Equivalent to `self.min`
    pub fn smallest(&self) -> Size {
        self.min
    }

    /// Returns the biggest size that satisfies the constraints.
    ///
    /// Equivalent to `self.max`
    pub fn biggest(&self) -> Size {
        self.max
    }

    pub fn constrain(&self, size: Size) -> Size {
        Size::new(self.constrain_width(size.width), self.constrain_height(size.height))
    }

    pub fn constrain_width(&self, width: f64) -> f64 {
        width.max(self.min.width).min(self.max.width)
    }

    pub fn constrain_height(&self, height: f64) -> f64 {
        height.max(self.min.height).min(self.max.height)
    }

    pub fn max_width(&self) -> f64 {
        self.max.width
    }

    pub fn finite_max_width(&self) -> Option<f64> {
        if self.max.width.is_finite() {
            Some(self.max.width)
        } else {
            None
        }
    }

    pub fn max_height(&self) -> f64 {
        self.max.height
    }

    pub fn finite_max_height(&self) -> Option<f64> {
        if self.max.height.is_finite() {
            Some(self.max.height)
        } else {
            None
        }
    }
}

impl fmt::Debug for BoxConstraints {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{:?} => {:?}]", self.min, self.max)
    }
}

/*
/// Aligns a child box into a parent box. Returns the offset of the child into the parent,
/// and updates the baseline of the parent.
pub fn align_boxes(alignment: Alignment, parent: &mut Measurements, child: Measurements) -> Offset {
    let parent_pos = Point::new(
        0.5 * parent.width() * (1.0 + alignment.x),
        0.5 * parent.height() * (1.0 + alignment.y),
    );
    let child_pos = Point::new(
        0.5 * child.width() * (1.0 + alignment.x),
        0.5 * child.height() * (1.0 + alignment.y),
    );
    let offset = parent_pos - child_pos;
    parent.baseline = child.baseline.map(|baseline| baseline + offset.y);
    offset
}*/

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Alignment {
    Relative(f64),
    FirstBaseline,
    LastBaseline,
}

impl Alignment {
    pub const CENTER: Alignment = Alignment::Relative(0.5);
    pub const START: Alignment = Alignment::Relative(0.0);
    pub const END: Alignment = Alignment::Relative(1.0);
}

impl Default for Alignment {
    fn default() -> Self {
        Alignment::Relative(0.0)
    }
}

/// Describes a box to be positioned inside a containing block.
#[derive(Copy, Clone, Debug)]
pub struct Layout {
    pub x_align: Alignment,
    pub y_align: Alignment,
    /// Padding around the widget
    pub padding_left: f64,
    pub padding_top: f64,
    pub padding_right: f64,
    pub padding_bottom: f64,
    pub measurements: Measurements,
}

impl Layout {
    pub fn new(size: Size) -> Layout {
        Layout {
            x_align: Alignment::START,
            y_align: Alignment::START,
            padding_left: 0.0,
            padding_top: 0.0,
            padding_right: 0.0,
            padding_bottom: 0.0,
            measurements: Measurements::new(size),
        }
    }

    /// Returns the size of the content box (without any padding).
    pub fn content_box_size(&self) -> Size {
        self.measurements.size
    }

    /// Returns the size of padding box (content size inflated with padding).
    pub fn padding_box_size(&self) -> Size {
        Size::new(
            self.measurements.size.width + self.padding_right + self.padding_left,
            self.measurements.size.height + self.padding_top + self.padding_bottom,
        )
    }

    pub fn padding_box_baseline(&self) -> Option<f64> {
        self.measurements.baseline.map(|x| x + self.padding_top)
    }

    /// Places this box inside a containing block, taking into account alignment and padding.
    ///
    /// Returns the offset of the content box.
    pub fn content_box_offset(&self, containing_block_size: Size) -> Offset {
        let mut bounds = Rect::new(Point::origin(), containing_block_size);
        bounds.origin.x += self.padding_left;
        bounds.origin.y += self.padding_top;
        bounds.size.width -= self.padding_left + self.padding_right;
        bounds.size.height -= self.padding_top + self.padding_bottom;

        // align right => x = 1.0
        // content size = 40, parent size = 100, padding = 5
        // => offset should be: 100 - 5 - 40 = 55
        //
        // calculation:
        // 1.0 * (100 - 40 - 5 - 5)
        //
        // offset + x * content_size == x * parent_size
        // offset = x * (parent_size - content_size - )

        let x = match self.x_align {
            Alignment::Relative(x) => {
                self.padding_left
                    + x * (containing_block_size.width
                        - self.padding_left
                        - self.padding_right
                        - self.measurements.size.width)
            }
            _ => 0.0,
        };
        let y = match self.y_align {
            Alignment::Relative(x) => {
                self.padding_top
                    + x * (containing_block_size.height
                        - self.padding_top
                        - self.padding_bottom
                        - self.measurements.size.height)
            }
            _ => 0.0,
        };

        Offset::new(x, y)
    }
}

/// Measurements of a Widget, returned by `Widget::layout`.
#[derive(Copy, Clone, Debug)]
pub struct Measurements {
    /// Calculated size of the widget.
    ///
    /// The widget bounds are defined in the widget's local space as `Rect::new(Point::origin(), self.size)`.
    pub size: Size,
    /// Clip bounds of the widget.
    ///
    /// By default, this is set to `None`, which means that the widget shouldn't perform
    /// any additional clipping.
    pub clip_bounds: Option<Rect>,
    /// Baseline offset relative to *this* node.
    /// The baseline relative to the parent node is `offset.y + baseline`.
    pub baseline: Option<f64>,
}

// required because we also have a custom hash impl
// (https://rust-lang.github.io/rust-clippy/master/index.html#derive_hash_xor_eq)
impl PartialEq for Measurements {
    fn eq(&self, other: &Self) -> bool {
        self.size.width.to_bits() == other.size.width.to_bits()
            && self.size.height.to_bits() == other.size.height.to_bits()
            && matches!((self.clip_bounds, other.clip_bounds), (Some(a),Some(b)) if
                a.origin.x.to_bits() == b.origin.x.to_bits()
                && a.origin.y.to_bits() == b.origin.y.to_bits()
                && a.size.width.to_bits() == b.size.width.to_bits()
                && a.size.height.to_bits() == b.size.height.to_bits())
            && matches!((self.baseline, other.baseline), (Some(a), Some(b)) if a.to_bits() == b.to_bits())
    }
}

impl Data for Measurements {
    fn same(&self, other: &Self) -> bool {
        self == other
    }
}

impl Hash for Measurements {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.size.width.to_bits().hash(state);
        self.size.height.to_bits().hash(state);
        self.clip_bounds
            .map(|cb| {
                (
                    cb.origin.x.to_bits(),
                    cb.origin.y.to_bits(),
                    cb.size.width.to_bits(),
                    cb.size.height.to_bits(),
                )
            })
            .hash(state);
        self.baseline.map(|x| x.to_bits()).hash(state);
    }
}

impl Default for Measurements {
    /// Returns zero-sized measurements.
    fn default() -> Self {
        Measurements {
            size: Size::zero(),
            clip_bounds: None,
            baseline: None,
        }
    }
}

impl Measurements {
    /// Creates new `Measurements` representing a widget with the given size, and no baseline specified.
    ///
    /// The clip bounds are are equal to the widget bounds.
    pub fn new(size: Size) -> Measurements {
        let mut m = Measurements::default();
        m.size = size;
        m
    }

    /// Creates new `Measurements` representing a widget with the given size, and the specified baseline.
    pub fn with_baseline(size: Size, baseline: f64) -> Measurements {
        Measurements {
            size,
            clip_bounds: None,
            baseline: Some(baseline),
        }
    }

    /// Returns the bounding rectangle of the widget in its local space.
    ///
    /// The rectangle's upper-left corner is at the origin (0,0), and its size is `self.size`.
    pub fn local_bounds(&self) -> Rect {
        Rect::new(Point::origin(), self.size)
    }

    /// Returns the layout width of the widget.
    pub fn width(&self) -> f64 {
        self.size.width
    }

    /// Returns the layout height of the widget.
    pub fn height(&self) -> f64 {
        self.size.height
    }

    /// Returns a copy of these measurements, adjusted so that it satisfies the
    /// given [`BoxConstraints`].
    ///
    /// FIXME/TODO? The clip bounds are left unchanged.
    pub fn constrain(&self, constraints: BoxConstraints) -> Measurements {
        let mut m = *self;
        m.size = constraints.constrain(m.size);
        m
    }
}

impl From<Size> for Measurements {
    /// Creates measurements from a size. See [`Measurements::new`].
    fn from(s: Size) -> Self {
        Measurements::new(s.into())
    }
}
