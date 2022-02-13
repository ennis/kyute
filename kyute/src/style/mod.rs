//! Drawing code for GUI elements.
mod border;
mod box_style;
mod paint;
mod theme;

use crate::{env::Environment, Color, EnvKey, EnvValue, PaintCtx, Rect};
use kyute_shell::{
    drawing::{RectExt, ToSkia},
    skia as sk,
};
use std::str::FromStr;

pub use border::{Border, BorderPosition, BorderStyle};
pub use box_style::{BoxStyle, BoxShadow, BoxShadowParams};
pub use paint::{GradientStop, LinearGradient, Paint};
pub use theme::{define_theme, ThemeData, ThemeLoadError};

/// Unit of length: device-independent pixel.
pub struct Dip;

/// A length in DIPs.
pub type DipLength = euclid::Length<f64, Dip>;
pub type Angle = euclid::Angle<f64>;

/// Length specification.
#[derive(Copy, Clone, Debug, PartialEq, serde::Deserialize)]
#[serde(tag = "unit", content = "value")]
pub enum Length {
    /// Actual screen pixels (the actual physical size depends on the density of the screen).
    #[serde(rename = "px")]
    Px(f64),
    /// Device-independent pixels (DIPs), close to 1/96th of an inch.
    #[serde(rename = "dip")]
    Dip(f64),
    /// Inches (logical inches? approximate inches?).
    #[serde(rename = "in")]
    In(f64),
}

impl Length {
    /// Zero length.
    pub fn zero() -> Length {
        Length::Dip(0.0)
    }

    pub fn to_dips(self, scale_factor: f64) -> f64 {
        match self {
            Length::Px(x) => x / scale_factor,
            Length::In(x) => 96.0 * x,
            Length::Dip(x) => x,
        }
    }
}

impl Default for Length {
    fn default() -> Self {
        Length::Dip(0.0)
    }
}

/// By default, a naked f64 represents a dip.
impl From<f64> for Length {
    fn from(v: f64) -> Self {
        Length::Dip(v)
    }
}

/// Trait for values convertible to DIPs.
pub trait UnitExt {
    fn dip(self) -> Length;
    fn inch(self) -> Length;
    fn px(self) -> Length;
    fn degrees(self) -> Angle;
    fn radians(self) -> Angle;
}

impl UnitExt for f32 {
    fn dip(self) -> Length {
        Length::Dip(self as f64)
    }
    fn inch(self) -> Length {
        Length::In(self as f64)
    }
    fn px(self) -> Length {
        Length::Px(self as f64)
    }
    fn degrees(self) -> Angle {
        Angle::degrees(self as f64)
    }
    fn radians(self) -> Angle {
        Angle::radians(self as f64)
    }
}

impl UnitExt for f64 {
    fn dip(self) -> Length {
        Length::Dip(self)
    }
    fn inch(self) -> Length {
        Length::In(self)
    }
    fn px(self) -> Length {
        Length::Px(self)
    }
    fn degrees(self) -> Angle {
        Angle::degrees(self)
    }
    fn radians(self) -> Angle {
        Angle::radians(self)
    }
}

impl UnitExt for i32 {
    fn dip(self) -> Length {
        Length::Dip(self as f64)
    }
    fn inch(self) -> Length {
        Length::In(self as f64)
    }
    fn px(self) -> Length {
        Length::Px(self as f64)
    }
    fn degrees(self) -> Angle {
        Angle::degrees(self as f64)
    }
    fn radians(self) -> Angle {
        Angle::radians(self as f64)
    }
}

impl UnitExt for u32 {
    fn dip(self) -> Length {
        Length::Dip(self as f64)
    }
    fn inch(self) -> Length {
        Length::In(self as f64)
    }
    fn px(self) -> Length {
        Length::Px(self as f64)
    }
    fn degrees(self) -> Angle {
        Angle::degrees(self as f64)
    }
    fn radians(self) -> Angle {
        Angle::radians(self as f64)
    }
}

/// Describes a blending mode.
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

/// Either a value or a reference to a value in an environment.
#[derive(Copy, Clone, Debug, PartialEq, serde::Deserialize)]
#[serde(untagged)]
pub enum ValueRef<T> {
    /// Inline value.
    Inline(T),
    /// Fetch the value from the environment.
    #[serde(skip)]
    Env(EnvKey<T>),
}

impl<T: EnvValue> ValueRef<T> {
    pub fn resolve(&self, env: &Environment) -> Option<T> {
        match self {
            ValueRef::Inline(v) => Some(v.clone()),
            ValueRef::Env(k) => env.get(*k),
        }
    }
}

impl<T: EnvValue + Default> ValueRef<T> {
    pub fn resolve_or_default(&self, env: &Environment) -> T {
        self.resolve(env).unwrap_or_default()
    }
}

impl<T> From<T> for ValueRef<T> {
    fn from(v: T) -> Self {
        ValueRef::Inline(v)
    }
}

impl<T> From<EnvKey<T>> for ValueRef<T> {
    fn from(k: EnvKey<T>) -> Self {
        ValueRef::Env(k)
    }
}

