use crate::{
    drawing::ToSkia,
    style::{border::Border, ColorRef, Length, Paint},
    Environment, Offset, PaintCtx, Rect, ValueRef,
};
use skia_safe as sk;

//--------------------------------------------------------------------------------------------------

fn radii_to_skia(ctx: &mut PaintCtx, radii: &[Length; 4]) -> [sk::Vector; 4] {
    let radii_dips = [
        radii[0].to_dips(ctx.scale_factor),
        radii[1].to_dips(ctx.scale_factor),
        radii[2].to_dips(ctx.scale_factor),
        radii[3].to_dips(ctx.scale_factor),
    ];

    // TODO x,y radii
    [
        sk::Vector::new(radii_dips[0] as sk::scalar, radii_dips[0] as sk::scalar),
        sk::Vector::new(radii_dips[1] as sk::scalar, radii_dips[1] as sk::scalar),
        sk::Vector::new(radii_dips[2] as sk::scalar, radii_dips[2] as sk::scalar),
        sk::Vector::new(radii_dips[3] as sk::scalar, radii_dips[3] as sk::scalar),
    ]
}

//--------------------------------------------------------------------------------------------------

/// Parameters of a box shadow effect.
#[derive(Copy, Clone, Debug, PartialEq, serde::Deserialize)]
pub struct BoxShadowParams {
    offset_x: ValueRef<Length>,
    offset_y: ValueRef<Length>,
    blur_radius: ValueRef<Length>,
    spread_radius: ValueRef<Length>,
    color: ColorRef,
}

/// Box shadow effect.
#[derive(Copy, Clone, Debug, PartialEq, serde::Deserialize)]
#[serde(tag = "type")]
pub enum BoxShadow {
    #[serde(rename = "drop")]
    Drop(BoxShadowParams),
    #[serde(rename = "inset")]
    Inset(BoxShadowParams),
}

impl BoxShadow {
    pub fn drop(
        offset_x: impl Into<ValueRef<Length>>,
        offset_y: impl Into<ValueRef<Length>>,
        blur_radius: impl Into<ValueRef<Length>>,
        spread_radius: impl Into<ValueRef<Length>>,
        color: impl Into<ColorRef>,
    ) -> BoxShadow {
        let params = BoxShadowParams {
            offset_x: offset_x.into(),
            offset_y: offset_y.into(),
            blur_radius: blur_radius.into(),
            spread_radius: spread_radius.into(),
            color: color.into(),
        };
        BoxShadow::Drop(params)
    }

    pub fn inset(
        offset_x: impl Into<ValueRef<Length>>,
        offset_y: impl Into<ValueRef<Length>>,
        blur_radius: impl Into<ValueRef<Length>>,
        spread_radius: impl Into<ValueRef<Length>>,
        color: impl Into<ColorRef>,
    ) -> BoxShadow {
        let params = BoxShadowParams {
            offset_x: offset_x.into(),
            offset_y: offset_y.into(),
            blur_radius: blur_radius.into(),
            spread_radius: spread_radius.into(),
            color: color.into(),
        };
        BoxShadow::Inset(params)
    }
}

/// Style of a container.
#[derive(Clone, Debug, serde::Deserialize)]
pub struct BoxStyle {
    border_radii: [ValueRef<Length>; 4],
    fill: Option<Paint>,
    border: Option<Border>,
    box_shadow: Option<BoxShadow>,
}

impl Default for BoxStyle {
    fn default() -> Self {
        BoxStyle::new()
    }
}

impl BoxStyle {
    pub fn new() -> BoxStyle {
        BoxStyle {
            border_radii: [ValueRef::Inline(Length::Dip(0.0)); 4],
            fill: None,
            border: None,
            box_shadow: None,
        }
    }

    /// Creates a new box with rounded corners.
    pub fn new_rounded(border_radii: [ValueRef<Length>; 4]) -> BoxStyle {
        BoxStyle {
            border_radii,
            fill: None,
            border: None,
            box_shadow: None,
        }
    }

    pub fn radius(mut self, radius: impl Into<ValueRef<Length>>) -> Self {
        let radius = radius.into();
        self.border_radii = [radius; 4];
        self
    }

    /// Sets the brush used to fill the rectangle.
    pub fn fill(mut self, paint: impl Into<Paint>) -> Self {
        self.fill = Some(paint.into());
        self
    }

    /// Adds a border.
    pub fn border(mut self, border: Border) -> Self {
        self.border = Some(border);
        self
    }

    /// Adds a box shadow.
    pub fn box_shadow(mut self, box_shadow: BoxShadow) -> Self {
        self.box_shadow = Some(box_shadow);
        self
    }

    /// Draws a box with this style in the given bounds.
    pub fn draw(&self, ctx: &mut PaintCtx, bounds: Rect, env: &Environment) {
        let radii = [
            self.border_radii[0].resolve_or_default(env),
            self.border_radii[1].resolve_or_default(env),
            self.border_radii[2].resolve_or_default(env),
            self.border_radii[3].resolve_or_default(env),
        ];
        let radii = radii_to_skia(ctx, &radii);

        // box shadow
        if let Some(ref box_shadow) = self.box_shadow {
            let params = match box_shadow {
                BoxShadow::Drop(params) | BoxShadow::Inset(params) => params,
            };

            let mut blur = sk::Paint::default();
            let offset_x = params.offset_x.resolve_or_default(env).to_dips(ctx.scale_factor);
            let offset_y = params.offset_y.resolve_or_default(env).to_dips(ctx.scale_factor);
            let blur_radius = params.blur_radius.resolve_or_default(env).to_dips(ctx.scale_factor);
            let spread = params.spread_radius.resolve_or_default(env).to_dips(ctx.scale_factor);
            let color = params.color.resolve_or_default(env);
            blur.set_mask_filter(sk::MaskFilter::blur(
                sk::BlurStyle::Normal,
                blur_radius as sk::scalar,
                None,
            ));
            blur.set_color(color.to_skia().to_color());

            match box_shadow {
                BoxShadow::Drop(_) => {
                    let mut shadow_bounds = bounds;
                    shadow_bounds.origin += Offset::new(offset_x, offset_y);
                    shadow_bounds = shadow_bounds.inflate(spread, spread);
                    let rrect = sk::RRect::new_rect_radii(shadow_bounds.to_skia(), &radii);
                    ctx.canvas.draw_rrect(rrect, &blur);
                }
                BoxShadow::Inset(_) => {
                    // TODO
                }
            }
        }

        // fill
        if let Some(ref brush) = self.fill {
            let mut paint = brush.to_sk_paint(env, ctx.bounds());
            paint.set_style(sk::PaintStyle::Fill);
            let rrect = sk::RRect::new_rect_radii(bounds.to_skia(), &radii);
            ctx.canvas.draw_rrect(rrect, &paint);
        }

        // borders
        if let Some(ref border) = self.border {
            border.draw(ctx, bounds, radii, env);
        }
    }
}

impl_env_value!(BoxStyle);
