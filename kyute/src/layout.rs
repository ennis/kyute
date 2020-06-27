//! Types and functions used for layouting widgets.

use crate::application::AppCtx;
use crate::node::NodeArena;
use crate::widget::Baseline;
use std::fmt::Formatter;
use std::ops::RangeBounds;
use generational_indextree::NodeId;
use kyute_shell::platform::Platform;
use std::fmt;
use std::ops::Bound;
use std::rc::Rc;
use crate::{Size, SideOffsets, Point, Offset};

/// Box constraints.
#[derive(Copy, Clone)]
pub struct BoxConstraints {
    pub min: Size,
    pub max: Size,
}

impl BoxConstraints {
    pub fn new(width: impl RangeBounds<f64>, height: impl RangeBounds<f64>) -> BoxConstraints {
        let min_width = match width.start_bound() {
            Bound::Unbounded => 0.0,
            Bound::Excluded(&x) => x,
            Bound::Included(&x) => x,
        };
        let max_width = match width.end_bound() {
            Bound::Unbounded => std::f64::INFINITY,
            Bound::Excluded(&x) => x,
            Bound::Included(&x) => x,
        };
        let min_height = match height.start_bound() {
            Bound::Unbounded => 0.0,
            Bound::Excluded(&x) => x,
            Bound::Included(&x) => x,
        };
        let max_height = match height.end_bound() {
            Bound::Unbounded => std::f64::INFINITY,
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

    pub fn tight(size: Size) -> BoxConstraints {
        BoxConstraints {
            min: size,
            max: size,
        }
    }

    pub fn enforce(&self, other: &BoxConstraints) -> BoxConstraints {
        BoxConstraints {
            min: self.min.max(other.min),
            max: self.max.min(other.max),
        }
    }

    pub fn deflate(&self, insets: &SideOffsets) -> BoxConstraints {
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

impl fmt::Debug for BoxConstraints {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{} => {}]", self.min, self.max)
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
        0.5 * parent.size.width * (1.0 + alignment.x),
        0.5 * parent.size.height * (1.0 + alignment.y),
    );
    let child_pos = Point::new(
        0.5 * child.size.width * (1.0 + alignment.x),
        0.5 * child.size.height * (1.0 + alignment.y),
    );
    let offset = parent_pos - child_pos;
    parent.baseline = child.baseline.map(|baseline| baseline + offset.y);
    offset
}

/// Layout information for a visual node, relative to a parent node.
#[derive(Copy, Clone, Debug)]
pub struct Measurements {
    /// Size of this node.
    pub size: Size,
    /// Baseline offset relative to *this* node.
    /// The baseline relative to the parent node is `offset.y + baseline`.
    pub baseline: Option<f64>,
}

impl Default for Measurements {
    fn default() -> Self {
        Measurements {
            size: (0.0, 0.0).into(),
            baseline: None,
        }
    }
}

impl Measurements {
    /// Creates a new [`Layout`] with the given size, with no offset relative to its parent.
    pub fn new(size: Size) -> Measurements {
        Measurements {
            size,
            baseline: None,
        }
    }

    /// Replaces the baseline of this node.
    pub fn with_baseline(mut self, baseline: Option<f64>) -> Measurements {
        self.baseline = baseline;
        self
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

impl From<Size> for Measurements {
    fn from(s: Size) -> Self {
        Measurements::new(s)
    }
}
