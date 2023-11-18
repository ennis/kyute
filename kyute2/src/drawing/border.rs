use crate::{
    drawing::{BlendMode, Paint},
    Color,
};

/*
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
}*/

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, serde::Deserialize)]
pub enum BorderStyle {
    #[serde(rename = "solid")]
    Solid,
    #[serde(rename = "dotted")]
    Dotted,
    #[serde(rename = "dashed")]
    Dashed,
}

impl Default for BorderStyle {
    fn default() -> Self {
        BorderStyle::Solid
    }
}

#[derive(Clone, Debug)]
pub struct Border {
    pub widths: [f64; 4],
    pub paint: Paint,
    pub line_style: BorderStyle,
    pub blend_mode: BlendMode,
}

impl Default for Border {
    fn default() -> Self {
        Border {
            widths: [0.0; 4],
            paint: Paint::Color(Color::new(0.0, 0.0, 0.0, 0.0)),
            line_style: BorderStyle::Solid,
            blend_mode: BlendMode::SrcOver,
        }
    }
}

/*impl Border {
    /// Draws the described border in the given paint context, around the specified bounds.
    pub fn draw<S: Shape>(&self, ctx: &mut PaintCtx, shape: S) {
        if let Some(rrect) = shape.as_rounded_rect() {
            let [t, r, b, l] = self.widths;

            //rrect.

            let inset_x = 0.5 * (l + r);
            let offset_x = 0.5 * (l - r);
            let inset_y = 0.5 * (t + b);
            let offset_y = 0.5 * (t - b);
            let inset_rrect = rrect.translate(Vec2::new(offset_x, offset_y)).inset(inset_x, inset_y);

            let canvas = ctx.surface.surface().canvas();
            let mut paint = self.paint.to_sk_paint(rrect.rect());
            paint.set_style(sk::PaintStyle::Fill);
            match self.line_style {
                BorderStyle::Solid => {}
                BorderStyle::Dotted => {
                    // TODO: per-side dash pattern
                    let path_effect = sk::PathEffect::dash(&[t as sk::scalar, t as sk::scalar], 0.0);
                    paint.set_path_effect(path_effect);
                }
                BorderStyle::Dashed => {
                    let path_effect = sk::PathEffect::dash(&[5.0, 5.0], 0.0);
                    paint.set_path_effect(path_effect);
                }
            }
            paint.set_blend_mode(self.blend_mode.to_skia());
            canvas.draw_drrect(rrect.to_skia(), inset_rrect.to_skia(), &paint);
        } else {
            warn!("unimplemented border shape")
        }
    }
}
*/
