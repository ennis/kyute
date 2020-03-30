//! Types and functions used for layouting widgets.

pub type Size = euclid::default::Size2D<f64>;
pub type Bounds = euclid::default::Rect<f64>;
pub type Offset = euclid::default::Vector2D<f64>;
pub type Point = euclid::default::Point2D<f64>;

/// Edge insets.
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub struct EdgeInsets {
    pub left: f64,
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
}

impl From<f64> for EdgeInsets {
    fn from(v: f64) -> Self {
        EdgeInsets::all(v)
    }
}

impl EdgeInsets {
    pub fn all(v: f64) -> EdgeInsets {
        EdgeInsets {
            left: v,
            top: v,
            right: v,
            bottom: v,
        }
    }
}

/// Box constraints.
#[derive(Copy, Clone, Debug)]
pub struct BoxConstraints {
    pub min: Size,
    pub max: Size,
}

impl BoxConstraints {
    pub fn new(min: Size, max: Size) -> BoxConstraints {
        BoxConstraints { min, max }
    }

    pub fn loose(size: Size) -> BoxConstraints {
        BoxConstraints {
            min: Size::new(0.0, 0.0),
            max: size,
        }
    }

    pub fn tight(size: Size) -> BoxConstraints {
        BoxConstraints {
            min: size,
            max: size,
        }
    }

    pub fn deflate(&self, insets: &EdgeInsets) -> BoxConstraints {
        let max_w = self.max.width - (insets.left + insets.right);
        let max_h = self.max.height - (insets.top + insets.bottom);

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
        Size::new(
            self.constrain_width(size.width),
            self.constrain_height(size.height),
        )
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

pub fn align_box(alignment: Alignment, parent_size: Size, child_size: Size) -> Offset {
    let parent_pos = Point::new(
        0.5 * parent_size.width * (1.0 + alignment.x),
        0.5 * parent_size.height * (1.0 + alignment.y),
    );
    let child_pos = Point::new(
        0.5 * child_size.width * (1.0 + alignment.x),
        0.5 * child_size.height * (1.0 + alignment.y),
    );
    let offset = parent_pos - child_pos;
    offset
}

/// Layout information for a visual node, relative to a parent node.
#[derive(Copy, Clone, Debug)]
pub struct Layout {
    /// Offset within the parent node.
    pub offset: Offset,
    /// Size of this node.
    pub size: Size,
    /// Baseline offset relative to *this* node.
    /// The baseline relative to the parent node is `offset.y + baseline`.
    pub baseline: Option<f64>,
}

impl Default for Layout {
    fn default() -> Self {
        Layout {
            offset: (0.0, 0.0).into(),
            size: (0.0, 0.0).into(),
            baseline: None,
        }
    }
}

impl Layout {
    /// Creates a new [`Layout`] with the given size, with no offset relative to its parent.
    pub fn new(size: Size) -> Layout {
        Layout {
            offset: (0.0, 0.0).into(),
            size,
            baseline: None,
        }
    }

    /// Aligns a parent node and a child node.
    pub fn align(parent: &mut Layout, child: &mut Layout, alignment: Alignment) {
        child.offset = align_box(alignment, parent.size, child.size);
        parent.baseline = child.baseline.map(|baseline| baseline + child.offset.y);
    }

    /// Replaces the baseline of this node.
    pub fn with_baseline(mut self, baseline: Option<f64>) -> Layout {
        self.baseline = baseline;
        self
    }

    /// Replaces the offset within the parent node.
    pub fn with_offset(mut self, by: Offset) -> Layout {
        self.offset = by;
        self
    }

    pub fn offset(&self) -> Offset {
        self.offset
    }

    pub fn size(&self) -> Size {
        self.size
    }

    pub fn width(&self) -> f64 {
        self.size.width
    }

    pub fn height(&self) -> f64 {
        self.size.height
    }
}

impl From<Size> for Layout {
    fn from(s: Size) -> Self {
        Layout::new(s)
    }
}

/// Layout of a node in window coordinates.
///
/// TODO: this could be replaced with just the Bounds, since the baseline
/// is not really needed during rendering, or even with a more generic `PaintCtx`.
/// -> however the paint layout should be passed down in event, in paint, in hit-test...
/// -> PaintCtx should also contain the current clip bounds
/// -> PaintCtx should have a transform stack (and a clip stack)
#[derive(Copy, Clone, Debug)]
pub struct PaintLayout {
    pub bounds: Bounds,
    pub baseline: Option<f64>,
}

impl PaintLayout {
    pub(super) fn new(origin: Point, layout: &Layout) -> Self {
        PaintLayout {
            bounds: Bounds::new(origin + layout.offset, layout.size),
            baseline: layout.baseline,
        }
    }
}


// Bikeshedding
// - Layout
// - Geometry
// - PaintBox