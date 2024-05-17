//! Types and functions used for layouting widgets.
use crate::{LengthOrPercentage, Rect, Size};
use kurbo::{Insets, Vec2};
use std::{
    fmt,
    hash::{Hash, Hasher},
    ops::{Range, RangeBounds},
};

////////////////////////////////////////////////////////////////////////////////////////////////////
// LayoutConstraints
////////////////////////////////////////////////////////////////////////////////////////////////////

/// Layout constraints passed down to child widgets
#[derive(Copy, Clone)]
pub struct BoxConstraints {
    /// Minimum allowed size.
    pub min: Size,
    /// Maximum allowed size (can be infinite).
    pub max: Size,
}

impl Default for BoxConstraints {
    fn default() -> Self {
        BoxConstraints {
            min: Size::ZERO,
            max: Size::new(f64::INFINITY, f64::INFINITY),
        }
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
        //&& self.font_size == other.font_size
    }
}

impl Hash for BoxConstraints {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.min.width.to_bits().hash(state);
        self.min.height.to_bits().hash(state);
        self.max.width.to_bits().hash(state);
        self.max.height.to_bits().hash(state);
        //self.font_size.to_bits().hash(state);
    }
}

/*impl Data for LayoutParams {
    fn same(&self, other: &Self) -> bool {
        self == other
    }
}*/

impl fmt::Debug for BoxConstraints {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.min.width == self.max.width {
            write!(f, "[w={}, ", self.min.width)?;
        } else {
            if self.max.width.is_finite() {
                write!(f, "[{}≤w≤{}, ", self.min.width, self.max.width)?;
            } else {
                write!(f, "[{}≤w≤∞, ", self.min.width)?;
            }
        }

        if self.min.height == self.max.height {
            write!(f, "h={} ", self.min.height)?;
        } else {
            if self.max.height.is_finite() {
                write!(f, "{}≤h≤{}", self.min.height, self.max.height)?;
            } else {
                write!(f, "{}≤h≤∞", self.min.height)?;
            }
        }

        write!(f, "]")
    }
}

fn range_bounds_to_lengths(bounds: impl RangeBounds<f64>) -> (f64, f64) {
    let start = match bounds.start_bound() {
        std::ops::Bound::Included(&x) => x,
        std::ops::Bound::Excluded(&x) => x,
        std::ops::Bound::Unbounded => 0.0,
    };
    let end = match bounds.end_bound() {
        std::ops::Bound::Included(&x) => x,
        std::ops::Bound::Excluded(&x) => x,
        std::ops::Bound::Unbounded => f64::INFINITY,
    };
    (start, end)
}

impl BoxConstraints {
    pub fn deflate(&self, insets: Insets) -> BoxConstraints {
        BoxConstraints {
            max: Size {
                width: (self.max.width - insets.x_value()).max(self.min.width),
                height: (self.max.height - insets.y_value()).max(self.min.height),
            },
            ..*self
        }
    }

    pub fn loose(size: Size) -> BoxConstraints {
        BoxConstraints {
            min: Size::ZERO,
            max: size,
        }
    }

