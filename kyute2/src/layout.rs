//! Types and functions used for layouting widgets.
use crate::Size;
use kurbo::{Insets, Vec2};
use std::{
    fmt,
    hash::{Hash, Hasher},
};

////////////////////////////////////////////////////////////////////////////////////////////////////
// LayoutConstraints
////////////////////////////////////////////////////////////////////////////////////////////////////

/// Layout constraints passed down to child widgets
#[derive(Copy, Clone)]
pub struct LayoutParams {
    /// TODO
    pub widget_state: (),
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
            widget_state: (),
            scale_factor: 1.0,
            min: Size::ZERO,
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

/*impl Data for LayoutParams {
    fn same(&self, other: &Self) -> bool {
        self == other
    }
}*/

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
    pub fn deflate(&self, insets: Insets) -> LayoutParams {
        LayoutParams {
            max: Size {
                width: (self.max.width - insets.x_value()).max(self.min.width),
                height: (self.max.height - insets.y_value()).max(self.min.height),
            },
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

/// Describes how some content should be positioned inside a containing block.
#[derive(Copy, Clone, PartialEq)]
pub struct Geometry {
    /// X-axis alignment.
    pub x_align: Alignment,
    /// Y-axis alignment.
    pub y_align: Alignment,
    /// Padding
    pub padding: Insets,
    /// Content size.
    pub content_size: Size,
    /// Content baseline.
    pub content_baseline: Option<f64>,
    // TODO maybe layout should also contain shape information? This is useful for e.g. borders, which need
    // the border radii. Also this way we'd be able to accumulate borders.
}

impl fmt::Debug for Geometry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // [ width x height, baseline:{}, padding=(t r b l), align=(x, y) ]

        write!(f, "[")?;
        write!(f, "{:?}", self.content_size)?;

        if let Some(baseline) = self.content_baseline {
            write!(f, ", baseline={}", baseline)?;
        }
        if self.padding.x0 != 0.0 || self.padding.x1 != 0.0 || self.padding.y0 != 0.0 || self.padding.y1 != 0.0 {
            write!(
                f,
                ", padding=({} {} {} {})",
                self.padding.x0, self.padding.y0, self.padding.x1, self.padding.y1,
            )?;
        }
        write!(f, ", align=({:?} {:?})", self.x_align, self.y_align)?;
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
    /// Zero-sized geometry with no padding and no baseline.
    pub const ZERO: Geometry = Geometry::new(Size::ZERO);

    pub const fn new(content_size: Size) -> Geometry {
        Geometry {
            x_align: Alignment::START,
            y_align: Alignment::START,
            padding: Insets::ZERO,
            content_size,
            content_baseline: None,
        }
    }

    /// Returns the size of the padding box.
    ///
    /// The padding box is the content box inflated by the padding.   
    pub fn padding_box_size(&self) -> Size {
        (self.content_size.to_rect() + self.padding).size()
    }

    /// Baseline from the top of the padding box.
    pub fn padding_box_baseline(&self) -> Option<f64> {
        self.content_baseline.map(|y| y + self.padding.y0)
    }

    /// Places the content inside a containing box with the given measurements.
    ///
    /// If this box' vertical alignment is `FirstBaseline` or `LastBaseline`,
    /// it will be aligned to the baseline of the containing box.
    ///
    /// Returns the offset of the content box.
    pub fn place_into(&self, container_size: Size, container_baseline: Option<f64>) -> Vec2 {
        let pad = self.padding;
        //let bounds = container_size.to_rect() - pad;

        let x = match self.x_align {
            Alignment::Relative(x) => pad.x0 + x * (container_size.width - pad.x0 - pad.x1 - self.content_size.width),
            // TODO vertical baseline alignment
            _ => 0.0,
        };
        let y = match self.y_align {
            Alignment::Relative(x) => pad.y0 + x * (container_size.height - pad.y0 - pad.y1 - self.content_size.height),
            Alignment::FirstBaseline => {
                // align this box baseline to the containing box baseline
                let mut y = match (container_baseline, self.content_baseline) {
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
}

impl Default for Geometry {
    fn default() -> Self {
        Geometry::ZERO
    }
}
