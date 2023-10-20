use crate::{drawing::ToSkia, Color, Rect, Vec2};
use kurbo::{Insets, RoundedRect};
use skia_safe as sk;

/// Box shadow parameters.
#[derive(Copy, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serializing", derive(serde::Deserialize))]
pub struct BoxShadow {
    pub color: Color,
    pub offset: Vec2,
    pub blur: f64,
    pub spread: f64,
    pub inset: bool,
}

/// Adapted from https://source.chromium.org/chromium/chromium/src/+/main:third_party/blink/renderer/core/paint/box_painter_base.cc;drc=3d2b7a03c8d788be1803d1fa5a79999508ad26dc;l=268
/// Adjusts the size of the outer rrect for drawing an inset shadow
/// (so that, once blurred, we get the correct result).
fn area_casting_shadow_in_hole(hole: &Rect, offset: Vec2, blur_radius: f64, spread: f64) -> Rect {
    let mut bounds = *hole;
    bounds = bounds.inflate(blur_radius, blur_radius);
    if spread < 0.0 {
        bounds = bounds.inflate(-spread, -spread);
    }
    let offset_bounds = bounds - offset;
    bounds.union(offset_bounds)
}

// Per spec, sigma is exactly half the blur radius:
// https://www.w3.org/TR/css-backgrounds-3/#shadow-blur
// https://html.spec.whatwg.org/C/#when-shadows-are-drawn
fn blur_radius_to_std_dev(radius: f64) -> sk::scalar {
    (radius * 0.5) as sk::scalar
}

pub fn draw_box_shadow(canvas: &mut sk::Canvas, rrect: &RoundedRect, shadow: &BoxShadow) {
    // setup skia paint (mask blur)
    let mut shadow_paint = sk::Paint::default();
    shadow_paint.set_mask_filter(sk::MaskFilter::blur(
        sk::BlurStyle::Normal,
        blur_radius_to_std_dev(shadow.blur),
        None,
    ));
    shadow_paint.set_color(shadow.color.to_skia().to_color());
    shadow_paint.set_anti_alias(true);

    if !shadow.inset {
        // drop shadow
        let rect2 = (rrect.rect() + shadow.offset).inset(Insets::uniform(shadow.spread));
        let shadow_rrect = RoundedRect::from_rect(rect2, rrect.radii());
        canvas.draw_rrect(shadow_rrect.to_skia(), &shadow_paint);
    } else {
        // inset shadow

        // The shadow shape of the element is a ring shape bounded by:
        // - inside: the inner edge of the shadow inside the element, which is the element shape
        //   translated by the shadow offset and inset by the spread.
        // - outside: a surrounding rectangle, chosen so that the area is large enough to produce the
        //   correct drop shadow when blurred (see `area_casting_shadow_in_hole`, taken from chromium).
        //
        // Visually:
        //
        //     outer_rrect(not rounded)
        //     ┌───────────────────────────────────────────────┐
        //     │      inner_rrect: inset and translated        │
        //     │    ╭────────────────────────────────────╮     │
        //     │    │                                    │     │
        //     │    │                                    │     │
        //     │    │                                    │     │
        //     │    │                                    │     │
        //     │    ╰────────────────────────────────────╯     │
        //     │                                               │
        //     └───────────────────────────────────────────────┘
        //
        // The element shape should be wholly contained within outer_rrect.
        // The shadow shape is drawn (with drawDRRect) with a blur filter, and clipped to the
        // element shape.
        /*let inner_rrect = rrect.translate(shadow.offset).inset(self.spread, self.spread);
        let outer_rrect: RoundedRect =
            area_casting_shadow_in_hole(&inner_rrect.rect, self.offset, self.blur, self.spread).into();
        let inner_rrect = inner_rrect.to_skia();
        let outer_rrect = outer_rrect.to_skia();
        let canvas = ctx.surface.canvas();
        canvas.save();
        canvas.clip_rrect(rrect.to_skia(), sk::ClipOp::Intersect, true);
        canvas.draw_drrect(outer_rrect, inner_rrect, &shadow_paint);
        canvas.restore();*/
        todo!("inset shadows")
    }
}
