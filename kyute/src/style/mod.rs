//! Drawing code for GUI elements.
mod border;
mod box_style;
mod paint;
mod theme;

use crate::{
    drawing::{svg_path_to_skia, ToSkia},
    env::Environment,
    Color, EnvKey, Length, PaintCtx, Rect, RectExt, Transform, UnitExt, ValueRef,
};
use bitflags::bitflags;
use skia_safe as sk;

use crate::style::paint::IntoPaint;
pub use border::{Border, BorderPosition, BorderStyle};
pub use box_style::{BoxShadow, BoxShadowParams, BoxStyle};
pub use paint::{GradientStop, LinearGradient, Paint};
pub use theme::{define_theme, ThemeData, ThemeLoadError};

bitflags! {
    #[derive(Default)]
    pub struct VisualState: u8 {
        const DEFAULT = 0;
        const FOCUS   = 1 << 0;
        const ACTIVE  = 1 << 1;
        const HOVER   = 1 << 2;
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

/// ValueRef to a color.
pub type ColorRef = ValueRef<Color>;

pub fn darken(color: impl Into<ColorRef>, amount: f64) -> Color {
    color.into().resolve().unwrap().darken(amount)
}

pub fn lighten(color: impl Into<ColorRef>, amount: f64) -> Color {
    color.into().resolve().unwrap().lighten(amount)
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
    pub fn fill(mut self, paint: impl IntoPaint) -> Self {
        self.fill = Some(paint.into_paint());
        self
    }

    /// Sets the brush used to stroke the path.
    pub fn stroke(mut self, paint: impl IntoPaint) -> Self {
        self.fill = Some(paint.into_paint());
        self
    }

    pub fn draw(&self, ctx: &mut PaintCtx, bounds: Rect) {
        // fill
        if let Some(ref brush) = self.fill {
            let mut paint = brush.to_sk_paint(bounds);
            paint.set_style(sk::PaintStyle::Fill);
            ctx.canvas.save();
            ctx.canvas.translate(bounds.top_left().to_skia());
            ctx.canvas.draw_path(&self.path, &paint);
            ctx.canvas.restore();
        }

        // stroke
        if let Some(ref stroke) = self.stroke {
            let mut paint = stroke.to_sk_paint(bounds);
            paint.set_style(sk::PaintStyle::Stroke);
            ctx.canvas.save();
            ctx.canvas.translate(bounds.top_left().to_skia());
            ctx.canvas.draw_path(&self.path, &paint);
            ctx.canvas.restore();
        }
    }
}

//--------------------------------------------------------------------------------------------------

pub trait CornerLengths {
    /// Returns corner lengths.
    fn into_corner_lengths(self) -> [Length; 4];
}

impl CornerLengths for Length {
    fn into_corner_lengths(self) -> [Length; 4] {
        [self; 4]
    }
}

impl CornerLengths for f64 {
    fn into_corner_lengths(self) -> [Length; 4] {
        [self.dip(); 4]
    }
}

impl CornerLengths for f32 {
    fn into_corner_lengths(self) -> [Length; 4] {
        [self.dip(); 4]
    }
}

//--------------------------------------------------------------------------------------------------
pub trait PaintCtxExt {
    fn draw_styled_box(&mut self, bounds: Rect, box_style: &BoxStyle);
}

impl<'a> PaintCtxExt for PaintCtx<'a> {
    fn draw_styled_box(&mut self, bounds: Rect, box_style: &BoxStyle) {
        box_style.draw(self, bounds)
    }
}
