//! Drawing code for GUI elements.
mod border;
mod box_style;
mod paint;
mod parser;
mod theme;
mod style2;

use crate::{
    animation::PaintCtx,
    drawing::{svg_path_to_skia, ToSkia},
    Color, EnvRef, Length, Offset, Rect, RectExt, UnitExt,
};
use bitflags::bitflags;
use skia_safe as sk;
use std::convert::{TryFrom, TryInto};

pub use border::{Border, BorderPosition, BorderStyle};
pub use paint::{ColorStop, LinearGradient, Paint, RepeatMode, UniformData};
pub use theme::{define_theme, ThemeData, ThemeLoadError};

bitflags! {
    /// Encodes the active visual states of a widget.
    #[derive(Default)]
    pub struct VisualState: u8 {
        /// Normal state.
        const DEFAULT  = 0;

        /// The widget has focus.
        ///
        /// Typically a border or a color highlight is drawn on the widget to signify the focused state.
        const FOCUS    = 1 << 0;

        /// The widget is "active" (e.g. pressed, for a button).
        const ACTIVE   = 1 << 1;

        /// A cursor is hovering atop the widget.
        const HOVER    = 1 << 2;

        /// The widget is disabled.
        ///
        /// Typically a widget is "greyed-out" when it is disabled.
        const DISABLED = 1 << 3;
    }
}

/// Describes a blending mode.
// TODO move to crate::drawing?
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BlendMode {
    Clear,
    Src,
    Dst,
    SrcOver,
    DstOver,
    SrcIn,
    DstIn,
    SrcOut,
    DstOut,
    SrcATop,
    DstATop,
    Xor,
    Plus,
    Modulate,
    Screen,
    Overlay,
    Darken,
    Lighten,
    ColorDodge,
    ColorBurn,
    HardLight,
    SoftLight,
    Difference,
    Exclusion,
    Multiply,
    Hue,
    Saturation,
    Color,
    Luminosity,
}

impl ToSkia for BlendMode {
    type Target = sk::BlendMode;

    fn to_skia(&self) -> Self::Target {
        match *self {
            BlendMode::Clear => sk::BlendMode::Clear,
            BlendMode::Src => sk::BlendMode::Src,
            BlendMode::Dst => sk::BlendMode::Dst,
            BlendMode::SrcOver => sk::BlendMode::SrcOver,
            BlendMode::DstOver => sk::BlendMode::DstOver,
            BlendMode::SrcIn => sk::BlendMode::SrcIn,
            BlendMode::DstIn => sk::BlendMode::DstIn,
            BlendMode::SrcOut => sk::BlendMode::SrcOut,
            BlendMode::DstOut => sk::BlendMode::DstOut,
            BlendMode::SrcATop => sk::BlendMode::SrcATop,
            BlendMode::DstATop => sk::BlendMode::DstATop,
            BlendMode::Xor => sk::BlendMode::Xor,
            BlendMode::Plus => sk::BlendMode::Plus,
            BlendMode::Modulate => sk::BlendMode::Modulate,
            BlendMode::Screen => sk::BlendMode::Screen,
            BlendMode::Overlay => sk::BlendMode::Overlay,
            BlendMode::Darken => sk::BlendMode::Darken,
            BlendMode::Lighten => sk::BlendMode::Lighten,
            BlendMode::ColorDodge => sk::BlendMode::ColorDodge,
            BlendMode::ColorBurn => sk::BlendMode::ColorBurn,
            BlendMode::HardLight => sk::BlendMode::HardLight,
            BlendMode::SoftLight => sk::BlendMode::SoftLight,
            BlendMode::Difference => sk::BlendMode::Difference,
            BlendMode::Exclusion => sk::BlendMode::Exclusion,
            BlendMode::Multiply => sk::BlendMode::Multiply,
            BlendMode::Hue => sk::BlendMode::Hue,
            BlendMode::Saturation => sk::BlendMode::Saturation,
            BlendMode::Color => sk::BlendMode::Color,
            BlendMode::Luminosity => sk::BlendMode::Luminosity,
        }
    }
}

//--------------------------------------------------------------------------------------------------

