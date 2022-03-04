//! Types and functions used for layouting widgets.
use crate::{Data, Offset, Point, Rect, SideOffsets, Size};
use std::{
    fmt,
    hash::{Hash, Hasher},
    ops::{Bound, RangeBounds},
};

/// Box constraints.
#[derive(Copy, Clone, PartialEq)]
pub struct BoxConstraints {
    pub min: Size,
    pub max: Size,
}

impl Data for BoxConstraints {
    fn same(&self, other: &Self) -> bool {
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

    /*pub fn new(min: Size, max: Size) -> BoxConstraints {
        BoxConstraints { min, max }
    }*/

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

    /*/// Returns the .
    pub fn tight_or(&self, default: Size) -> Size {
        Size::new(
            if self.has_tight_width() { self.max.width } else { default.width },
            if self.has_tight_height() { self.max.height } else { default.height },
        )
    }

    pub fn has_bounded_width(&self) -> bool {
        self.max.width.is_finite()
    }

    pub fn has_bounded_height(&self) -> bool {
        self.max.height.is_finite()
    }*/

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

    pub fn max_height(&self) -> f64 {
        self.max.height
    }
}

impl fmt::Debug for BoxConstraints {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{:?} => {:?}]", self.min, self.max)
    }
}

/// Alignment.
#[derive(Copy, Clone, Debug, PartialEq)]
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

/// Layout information for a visual node, relative to a parent node.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Measurements {
    /// Bounds of this node relative to the parent node origin.
    /// TODO replace with size+anchor point? might be more intuitive
    pub bounds: Rect,
    /// Baseline offset relative to *this* node.
    /// The baseline relative to the parent node is `offset.y + baseline`.
    pub baseline: Option<f64>,
}

impl Hash for Measurements {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.bounds.origin.x.to_bits().hash(state);
        self.bounds.origin.y.to_bits().hash(state);
        self.bounds.size.width.to_bits().hash(state);
        self.bounds.size.height.to_bits().hash(state);
        self.baseline.map(|x| x.to_bits()).hash(state);
    }
}

impl Default for Measurements {
    fn default() -> Self {
        Measurements {
            bounds: Rect::zero(),
            baseline: None,
        }
    }
}

impl Measurements {
    /// Creates a new [`Layout`] with the given size, with no offset relative to its parent.
    pub fn new(bounds: Rect) -> Measurements {
        Measurements { bounds, baseline: None }
    }

    /// Replaces the baseline of this node.
    pub fn with_baseline(mut self, baseline: Option<f64>) -> Measurements {
        self.baseline = baseline;
        self
    }

    pub fn size(&self) -> Size {
        self.bounds.size
    }

    pub fn width(&self) -> f64 {
        self.bounds.size.width
    }

    pub fn height(&self) -> f64 {
        self.bounds.size.height
    }

    pub fn constrain(&self, constraints: BoxConstraints) -> Measurements {
        let mut m = self.clone();
        m.bounds.size = constraints.constrain(m.bounds.size);
        m
    }
}

impl From<Rect> for Measurements {
    fn from(bounds: Rect) -> Self {
        Measurements::new(bounds)
    }
}

/*
#[derive(Clone)]
struct LayoutItemImpl {
    measurements: Measurements,
    children: Vec<(Offset, LayoutItem)>,
}

/// Represents the visual layout of a widget subtree.
#[derive(Clone)]
pub struct LayoutItem(Arc<LayoutItemImpl>);

impl fmt::Debug for LayoutItem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "{:?}", self.0.measurements)
    }
}

impl LayoutItem {
    pub fn new(measurements: Measurements) -> LayoutItem {
        LayoutItem(Arc::new(LayoutItemImpl {
            measurements,
            children: vec![],
        }))
    }

    pub fn with_children(
        measurements: Measurements,
        children: Vec<(Offset, LayoutItem)>,
    ) -> LayoutItem {
        LayoutItem(Arc::new(LayoutItemImpl {
            measurements,
            children,
        }))
    }

    pub fn add_child(&mut self, offset: Offset, item: LayoutItem) {
        Arc::make_mut(&mut self.0).children.push((offset, item));
    }

    pub fn size(&self) -> Size {
        self.0.measurements.size()
    }

    pub fn measurements(&self) -> Measurements {
        self.0.measurements
    }

    pub fn baseline(&self) -> Option<f64> {
        self.0.measurements.baseline
    }

    pub fn bounds(&self) -> Rect {
        Rect::new(Point::origin(), self.0.measurements.size())
    }

    pub fn children(&self) -> &[(Offset, LayoutItem)] {
        &self.0.children
    }

    pub fn child(&self, at: usize) -> Option<LayoutItem> {
        self.0
            .children
            .get(at)
            .map(|(_offset, layout)| layout.clone())
    }
}
*/
