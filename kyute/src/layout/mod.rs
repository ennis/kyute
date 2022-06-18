//! Types and functions used for layouting widgets.
pub mod grid;

use crate::{style, Data, Offset, Point, Rect, SideOffsets, Size};
use std::{
    fmt,
    fmt::Formatter,
    hash::{Hash, Hasher},
    ops::{Bound, RangeBounds},
};

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

/// Alignment.
#[derive(Copy, Clone, PartialEq)]
pub struct Alignment {
    pub x: f64,
    pub y: f64,
}

impl Alignment {
    pub const TOP_LEFT: Alignment = Alignment { x: -1.0, y: -1.0 };
    pub const TOP_RIGHT: Alignment = Alignment { x: 1.0, y: -1.0 };
    pub const BOTTOM_LEFT: Alignment = Alignment { x: -1.0, y: 1.0 };
    pub const BOTTOM_RIGHT: Alignment = Alignment { x: 1.0, y: 1.0 };
    pub const CENTER_LEFT: Alignment = Alignment { x: -1.0, y: 0.0 };
    pub const CENTER_RIGHT: Alignment = Alignment { x: 1.0, y: 0.0 };
    pub const TOP_CENTER: Alignment = Alignment { x: 0.0, y: -1.0 };
    pub const BOTTOM_CENTER: Alignment = Alignment { x: 0.0, y: 1.0 };
    pub const CENTER: Alignment = Alignment { x: 0.0, y: 0.0 };
}

impl fmt::Debug for Alignment {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({},{})", self.x, self.y)
    }
}

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

pub enum Alignment2 {
    Value(f64),
    FirstBaseline,
    LastBaseline,
    // TODO: last baseline
}

/// Layout (size & positioning) information returned by a child widget.
///
/// See [`Widget::layout`].
#[derive(Clone, Debug, PartialEq)]
pub struct Layout {
    /// Size of the widget.
    pub size: Size,
    /// Clip bounds.
    pub clip: Rect,
    pub baseline: f64,

    /// Position of the widget in the parent grid, if the parent is a grid.
    pub grid_area: style::values::grid::Area,

    /// Alignment of the widget in the parent space along the inline axis (the text direction).
    pub justify: Alignment2,

    /// Alignment of the widget in the parent space along the block axis (perpendicular to the text direction).
    pub align: Alignment2,
}