    pub fn loosen(&self) -> BoxConstraints {
        BoxConstraints {
            min: Size::ZERO,
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

    fn compute_length(&self, length: LengthOrPercentage, max_length: f64) -> f64 {
        match length {
            LengthOrPercentage::Px(px) => px,
            LengthOrPercentage::Percentage(x) => x * max_length,
        }
    }

    pub fn compute_width(&self, width: LengthOrPercentage) -> f64 {
        self.compute_length(width, self.max.width)
    }

    pub fn compute_height(&self, height: LengthOrPercentage) -> f64 {
        self.compute_length(height, self.max.height)
    }

    pub fn set_width_range(&mut self, width: impl RangeBounds<f64>) {
        let (min, max) = range_bounds_to_lengths(width);
        self.min.width = min;
        self.max.width = max;
    }

    pub fn set_height_range(&mut self, height: impl RangeBounds<f64>) {
        let (min, max) = range_bounds_to_lengths(height);
        self.min.height = min;
        self.max.height = max;
    }

    pub fn width_range(&self) -> Range<f64> {
        self.min.width..self.max.width
    }

    pub fn height_range(&self) -> Range<f64> {
        self.min.height..self.max.height
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Alignment
////////////////////////////////////////////////////////////////////////////////////////////////////

// TODO Alignment is complicated, and what is meant varies under the context:
// - "left" or "right" is valid only when not talking about text.
// - otherwise, it's "trailing" and "leading", which takes into account the current text direction

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

////////////////////////////////////////////////////////////////////////////////////////////////////
// Geometry
////////////////////////////////////////////////////////////////////////////////////////////////////

/// Describes the size of an element and how it should be positioned inside a containing block.
#[derive(Copy, Clone, PartialEq)]
pub struct Geometry {
    /// Element size.
    ///
    /// Note that descendants can overflow and fall outside of the bounds defined by `size`.
    /// Use `bounding_rect` to get the size of the element and its descendants combined.
    pub size: Size,

    /// Element baseline.
    pub baseline: Option<f64>,

    /// Bounding box of the content and its descendants. This includes the union of the bounding rectangles of all descendants, if the element allows overflowing content.
    pub bounding_rect: Rect,

    /// Paint bounds.
    ///
    /// This is the region that is dirtied when the content and its descendants needs to be repainted.
    /// It can be different from `bounding_rect` if the element has drawing effects that bleed outside of the bounds used for hit-testing (e.g. drop shadows).
    pub paint_bounding_rect: Rect,
}

impl fmt::Debug for Geometry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // [ width x height, baseline:{}, padding=(t r b l), align=(x, y) ]

        write!(f, "[")?;
        write!(f, "{:?}", self.size)?;

        if let Some(baseline) = self.baseline {
            write!(f, ", baseline={}", baseline)?;
        }
        /*if self.padding.x0 != 0.0 || self.padding.x1 != 0.0 || self.padding.y0 != 0.0 || self.padding.y1 != 0.0 {
            write!(
                f,
                ", padding=({} {} {} {})",
                self.padding.x0, self.padding.y0, self.padding.x1, self.padding.y1,
            )?;
        }*/
        //write!(f, ", align=({:?} {:?})", self.x_align, self.y_align)?;
        write!(f, ", bounds={}", self.bounding_rect)?;
        write!(f, ", paint_bounds={}", self.paint_bounding_rect)?;
        write!(f, "]")?;
        Ok(())
    }
}

impl From<Size> for Geometry {
    fn from(value: Size) -> Self {
        Geometry::new(value)
    }
}

impl Geometry {
    /// Zero-sized geometry with no baseline.
    pub const ZERO: Geometry = Geometry::new(Size::ZERO);

    pub const fn new(size: Size) -> Geometry {
        Geometry {
            size,
            baseline: None,
            bounding_rect: Rect {
                x0: 0.0,
                y0: 0.0,
                x1: size.width,
                y1: size.height,
            },
            paint_bounding_rect: Rect {
                x0: 0.0,
                y0: 0.0,
                x1: size.width,
                y1: size.height,
            },
        }
    }

    /*/// Returns the size of the padding box.
    ///
    /// The padding box is the element box inflated by the padding.
    pub fn padding_box_size(&self) -> Size {
        (self.size.to_rect() + self.padding).size()
    }

    /// Baseline from the top of the padding box.
    pub fn padding_box_baseline(&self) -> Option<f64> {
        self.baseline.map(|y| y + self.padding.y0)
    }*/

    /*/// Places the content inside a containing box with the given measurements.
    ///
    /// If this box' vertical alignment is `FirstBaseline` or `LastBaseline`,
    /// it will be aligned to the baseline of the containing box.
    ///
    /// Returns the offset of the element box.
    pub fn place_into(&self, container_size: Size, container_baseline: Option<f64>) -> Vec2 {
        let pad = self.padding;
        //let bounds = container_size.to_rect() - pad;

        let x = match self.x_align {
            Alignment::Relative(x) => pad.x0 + x * (container_size.width - pad.x0 - pad.x1 - self.size.width),
            // TODO vertical baseline alignment
            _ => 0.0,
        };
        let y = match self.y_align {
            Alignment::Relative(x) => pad.y0 + x * (container_size.height - pad.y0 - pad.y1 - self.size.height),
            Alignment::FirstBaseline => {
                // align this box baseline to the containing box baseline
                let mut y = match (container_baseline, self.baseline) {
                    (Some(container_baseline), Some(content_baseline)) => {
                        // containing-box-baseline == y-offset + self-baseline
                        container_baseline - content_baseline
                    }
                    _ => {
                        // the containing box or this box have no baseline
                        0.0
                    }
                };

                // ensure sufficient padding, even if this means breaking the baseline alignment
                if y < pad.y0 {
                    y = pad.y0;
                }
                y
            }
            // TODO last baseline alignment
            _ => 0.0,
        };

        Vec2::new(x, y)
    }*/
}

impl Default for Geometry {
    fn default() -> Self {
        Geometry::ZERO
    }
}

/// Places the content inside a containing box with the given measurements.
///
/// If this box' vertical alignment is `FirstBaseline` or `LastBaseline`,
/// it will be aligned to the baseline of the containing box.
///
/// Returns the offset of the element box.
pub fn place_into(
    size: Size,
    baseline: Option<f64>,
    container_size: Size,
    container_baseline: Option<f64>,
    x_align: Alignment,
    y_align: Alignment,
    pad: &Insets,
) -> Vec2 {
    let x = match x_align {
        Alignment::Relative(x) => pad.x0 + x * (container_size.width - pad.x0 - pad.x1 - size.width),
        // TODO vertical baseline alignment
        _ => 0.0,
    };
    let y = match y_align {
        Alignment::Relative(x) => pad.y0 + x * (container_size.height - pad.y0 - pad.y1 - size.height),
        Alignment::FirstBaseline => {
            // align this box baseline to the containing box baseline
            let mut y = match (container_baseline, baseline) {
                (Some(container_baseline), Some(content_baseline)) => {
                    // containing-box-baseline == y-offset + self-baseline
                    container_baseline - content_baseline
                }
                _ => {
                    // the containing box or this box have no baseline
                    0.0
                }
            };

            // ensure sufficient padding, even if this means breaking the baseline alignment
            if y < pad.y0 {
                y = pad.y0;
            }
            y
        }
        // TODO last baseline alignment
        _ => 0.0,
    };

    Vec2::new(x, y)
}
