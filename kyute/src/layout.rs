//! Types and functions used for layouting widgets.
use crate::{style::WidgetState, Data, Offset, Point, Rect, SideOffsets, Size};
use std::{
    fmt,
    fmt::{Debug, Formatter},
    hash::{Hash, Hasher},
    ops::{Bound, RangeBounds},
};

////////////////////////////////////////////////////////////////////////////////////////////////////
// LayoutConstraints
////////////////////////////////////////////////////////////////////////////////////////////////////

/// Layout constraints passed down to child widgets
#[derive(Copy, Clone)]
pub struct LayoutParams {
    pub widget_state: WidgetState,
    /// Scale factor.
    pub scale_factor: f64,
    /// Minimum allowed size.
    pub min: Size,
    /// Maximum allowed size (can be infinite).
    pub max: Size,
}

impl Default for LayoutParams {
    fn default() -> Self {
        LayoutParams {
            widget_state: WidgetState::default(),
            scale_factor: 1.0,
            min: Size::zero(),
            max: Size::new(f64::INFINITY, f64::INFINITY),
        }
    }
}

// required because we also have a custom hash impl
// (https://rust-lang.github.io/rust-clippy/master/index.html#derive_hash_xor_eq)
impl PartialEq for LayoutParams {
    fn eq(&self, other: &Self) -> bool {
        self.min.width.to_bits() == other.min.width.to_bits()
            && self.min.height.to_bits() == other.min.height.to_bits()
            && self.max.width.to_bits() == other.max.width.to_bits()
            && self.max.height.to_bits() == other.max.height.to_bits()
            && self.scale_factor.to_bits() == other.scale_factor.to_bits()
            && self.widget_state == other.widget_state
    }
}

impl Hash for LayoutParams {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.scale_factor.to_bits().hash(state);
        self.min.width.to_bits().hash(state);
        self.min.height.to_bits().hash(state);
        self.max.width.to_bits().hash(state);
        self.max.height.to_bits().hash(state);
        self.widget_state.hash(state);
    }
}

impl Data for LayoutParams {
    fn same(&self, other: &Self) -> bool {
        self == other
    }
}

impl fmt::Debug for LayoutParams {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "[{:?} => {:?} (x{:.1}), st={:?}]",
            self.min, self.max, self.scale_factor, self.widget_state
        )
    }
}

impl LayoutParams {
    pub fn deflate(&self, insets: SideOffsets) -> LayoutParams {
        let max_w = self.max.width - insets.horizontal();
        let max_h = self.max.height - insets.vertical();

        LayoutParams {
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

    pub fn constrain(&self, size: Size) -> Size {
        Size::new(self.constrain_width(size.width), self.constrain_height(size.height))
    }

    pub fn constrain_width(&self, width: f64) -> f64 {
        width.max(self.min.width).min(self.max.width)
    }

    pub fn constrain_height(&self, height: f64) -> f64 {
        height.max(self.min.height).min(self.max.height)
    }

    /*fn resolve_length(&self, length: Length, max_length: f64) -> f64 {
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
    }*/
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
///
/// This groups the box' measurements (see `Measurements`), and how it should be placed within
/// a containing box (alignment and padding).
#[derive(Copy, Clone, PartialEq)]
pub struct BoxLayout {
    pub x_align: Alignment,
    pub y_align: Alignment,
    /// Padding around the widget
    pub padding_left: f64,
    pub padding_top: f64,
    pub padding_right: f64,
    pub padding_bottom: f64,
    pub measurements: Measurements,
    // TODO maybe layout should also contain shape information? This is useful for e.g. borders, which need
    // the border radii. Also this way we'd be able to accumulate borders.
}

impl Debug for BoxLayout {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // [ width x height, baseline:{}, padding=(t r b l), align=(x, y) ]

        write!(f, "[")?;
        write!(f, "{:?}", self.measurements.size)?;

        if let Some(baseline) = self.measurements.baseline {
            write!(f, ", baseline={}", baseline)?;
        }
        if self.padding_top != 0.0
            || self.padding_right != 0.0
            || self.padding_bottom != 0.0
            || self.padding_left != 0.0
        {
            write!(
                f,
                ", padding=({} {} {} {})",
                self.padding_top, self.padding_right, self.padding_bottom, self.padding_left
            )?;
        }
        write!(f, ", align=({:?} {:?})", self.x_align, self.y_align)?;
        write!(f, "]")?;
        Ok(())
    }
}

impl BoxLayout {
    pub fn new(size: Size) -> BoxLayout {
        BoxLayout {
            x_align: Alignment::START,
            y_align: Alignment::START,
            padding_left: 0.0,
            padding_top: 0.0,
            padding_right: 0.0,
            padding_bottom: 0.0,
            measurements: Measurements::new(size),
        }
    }

    /// Returns the size of the box.
    pub fn content_box_size(&self) -> Size {
        self.measurements.size
    }

    /// Returns the size of the padding box.
    ///
    /// The padding box is the content box inflated by the padding.   
    pub fn padding_box_size(&self) -> Size {
        Size::new(
            self.measurements.size.width + self.padding_right + self.padding_left,
            self.measurements.size.height + self.padding_top + self.padding_bottom,
        )
    }

    pub fn padding_box_baseline(&self) -> Option<f64> {
        self.measurements.baseline.map(|x| x + self.padding_top)
    }

    /// Places this box inside a containing box with the given measurements.
    ///
    /// If this box' vertical alignment is `FirstBaseline` or `LastBaseline`,
    /// it will be aligned to the baseline of the containing box.
    ///
    /// Returns the offset of the content box.
    pub fn place_into(&self, containing_box: &Measurements) -> Offset {
        let mut bounds = containing_box.local_bounds();
        bounds.origin.x += self.padding_left;
        bounds.origin.y += self.padding_top;
        bounds.size.width -= self.padding_left + self.padding_right;
        bounds.size.height -= self.padding_top + self.padding_bottom;

        let x = match self.x_align {
            Alignment::Relative(x) => {
                self.padding_left
                    + x * (containing_box.size.width
                        - self.padding_left
                        - self.padding_right
                        - self.measurements.size.width)
            }
            // TODO vertical baseline alignment
            _ => 0.0,
        };
        let y = match self.y_align {
            Alignment::Relative(x) => {
                self.padding_top
                    + x * (containing_box.size.height
                        - self.padding_top
                        - self.padding_bottom
                        - self.measurements.size.height)
            }
            Alignment::FirstBaseline => {
                // align this box baseline to the containing box baseline
                let mut y = match (containing_box.baseline, self.measurements.baseline) {
                    (Some(containing_box_baseline), Some(self_baseline)) => {
                        // containing-box-baseline == y-offset + self-baseline
                        containing_box_baseline - self_baseline
                    }
                    _ => {
                        // the containing box or this box have no baseline
                        0.0
                    }
                };

                // ensure sufficient padding, even if this means breaking the baseline alignment
                if y < self.padding_top {
                    y = self.padding_top;
                }
                y
            }
            // TODO last baseline alignment
            _ => 0.0,
        };

        let offset = Offset::new(x, y);
        //let baseline = self.measurements.baseline.map(|b| b + offset.y);
        offset
    }
}

impl Default for BoxLayout {
    fn default() -> Self {
        BoxLayout {
            x_align: Default::default(),
            y_align: Default::default(),
            padding_left: 0.0,
            padding_top: 0.0,
            padding_right: 0.0,
            padding_bottom: 0.0,
            measurements: Default::default(),
        }
    }
}

/// Measurements of a box, returned by `Widget::layout`.
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
    ///
    /// TODO rename this?
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
