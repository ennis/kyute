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
    pub fn new(position: BorderPosition, left: Length, top: Length, right: Length, bottom: Length) -> Border {
        Border {
            widths: [left, top, right, bottom],
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
        Border::new(BorderPosition::Inside, width, width, width, width)
    }

    pub fn outside(width: impl Into<Length>) -> Border {
        let width = width.into();
        Border::new(BorderPosition::Outside, width, width, width, width)
    }

    pub fn center(width: impl Into<Length>) -> Border {
        let width = width.into();
        Border::new(BorderPosition::Center, width, width, width, width)
    }

    pub fn around(width: impl Into<Length>) -> Border {
        let width = width.into();
        Border::new(BorderPosition::Around, width, width, width, width)
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

    pub fn get_clip_bounds_offsets(&self, scale_factor: f64, available_space: Size) -> SideOffsets {
        todo!()
    }

    /// Draws the described border in the given paint context, around the specified bounds.
    pub fn draw(&self, ctx: &mut PaintCtx, bounds: Rect, radii: [sk::Vector; 4]) {
        let rounded = !(radii[0].is_zero() && radii[1].is_zero() && radii[2].is_zero() && radii[3].is_zero());

        let offset = Offset::new(
            self.offset_x.to_dips(ctx.scale_factor, bounds.size.width),
            self.offset_y.to_dips(ctx.scale_factor, bounds.size.height),
        );

        // LTRB
        let (l, t, r, b) = (
            self.widths[0].to_dips(ctx.scale_factor, bounds.size.width),
            self.widths[1].to_dips(ctx.scale_factor, bounds.size.height),
            self.widths[2].to_dips(ctx.scale_factor, bounds.size.width),
            self.widths[3].to_dips(ctx.scale_factor, bounds.size.height),
        );
        //let uniform_border = widths.iter().all(|&w| ulps_eq!(w, widths[0]));

        let mut rrect = if rounded {
            sk::RRect::new_rect_radii(bounds.to_skia(), &radii).with_offset(offset.to_skia())
        } else {
            sk::RRect::new_rect(bounds.to_skia()).with_offset(offset.to_skia())
        };

        let canvas = ctx.surface.canvas();
        let mut paint = self.paint.to_sk_paint(bounds.translate(offset));
        paint.set_style(sk::PaintStyle::Fill);
        paint.set_blend_mode(self.blend_mode.to_skia());

        match self.position {
            BorderPosition::Inside | BorderPosition::Around => {
                let inset_x = 0.5 * (l + r);
                let offset_x = 0.5 * (l - r);
                let inset_y = 0.5 * (t + b);
                let offset_y = 0.5 * (t - b);
                let inset_rrect = rrect
                    .with_offset(Offset::new(offset_x, offset_y).to_skia())
                    .with_inset(Offset::new(inset_x, inset_y).to_skia());

                // Inside borders are clipped
                let bounds_rrect = sk::RRect::new_rect_radii(bounds.to_skia(), &radii);
                canvas.save();
                canvas.clip_rrect(bounds_rrect, sk::ClipOp::Intersect, None);
                canvas.draw_drrect(rrect, inset_rrect, &paint);
                canvas.restore();
            }
            BorderPosition::Outside => {
                let outset_x = 0.5 * (l + r);
                let offset_x = -0.5 * (l - r);
                let outset_y = 0.5 * (t + b);
                let offset_y = -0.5 * (t - b);
                let outset_rrect = rrect
                    .with_offset(Offset::new(offset_x, offset_y).to_skia())
                    .with_outset(Offset::new(outset_x, outset_y).to_skia());
                canvas.draw_drrect(outset_rrect, rrect, &paint);
            }
            // TODO
            BorderPosition::Center => {}
        };
    }
}
