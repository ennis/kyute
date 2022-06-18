/// Adapted from https://source.chromium.org/chromium/chromium/src/+/main:third_party/blink/renderer/core/paint/box_painter_base.cc;drc=3d2b7a03c8d788be1803d1fa5a79999508ad26dc;l=268
/// Adjusts the size of the outer rrect for drawing an inset shadow
/// (so that, once blurred, we get the correct result).
fn area_casting_shadow_in_hole(hole: Rect, offset: Offset, blur_radius: f64, spread: f64) -> Rect {
    let mut bounds = hole;
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
pub(crate) fn draw_box_shadow(ctx: &mut PaintCtx, rect: Rect, corner_radii: [f64; 4], box_shadow: &BoxShadow) {
    // skia can draw rounded rects with different x & y corner radii, but we don't support that yet
    let corner_radii = [
        sk::Vector::new(corner_radii[0] as sk::scalar, corner_radii[0] as sk::scalar),
        sk::Vector::new(corner_radii[1] as sk::scalar, corner_radii[1] as sk::scalar),
        sk::Vector::new(corner_radii[2] as sk::scalar, corner_radii[2] as sk::scalar),
        sk::Vector::new(corner_radii[3] as sk::scalar, corner_radii[3] as sk::scalar),
    ];

    let x_offset = box_shadow.x_offset.to_dips(length_ctx);
    let y_offset = box_shadow.y_offset.to_dips(length_ctx);
    let offset = Offset::new(x_offset, y_offset);
    let blur = box_shadow.blur.to_dips(length_ctx);
    let spread = box_shadow.spread.to_dips(length_ctx);
    let color = box_shadow.color;

    // setup skia paint (mask blur)
    let mut shadow_paint = sk::Paint::default();
    shadow_paint.set_mask_filter(sk::MaskFilter::blur(
        sk::BlurStyle::Normal,
        blur_radius_to_std_dev(blur),
        None,
    ));
    shadow_paint.set_color(color.to_skia().to_color());

    if !box_shadow.inset {
        // drop shadow
        // calculate base shadow shape rectangle (apply offset & spread)
        let mut shadow_rect = rect.translate(offset).inflate(spread, spread);
        // TODO adjust radius
        let rrect = sk::RRect::new_rect_radii(shadow_rect.to_skia(), &corner_radii);
        canvas.draw_rrect(rrect, &shadow_paint);
    } else {
        let inner_rect = rect.translate(offset).inflate(-spread, -spread);
        let outer_rect = area_casting_shadow_in_hole(rect, offset, blur, spread);
        // TODO adjust radius
        let inner_rrect = sk::RRect::new_rect_radii(inner_rect.to_skia(), &corner_radii);
        let outer_rrect = sk::RRect::new_rect_radii(outer_rect.to_skia(), &corner_radii);
        canvas.draw_drrect(outer_rrect, inner_rrect, &shadow_paint);
    }
}
