//! Border description.
use crate::{
    style::{BlendMode, Length, Paint, ValueRef},
    Color, Environment, PaintCtx, Rect,
};
use approx::ulps_eq;
use kyute_shell::{
    drawing::{RectExt, ToSkia},
    skia as sk,
};

/// Border reference position
#[derive(Copy, Clone, Debug, PartialEq, serde::Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum BorderPosition {
    #[serde(rename = "inside")]
    Inside(ValueRef<Length>),
    #[serde(rename = "center")]
    Center,
    #[serde(rename = "outside")]
    Outside(ValueRef<Length>),
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, serde::Deserialize)]
pub enum BorderStyle {
    #[serde(rename = "solid")]
    Solid,
    #[serde(rename = "dotted")]
    Dotted,
}

/// Describes a border around a box.
#[derive(Clone, Debug, serde::Deserialize)]
pub struct Border {
    /// Left,top,right,bottom border widths.
    widths: [ValueRef<Length>; 4],
    /// Position of the border relative to the bounds.
    position: BorderPosition,
    paint: Paint,
    /// Border line style.
    style: BorderStyle,
    opacity: f64,
    blend_mode: BlendMode,
    enabled: bool,
}

impl Border {
    /// Creates a new border description with the specified side widths.
    fn new(width: ValueRef<Length>, position: BorderPosition) -> Border {
        let width = width.into();
        Border {
            widths: [width; 4],
            position,
            paint: Paint::SolidColor {
                color: Color::new(0.0, 0.0, 0.0, 1.0).into(),
            },
            style: BorderStyle::Solid,
            opacity: 1.0,
            blend_mode: BlendMode::SrcOver,
            enabled: true,
        }
    }

    pub fn inside(width: impl Into<ValueRef<Length>>) -> Border {
        let width = width.into();
        Border::new(width, BorderPosition::Inside(width))
    }

    pub fn outside(width: impl Into<ValueRef<Length>>) -> Border {
        let width = width.into();
        Border::new(width, BorderPosition::Outside(width))
    }

    pub fn center(width: impl Into<ValueRef<Length>>) -> Border {
        let width = width.into();
        Border::new(width, BorderPosition::Center)
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

    /// Draws the described border in the given paint context, around the specified bounds.
    pub fn draw(
        &self,
        ctx: &mut PaintCtx,
        bounds: Rect,
        radii: [sk::Vector; 4],
        env: &Environment,
    ) {
        let mut paint = self.paint.to_sk_paint(env, bounds);
        paint.set_style(sk::PaintStyle::Stroke);
        paint.set_blend_mode(self.blend_mode.to_skia());
        //paint.set_alpha_f(self.opacity as sk::scalar);

        let widths = [
            self.widths[0]
                .resolve_or_default(env)
                .to_dips(ctx.scale_factor) as sk::scalar,
            self.widths[1]
                .resolve_or_default(env)
                .to_dips(ctx.scale_factor) as sk::scalar,
            self.widths[2]
                .resolve_or_default(env)
                .to_dips(ctx.scale_factor) as sk::scalar,
            self.widths[3]
                .resolve_or_default(env)
                .to_dips(ctx.scale_factor) as sk::scalar,
        ];
        let uniform_border = widths.iter().all(|&w| ulps_eq!(w, widths[0]));

        let rect = match self.position {
            BorderPosition::Inside(x) => {
                let x = -0.5 - x.resolve_or_default(env).to_dips(ctx.scale_factor);
                bounds.inflate(x, x)
            }
            BorderPosition::Outside(x) => {
                let x = 0.5 + x.resolve_or_default(env).to_dips(ctx.scale_factor);
                bounds.inflate(x, x)
            }
            BorderPosition::Center => bounds,
        };

        if !uniform_border {
            // draw lines, ignore radii
            // TODO border colors

            // left
            if !ulps_eq!(widths[0], 0.0) {
                paint.set_stroke_width(widths[0]);
                ctx.canvas.draw_line(
                    rect.top_left().to_skia(),
                    rect.bottom_left().to_skia(),
                    &paint,
                );
            }

            // top
            if !ulps_eq!(widths[1], 0.0) {
                paint.set_stroke_width(widths[1]);
                ctx.canvas.draw_line(
                    rect.top_left().to_skia(),
                    rect.top_right().to_skia(),
                    &paint,
                );
            }

            // right
            if !ulps_eq!(widths[2], 0.0) {
                paint.set_stroke_width(widths[2]);
                ctx.canvas.draw_line(
                    rect.top_right().to_skia(),
                    rect.bottom_right().to_skia(),
                    &paint,
                );
            }

            // bottom
            if !ulps_eq!(widths[3], 0.0) {
                paint.set_stroke_width(widths[3]);
                ctx.canvas.draw_line(
                    rect.bottom_left().to_skia(),
                    rect.bottom_right().to_skia(),
                    &paint,
                );
            }
        } else {
            if radii[0].is_zero() && radii[1].is_zero() && radii[2].is_zero() && radii[3].is_zero()
            {
                ctx.canvas.draw_rect(rect.to_skia(), &paint);
            } else {
                let rrect = sk::RRect::new_rect_radii(rect.to_skia(), &radii);
                ctx.canvas.draw_rrect(rrect, &paint);
            }
        }
    }
}