/// Path visual.
pub struct Path {
    path: sk::Path,
    stroke: Option<Paint>,
    fill: Option<Paint>,
    box_shadow: Option<BoxShadow>,
}

impl Path {
    pub fn new(svg_path: &str) -> Path {
        Path {
            path: svg_path_to_skia(svg_path).expect("invalid path syntax"),
            stroke: None,
            fill: None,
            box_shadow: None,
        }
    }

    /// Sets the brush used to fill the path.
    pub fn fill(mut self, paint: impl Into<Paint>) -> Self {
        self.fill = Some(paint.into());
        self
    }

    /// Sets the brush used to stroke the path.
    pub fn stroke(mut self, paint: impl Into<Paint>) -> Self {
        self.fill = Some(paint.into());
        self
    }

    pub fn draw(&self, ctx: &mut PaintCtx, bounds: Rect) {
        // fill
        let canvas = ctx.surface.canvas();
        if let Some(ref brush) = self.fill {
            let mut paint = brush.to_sk_paint(bounds);
            paint.set_style(sk::PaintStyle::Fill);
            canvas.save();
            canvas.translate(bounds.top_left().to_skia());
            canvas.draw_path(&self.path, &paint);
            canvas.restore();
        }

        // stroke
        if let Some(ref stroke) = self.stroke {
            let mut paint = stroke.to_sk_paint(bounds);
            paint.set_style(sk::PaintStyle::Stroke);
            canvas.save();
            canvas.translate(bounds.top_left().to_skia());
            canvas.draw_path(&self.path, &paint);
            canvas.restore();
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

/// Box shadow parameters.
#[derive(Copy, Clone, Debug, PartialEq, serde::Deserialize)]
pub struct BoxShadow {
    pub color: Color,
    pub x_offset: Length,
    pub y_offset: Length,
    pub blur: Length,
    pub spread: Length,
    pub inset: bool,
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Style
////////////////////////////////////////////////////////////////////////////////////////////////////

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

/// Style of a container.
#[derive(Clone, Debug)]
pub struct Style {
    pub border_radii: [Length; 4],
    pub border: Option<Border>,
    pub background: Option<Paint>,
    pub box_shadows: Vec<BoxShadow>,
}

impl Default for Style {
    fn default() -> Self {
        Style::new()
    }
}

fn radii_to_skia(ctx: &mut PaintCtx, bounds: Rect, radii: &[Length; 4]) -> [sk::Vector; 4] {
    // FIXME: height-relative sizes
    let radii_dips = [
        radii[0].to_dips(ctx.scale_factor, bounds.size.width),
        radii[1].to_dips(ctx.scale_factor, bounds.size.width),
        radii[2].to_dips(ctx.scale_factor, bounds.size.width),
        radii[3].to_dips(ctx.scale_factor, bounds.size.width),
    ];

    // TODO x,y radii
    [
        sk::Vector::new(radii_dips[0] as sk::scalar, radii_dips[0] as sk::scalar),
        sk::Vector::new(radii_dips[1] as sk::scalar, radii_dips[1] as sk::scalar),
        sk::Vector::new(radii_dips[2] as sk::scalar, radii_dips[2] as sk::scalar),
        sk::Vector::new(radii_dips[3] as sk::scalar, radii_dips[3] as sk::scalar),
    ]
}

impl Style {
    pub fn new() -> Style {
        Style {
            border_radii: [Length::Dip(0.0); 4],
            background: None,
            border: None,
            box_shadows: vec![],
        }
    }

    ///
    pub fn is_transparent(&self) -> bool {
        self.background.is_none() && self.border.is_none() && self.box_shadows.is_empty()
    }

    pub fn clip_rect(&self, bounds: Rect, scale_factor: f64) -> Rect {
        let mut rect = bounds;
        for box_shadow in self.box_shadows.iter() {
            if !box_shadow.inset {
                let mut shadow_rect = bounds;
                shadow_rect.origin.x += box_shadow.x_offset.to_dips(scale_factor, bounds.width());
                shadow_rect.origin.y += box_shadow.y_offset.to_dips(scale_factor, bounds.height());
                let spread = box_shadow.spread.to_dips(scale_factor, bounds.width());
                let radius = box_shadow.blur.to_dips(scale_factor, bounds.width());
                shadow_rect = shadow_rect.inflate(spread + radius, spread + radius);
                rect = rect.union(&shadow_rect);
            }
        }
        rect
    }

    /// Specifies the radius of the 4 corners of the box.
    pub fn radius(mut self, radius: impl Into<Length>) -> Self {
        let radius = radius.into();
        self.border_radii = [radius; 4];
        self
    }

    /// Specifies the radius of each corner of the box separately.
    pub fn radii(
        mut self,
        top_left: impl Into<Length>,
        top_right: impl Into<Length>,
        bottom_right: impl Into<Length>,
        bottom_left: impl Into<Length>,
    ) -> Self {
        self.border_radii = [
            top_left.into(),
            top_right.into(),
            bottom_right.into(),
            bottom_left.into(),
        ];
        self
    }

    /// Sets the brush used to fill the rectangle.
    pub fn background(mut self, paint: impl Into<Paint>) -> Self {
        self.background = Some(paint.into());
        self
    }

    /// Sets the border.
    pub fn border(mut self, border: Border) -> Self {
        self.border = Some(border);
        self
    }

    /// Adds a box shadow.
    pub fn box_shadow(mut self, box_shadow: BoxShadow) -> Self {
        self.box_shadows.push(box_shadow);
        self
    }

    /// Draws a box with this style in the given bounds.
    pub fn draw(&self, ctx: &mut PaintCtx, bounds: Rect) {
        let radii = radii_to_skia(ctx, bounds, &self.border_radii);
        let canvas = ctx.surface.canvas();

        // --- box shadows ---
        // TODO move in own function
        for box_shadow in self.box_shadows.iter() {
            let x_offset = box_shadow.x_offset.to_dips(ctx.scale_factor, bounds.size.width);
            let y_offset = box_shadow.y_offset.to_dips(ctx.scale_factor, bounds.size.height);
            let offset = Offset::new(x_offset, y_offset);
            let blur = box_shadow.blur.to_dips(ctx.scale_factor, bounds.size.width);
            let spread = box_shadow.spread.to_dips(ctx.scale_factor, bounds.size.width);
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
                let mut rect = bounds.translate(offset).inflate(spread, spread);
                // TODO adjust radius
                let rrect = sk::RRect::new_rect_radii(rect.to_skia(), &radii);
                canvas.draw_rrect(rrect, &shadow_paint);
            } else {
                let inner_rect = bounds.translate(offset).inflate(-spread, -spread);
                let outer_rect = area_casting_shadow_in_hole(bounds, offset, blur, spread);
                // TODO adjust radius
                let inner_rrect = sk::RRect::new_rect_radii(inner_rect.to_skia(), &radii);
                let outer_rrect = sk::RRect::new_rect_radii(outer_rect.to_skia(), &radii);
                canvas.draw_drrect(outer_rrect, inner_rrect, &shadow_paint);
            }
        }

        // --- background ---
        if let Some(ref brush) = self.background {
            let mut paint = brush.to_sk_paint(bounds);
            paint.set_style(sk::PaintStyle::Fill);
            let rrect = sk::RRect::new_rect_radii(bounds.to_skia(), &radii);
            ctx.surface.canvas().draw_rrect(rrect, &paint);
        }

        // --- border ---
        if let Some(ref border) = self.border {
            border.draw(ctx, bounds, radii);
        }
    }
}

impl_env_value!(Style);

/// From CSS value.
impl TryFrom<&str> for Style {
    type Error = ();
    fn try_from(css: &str) -> Result<Self, ()> {
        Style::parse(css).map_err(|_| ())
    }
}

//--------------------------------------------------------------------------------------------------
pub trait PaintCtxExt {
    fn draw_styled_box(&mut self, bounds: Rect, box_style: &Style);
}

impl<'a> PaintCtxExt for PaintCtx<'a> {
    fn draw_styled_box(&mut self, bounds: Rect, box_style: &Style) {
        box_style.draw(self, bounds)
    }
}
