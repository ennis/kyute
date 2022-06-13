//! Border description.
use crate::{
    animation::PaintCtx,
    drawing::ToSkia,
    style::{BlendMode, Length, Paint},
    Color, Offset, Rect, RectExt,
};
use approx::ulps_eq;
use kyute_common::{SideOffsets, Size};
use skia_safe as sk;

/// Border reference position
#[derive(Copy, Clone, Debug, PartialEq, serde::Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum BorderPosition {
    /// The border is positioned inside the widget bounds.
    #[serde(rename = "inside")]
    Inside,
    #[serde(rename = "center")]
    Center,
    #[serde(rename = "outside")]
    Outside,
    #[serde(rename = "around")]
    Around,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, serde::Deserialize)]
pub enum BorderStyle {
    #[serde(rename = "solid")]
    Solid,
    #[serde(rename = "dotted")]
    Dotted,
}

impl Default for BorderStyle {
    fn default() -> Self {
        BorderStyle::Solid
    }
}

/// Describes a border around a box.
#[derive(Clone, Debug)]
pub struct Border {
    /// Left,top,right,bottom border widths.
    pub widths: [Length; 4],
    pub paint: Paint,
    pub line_style: BorderStyle,
    pub blend_mode: BlendMode,
}

impl Border {
    /// Creates a new border description with the specified side widths.
    pub fn new(left: Length, top: Length, right: Length, bottom: Length) -> Border {
        Border {
            widths: [left, top, right, bottom],
            paint: Paint::SolidColor(Color::new(0.0, 0.0, 0.0, 1.0)),
            line_style: Default::default(),
            blend_mode: BlendMode::SrcOver,
        }
    }

    pub fn new_all_same(width: Length) -> Border {
        Border {
            widths: [width, width, width, width],
            paint: Paint::SolidColor(Color::new(0.0, 0.0, 0.0, 1.0)),
            line_style: Default::default(),
            blend_mode: BlendMode::SrcOver,
        }
    }

    pub fn paint(mut self, paint: impl Into<Paint>) -> Self {
        self.paint = paint.into();
        self
    }

    pub fn blend(mut self, blend_mode: BlendMode) -> Self {
        self.blend_mode = blend_mode;
        self
    }

    pub fn calculate_widths(&self, scale_factor: f64, available_space: Size) -> [f64; 4] {
        [
            self.widths[0].to_dips(scale_factor, available_space.height), // top
            self.widths[1].to_dips(scale_factor, available_space.width),  // right
            self.widths[2].to_dips(scale_factor, available_space.height), // bottom
            self.widths[3].to_dips(scale_factor, available_space.width),  // left
        ]
    }

    /// Draws the described border in the given paint context, around the specified bounds.
    pub fn draw(&self, ctx: &mut PaintCtx, bounds: Rect, radii: [sk::Vector; 4]) {
        let rounded = !(radii[0].is_zero() && radii[1].is_zero() && radii[2].is_zero() && radii[3].is_zero());
        let [t, r, b, l] = self.calculate_widths(ctx.scale_factor, bounds.size);

        let mut rrect = if rounded {
            sk::RRect::new_rect_radii(bounds.to_skia(), &radii)
        } else {
            sk::RRect::new_rect(bounds.to_skia())
        };

        let canvas = ctx.surface.canvas();
        let mut paint = self.paint.to_sk_paint(bounds);
        paint.set_style(sk::PaintStyle::Fill);
        paint.set_blend_mode(self.blend_mode.to_skia());

        let inset_x = 0.5 * (l + r);
        let offset_x = 0.5 * (l - r);
        let inset_y = 0.5 * (t + b);
        let offset_y = 0.5 * (t - b);
        let inset_rrect = rrect
            .with_offset(Offset::new(offset_x, offset_y).to_skia())
            .with_inset(Offset::new(inset_x, inset_y).to_skia());
        canvas.draw_drrect(rrect, inset_rrect, &paint);
    }
}
