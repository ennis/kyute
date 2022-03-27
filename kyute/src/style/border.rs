//! Border description.
use crate::{
    drawing::ToSkia,
    style::{BlendMode, Length, Paint},
    Color, Offset, PaintCtx, Rect, RectExt,
};
use approx::ulps_eq;
use kyute_common::{SideOffsets, Size};
use skia_safe as sk;

/// Border reference position
#[derive(Copy, Clone, Debug, PartialEq, serde::Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum BorderPosition {
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

/// Describes a border around a box.
#[derive(Clone, Debug)]
pub struct Border {
    /// Left,top,right,bottom border widths.
    pub widths: [Length; 4],
    /// Position of the border relative to the bounds.
    pub position: BorderPosition,
    pub paint: Paint,
    /// Border line style.
    pub style: BorderStyle,
    pub opacity: f64,
    pub blend_mode: BlendMode,
    pub enabled: bool,
    pub offset_x: Length,
    pub offset_y: Length,
}

impl Border {
    /// Creates a new border description with the specified side widths.
    fn new(width: Length, position: BorderPosition) -> Border {
        Border {
            widths: [width; 4],
            position,
            paint: Paint::SolidColor {
                color: Color::new(0.0, 0.0, 0.0, 1.0),
            },
            style: BorderStyle::Solid,
            opacity: 1.0,
            blend_mode: BlendMode::SrcOver,
            enabled: true,
            offset_x: Default::default(),
            offset_y: Default::default(),
        }
    }

    pub fn inside(width: impl Into<Length>) -> Border {
        let width = width.into();
        Border::new(width, BorderPosition::Inside)
    }

    pub fn outside(width: impl Into<Length>) -> Border {
        let width = width.into();
        Border::new(width, BorderPosition::Outside)
    }

    pub fn center(width: impl Into<Length>) -> Border {
        let width = width.into();
        Border::new(width, BorderPosition::Center)
    }

    pub fn around(width: impl Into<Length>) -> Border {
        let width = width.into();
        Border::new(width, BorderPosition::Around)
    }

    pub fn offset_x(mut self, offset_x: impl Into<Length>) -> Self {
        self.offset_x = offset_x.into();
        self
    }

    pub fn offset_y(mut self, offset_y: impl Into<Length>) -> Self {
        self.offset_y = offset_y.into();
        self
    }

    /*pub fn move_inside(mut self, pos: impl Into<ValueRef<Length>>) -> Self {
        self.position = BorderPosition::Inside(pos.into());
        self
    }

    pub fn move_outside(mut self, pos: impl Into<ValueRef<Length>>) -> Self {
        self.position = BorderPosition::Outside(pos.into());
        self
    }

    pub fn move_center(mut self) -> Self {
        self.position = BorderPosition::Center;
        self
    }*/

    pub fn paint(mut self, paint: impl Into<Paint>) -> Self {
        self.paint = paint.into();
        self
    }

    pub fn opacity(mut self, opacity: f64) -> Self {
        self.opacity = opacity;
        self
    }

    pub fn blend(mut self, blend_mode: BlendMode) -> Self {
        self.blend_mode = blend_mode;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn side_offsets(&self, scale_factor: f64, available_space: Size) -> SideOffsets {
        match self.position {
            BorderPosition::Inside | BorderPosition::Center | BorderPosition::Outside => SideOffsets::zero(),
            BorderPosition::Around => {
                SideOffsets::new(
                    self.widths[0].to_dips(scale_factor, available_space.height), // top
                    self.widths[1].to_dips(scale_factor, available_space.width),  // right
                    self.widths[2].to_dips(scale_factor, available_space.height), // bottom
                    self.widths[3].to_dips(scale_factor, available_space.width),  // left
                )
            }
        }
    }

    pub fn get_clip_bounds_offsets(&self) -> SideOffsets {
        todo!()
    }

    /// Draws the described border in the given paint context, around the specified bounds.
    pub fn draw(&self, ctx: &mut PaintCtx, bounds: Rect, radii: [sk::Vector; 4]) {
        let offset = Offset::new(
            self.offset_x.to_dips(ctx.scale_factor, bounds.size.width),
            self.offset_y.to_dips(ctx.scale_factor, bounds.size.height),
        );
        let bounds = bounds.translate(offset);
        let mut paint = self.paint.to_sk_paint(bounds);
        paint.set_style(sk::PaintStyle::Stroke);
        paint.set_blend_mode(self.blend_mode.to_skia());
        //paint.set_alpha_f(self.opacity as sk::scalar);

        // LTRB
        let widths = [
            self.widths[0].to_dips(ctx.scale_factor, bounds.size.width),
            self.widths[1].to_dips(ctx.scale_factor, bounds.size.height),
            self.widths[2].to_dips(ctx.scale_factor, bounds.size.width),
            self.widths[3].to_dips(ctx.scale_factor, bounds.size.height),
        ];
        let uniform_border = widths.iter().all(|&w| ulps_eq!(w, widths[0]));

        let rect = match self.position {
            BorderPosition::Inside | BorderPosition::Around => {
                let mut rect = bounds;
                rect.origin.x += 0.5 * widths[0];
                rect.origin.y += 0.5 * widths[1];
                rect.size.width -= 0.5 * (widths[0] + widths[2]);
                rect.size.height -= 0.5 * (widths[1] + widths[3]);
                rect
            }
            BorderPosition::Outside => {
                let mut rect = bounds;
                rect.origin.x -= 0.5 * widths[0];
                rect.origin.y -= 0.5 * widths[1];
                rect.size.width += 0.5 * (widths[0] + widths[2]);
                rect.size.height += 0.5 * (widths[1] + widths[3]);
                rect
            }
            BorderPosition::Center => bounds,
        };

        if !uniform_border {
            // draw lines, ignore radii
            // TODO multiple border colors

            // left
            if !ulps_eq!(widths[0], 0.0) {
                paint.set_stroke_width(widths[0] as sk::scalar);
                ctx.canvas
                    .draw_line(rect.top_left().to_skia(), rect.bottom_left().to_skia(), &paint);
            }

            // top
            if !ulps_eq!(widths[1], 0.0) {
                paint.set_stroke_width(widths[1] as sk::scalar);
                ctx.canvas
                    .draw_line(rect.top_left().to_skia(), rect.top_right().to_skia(), &paint);
            }

            // right
            if !ulps_eq!(widths[2], 0.0) {
                paint.set_stroke_width(widths[2] as sk::scalar);
                ctx.canvas
                    .draw_line(rect.top_right().to_skia(), rect.bottom_right().to_skia(), &paint);
            }

            // bottom
            if !ulps_eq!(widths[3], 0.0) {
                paint.set_stroke_width(widths[3] as sk::scalar);
                ctx.canvas
                    .draw_line(rect.bottom_left().to_skia(), rect.bottom_right().to_skia(), &paint);
            }
        } else {
            paint.set_stroke_width(widths[0] as sk::scalar);
            if radii[0].is_zero() && radii[1].is_zero() && radii[2].is_zero() && radii[3].is_zero() {
                ctx.canvas.draw_rect(rect.to_skia(), &paint);
            } else {
                let rrect = sk::RRect::new_rect_radii(rect.to_skia(), &radii);
                ctx.canvas.draw_rrect(rrect, &paint);
            }
        }
    }
}