impl<T> Default for ValueRef<T>
where
    T: Default,
{
    fn default() -> Self {
        ValueRef::Inline(T::default())
    }
}

/// ValueRef to a color.
pub type ColorRef = ValueRef<Color>;

/// Modifier applied to a color expr (`ColorExpr`).
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ColorModifier {
    Darken(f64),
    Lighten(f64),
}

impl ColorModifier {
    pub fn apply(self, color: Color) -> Color {
        match self {
            ColorModifier::Darken(amount) => color.darken(amount),
            ColorModifier::Lighten(amount) => color.lighten(amount),
        }
    }
}

/// A reference to a color value with a modifier.
#[derive(Copy, Clone, Debug, PartialEq, serde::Deserialize)]
pub struct ColorExpr {
    color: ColorRef,
    #[serde(skip)]
    modifier: Option<ColorModifier>,
}

impl ColorExpr {
    pub fn resolve(&self, env: &Environment) -> Option<Color> {
        let color = self.color.resolve(env)?;
        Some(self.modifier.map(|m| m.apply(color)).unwrap_or(color))
    }

    /*pub fn resolve_or_default(&self, env: &Environment) -> Color {
        let color = self.color.resolve(env).unwrap_or_default();
        self.modifier.map(|m| m.apply(color)).unwrap_or(color)
    }*/

    pub fn darken(color: impl Into<ColorRef>, amount: f64) -> ColorExpr {
        ColorExpr {
            color: color.into(),
            modifier: Some(ColorModifier::Darken(amount)),
        }
    }

    pub fn lighten(color: impl Into<ColorRef>, amount: f64) -> ColorExpr {
        ColorExpr {
            color: color.into(),
            modifier: Some(ColorModifier::Lighten(amount)),
        }
    }
}

impl From<ColorRef> for ColorExpr {
    fn from(color: ColorRef) -> Self {
        ColorExpr {
            color,
            modifier: None,
        }
    }
}

impl From<Color> for ColorExpr {
    fn from(color: Color) -> Self {
        ColorExpr {
            color: color.into(),
            modifier: None,
        }
    }
}

impl From<EnvKey<Color>> for ColorExpr {
    fn from(color: EnvKey<Color>) -> Self {
        ColorExpr {
            color: color.into(),
            modifier: None,
        }
    }
}

//--------------------------------------------------------------------------------------------------

/// Represents something that can be drawn in a layout box.
pub trait Visual {
    fn draw(&self, ctx: &mut PaintCtx, bounds: Rect, env: &Environment);
}

impl<VA, VB> Visual for (VA, VB)
where
    VA: Visual,
    VB: Visual,
{
    fn draw(&self, ctx: &mut PaintCtx, bounds: Rect, env: &Environment) {
        self.0.draw(ctx, bounds, env);
        self.1.draw(ctx, bounds, env);
    }
}

pub struct NullVisual;

impl Visual for NullVisual {
    fn draw(&self, _ctx: &mut PaintCtx, _bounds: Rect, _env: &Environment) {}
}

//--------------------------------------------------------------------------------------------------

/// Path visual.
pub struct Path {
    path: kyute_shell::drawing::Path,
    stroke: Option<Paint>,
    fill: Option<Paint>,
    box_shadow: Option<BoxShadow>,
}

impl Path {
    pub fn new(path: &str) -> Path {
        Path {
            path: kyute_shell::drawing::Path::from_str(path).unwrap(),
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

    pub fn draw(&self, ctx: &mut PaintCtx, bounds: Rect, env: &Environment) {
        let sk_path = self.path.to_skia();

        // fill
        if let Some(ref brush) = self.fill {
            let mut paint = brush.to_sk_paint(env, ctx.bounds());
            paint.set_style(sk::PaintStyle::Fill);
            ctx.canvas.save();
            ctx.canvas.translate(bounds.top_left().to_skia());
            ctx.canvas.draw_path(&sk_path, &paint);
            ctx.canvas.restore();
        }

        // stroke
        if let Some(ref stroke) = self.stroke {
            let mut paint = stroke.to_sk_paint(env, ctx.bounds());
            paint.set_style(sk::PaintStyle::Stroke);
            ctx.canvas.save();
            ctx.canvas.translate(bounds.top_left().to_skia());
            ctx.canvas.draw_path(&sk_path, &paint);
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
    fn draw_visual<V: Visual>(&mut self, bounds: Rect, visual: &V, env: &Environment);
    fn draw_styled_box(&mut self, bounds: Rect, box_style: &BoxStyle, env: &Environment);
}

impl<'a> PaintCtxExt for PaintCtx<'a> {
    fn draw_visual<V: Visual>(&mut self, bounds: Rect, visual: &V, env: &Environment) {
        visual.draw(self, bounds, env)
    }

    fn draw_styled_box(&mut self, bounds: Rect, box_style: &BoxStyle, env: &Environment) {
        box_style.draw(self, bounds, env)
    }
}
