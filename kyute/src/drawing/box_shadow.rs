use crate::drawing::{PaintCtx, RoundedRect, Shape, ToSkia};
use kyute_common::{Color, Offset, Rect};
use skia_safe as sk;

/// Box shadow parameters.
#[derive(Copy, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serializing", derive(serde::Deserialize))]
pub struct BoxShadow {
    pub color: Color,
    pub offset: Offset,
    pub blur: f64,
    pub spread: f64,
    pub inset: bool,
}

/// Adapted from https://source.chromium.org/chromium/chromium/src/+/main:third_party/blink/renderer/core/paint/box_painter_base.cc;drc=3d2b7a03c8d788be1803d1fa5a79999508ad26dc;l=268
/// Adjusts the size of the outer rrect for drawing an inset shadow
/// (so that, once blurred, we get the correct result).
fn area_casting_shadow_in_hole(hole: &Rect, offset: Offset, blur_radius: f64, spread: f64) -> Rect {
    let mut bounds = *hole;
    bounds = bounds.inflate(blur_radius, blur_radius);
    if spread < 0.0 {
        bounds = bounds.inflate(-spread, -spread);
    }
    let offset_bounds = bounds.translate(-offset);
    bounds.union(&offset_bounds)
}

// Per spec, sigma is exactly half the blur radius:
// https://www.w3.org/TR/css-backgrounds-3/#shadow-blur
// https://html.spec.whatwg.org/C/#when-shadows-are-drawn
fn blur_radius_to_std_dev(radius: f64) -> sk::scalar {
    (radius * 0.5) as sk::scalar
}

/// Draws a box shadow for the specified rounded rectangle shape.
///
/// The radii are specified clockwise starting from the top left corner.
impl BoxShadow {
    pub fn draw(&self, ctx: &mut PaintCtx, shape: &Shape) {
        match shape {
            Shape::RoundedRect(rrect) => {
                // setup skia paint (mask blur)
                let mut shadow_paint = sk::Paint::default();
                shadow_paint.set_mask_filter(sk::MaskFilter::blur(
                    sk::BlurStyle::Normal,
                    blur_radius_to_std_dev(self.blur),
                    None,
                ));
                shadow_paint.set_color(self.color.to_skia().to_color());

                if !self.inset {
                    // drop shadow
                    // calculate base shadow shape rectangle (apply offset & spread)
                    let shadow_rrect = rrect.translate(self.offset).outset(self.spread, self.spread);
                    ctx.surface.canvas().draw_rrect(shadow_rrect.to_skia(), &shadow_paint);
                } else {
                    let inner_rrect = rrect.translate(self.offset).inset(self.spread, self.spread);
                    let outer_rrect: RoundedRect =
                        area_casting_shadow_in_hole(&rrect.rect, self.offset, self.blur, self.spread).into();
                    let inner_rrect = inner_rrect.to_skia();
                    let outer_rrect = outer_rrect.to_skia();
                    ctx.surface
                        .canvas()
                        .draw_drrect(outer_rrect, inner_rrect, &shadow_paint);
                }
            }
        }
    }
}
