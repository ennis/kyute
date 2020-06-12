//! Rendering of styled boxes.
use crate::Bounds;
use kyute_shell::drawing::{
    Color, ColorInterpolationMode, DrawContext, ExtendMode, GradientStopCollection, Offset, Point,
};
use palette::Srgba;
use std::collections::HashMap;
use std::fmt::Debug;

use bitflags::_core::fmt::Formatter;
use serde::de::{EnumAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use kyute_shell::drawing;
use kyute_shell::platform::Platform;

/// Unit of length: device-independent pixel.
pub struct Dip;

/// A length in DIPs.
pub type DipLength = euclid::Length<f64, Dip>;
pub type Angle = euclid::Angle<f64>;

/// Length specification.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Length {
    /// Actual screen pixels (the actual physical size depends on the density of the screen).
    Px(f64),
    /// Device-independent pixels (DIPs), close to 1/96th of an inch.
    Dip(f64),
    /// Inches (logical inches? approximate inches?).
    In(f64),
}

impl Length {
    pub fn zero() -> Length {
        Length::Dip(0.0)
    }

    pub fn to_dips(self, ctx: &DrawContext) -> f64 {
        match self {
            Length::Px(x) => x / ctx.scale_factor(),
            Length::In(x) => 96.0 * x,
            Length::Dip(x) => x,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ColorRef {
    Direct(Color),
    Palette(PaletteIndex),
}

// Provide our own impl for Serialize/Deserialize for anything that has a Color
// because of https://github.com/serde-rs/serde/issues/1346
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
enum SerdeColorRef {
    Direct { r: f32, g: f32, b: f32, a: f32 },
    Palette { index: u32 },
}

impl serde::Serialize for ColorRef {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let c = match self {
            ColorRef::Direct(c) => SerdeColorRef::Direct {
                r: c.red,
                g: c.green,
                b: c.blue,
                a: c.alpha,
            },
            ColorRef::Palette(c) => SerdeColorRef::Palette{ index: c.0 }
        };
        c.serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for ColorRef {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let c = SerdeColorRef::deserialize(deserializer)?;
        Ok(match c {
            SerdeColorRef::Direct { r, g, b, a } => ColorRef::Direct(Color::new(r, g, b, a)),
            SerdeColorRef::Palette { index } => ColorRef::Palette(PaletteIndex(index)),
        })
    }
}

#[derive(Clone, Debug)]
pub enum Value<T: Clone + Debug> {
    /// Constant
    Constant(T),
    /// Expression that evaluates to a value of type `T`.
    Expr(String),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Shape {
    /// Rectangle.
    Rect,
    /// Rounded rectangle.
    RoundedRect(Length),
    /// Path (as an SVG string).
    Path(String),
    /// URL to the mask.
    MaskBitmap(String),
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum BlendMode {
    Normal,
    Multiply,
    Screen,
    Overlay,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum GradientType {
    Linear,
    Radial,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Brush {
    SolidColor(ColorRef),
    Gradient {
        angle: Angle,
        ty: GradientType,
        stops: Vec<(f64, ColorRef)>,
        reverse: bool,
    },
    Pattern {
        url: String,
    },
}

bitflags::bitflags! {
    #[derive(Serialize, Deserialize)]
    pub struct State: u32 {
        const ACTIVE = 1<<0;
        const DISABLED = 1<<1;
        const HOVER = 1<<2;
        const FOCUS = 1<<3;
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct StateFilter {
    pub value: State,
    pub mask: State,
}

impl Default for StateFilter {
    fn default() -> Self {
        StateFilter {
            value: State::empty(),
            mask: State::empty(),
        }
    }
}

/// Border reference position
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum BorderPosition {
    Inside(Length),
    Center,
    Outside(Length),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Border {
    pub opacity: f64,
    pub blend_mode: BlendMode,
    pub position: BorderPosition,
    pub width: Length,
    pub brush: Brush,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Shadow {
    pub opacity: f64,
    pub blend_mode: BlendMode,
    pub color: ColorRef,
    pub angle: Angle,
    pub distance: Length,
    pub spread: f64,
    pub size: Length,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Glow {
    pub opacity: f64,
    pub blend_mode: BlendMode,
    pub color: ColorRef,
    pub size: Length,
    pub choke: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Style {
    pub state_filter: StateFilter,
    pub fill: Option<Brush>,
    pub borders: Vec<Border>,
    pub inner_shadow: Option<Shadow>,
    pub drop_shadow: Option<Shadow>,
    pub inner_glow: Option<Glow>,
    pub outer_glow: Option<Glow>,
}

impl Default for Style {
    fn default() -> Self {
        Style {
            state_filter: StateFilter::default(),
            fill: None,
            borders: vec![],
            inner_shadow: None,
            drop_shadow: None,
            inner_glow: None,
            outer_glow: None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StyleSet {
    pub shape: Shape,
    pub styles: Vec<Style>,
}

#[derive(Clone, Debug)]
pub struct Palette {
    pub entries: Vec<Color>,
}

impl Palette {
    pub fn color(&self, index: u32) -> Color {
        self.entries
            .get(index as usize)
            .cloned()
            .unwrap_or(self.entries[0])
    }

    pub fn resolve(&self, colorref: ColorRef) -> Color {
        match colorref {
            ColorRef::Direct(c) => c,
            ColorRef::Palette(index) => self.color(index.0),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct SerdePalette(Vec<(f32, f32, f32, f32)>);

impl From<Palette> for SerdePalette {
    fn from(_: Palette) -> Self {
        unimplemented!()
    }
}

// custom impl to work around https://github.com/serde-rs/serde/issues/1346
impl serde::Serialize for Palette {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let v = SerdePalette(
            self.entries
                .iter()
                .cloned()
                .map(|c| (c.red, c.green, c.blue, c.alpha))
                .collect(),
        );
        v.serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for Palette {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let v: SerdePalette = Deserialize::deserialize(deserializer)?;
        Ok(Palette {
            entries: v
                .0
                .into_iter()
                .map(|(r, g, b, a)| Color::new(r, g, b, a))
                .collect(),
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StyleCollection {
    pub style_sets: HashMap<String, StyleSet>,
    pub palettes: Vec<Palette>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct PaletteIndex(pub u32);

fn make_gradient_stop_collection(
    ctx: &DrawContext,
    colors: &[(f64, ColorRef)],
    palette: &Palette,
) -> GradientStopCollection {
    let colors: Vec<_> = colors
        .iter()
        .map(|(x, c)| (*x, palette.resolve(*c)))
        .collect();
    GradientStopCollection::new(
        ctx,
        &colors,
        ColorInterpolationMode::GammaCorrect,
        ExtendMode::Clamp,
    )
}

fn make_brush(
    ctx: &DrawContext,
    bounds: &Bounds,
    brush: &Brush,
    palette: &Palette,
) -> kyute_shell::drawing::Brush {
    match brush {
        Brush::SolidColor(colorref) => {
            drawing::Brush::new_solid_color(ctx, palette.resolve(*colorref))
        }
        Brush::Gradient {
            angle,
            ty: GradientType::Linear,
            stops,
            reverse,
        } => {
            let c = bounds.center();
            let w = bounds.width();
            let h = bounds.height();

            let tan_th = angle.radians.tan();
            let (x, y) = if tan_th > h / w {
                (h / (2.0 * tan_th), 0.5 * h)
            } else {
                (0.5 * w, 0.5 * w * tan_th)
            };

            let a = c + Offset::new(-x, y);
            let b = c + Offset::new(x, -y);

            let gradient_stops = make_gradient_stop_collection(ctx, stops, palette);
            drawing::Brush::new_linear_gradient(ctx, &gradient_stops, a, b, 1.0)
        }
        Brush::Gradient {
            angle,
            ty: GradientType::Radial,
            stops,
            reverse,
        } => {
            eprintln!("radial gradients not implemented");
            drawing::Brush::new_solid_color(ctx, palette.color(0))
        }
        _ => {
            eprintln!("brush type not supported");
            drawing::Brush::new_solid_color(ctx, palette.color(0))
        }
    }
}

impl StyleSet {
    fn draw(&self, platform: &Platform, ctx: &mut DrawContext, bounds: &Bounds, state: State, palette: &Palette) {
        let mut fill = None;
        let mut borders: Vec<Border> = Vec::new();
        let mut inner_shadow = None;
        let mut drop_shadow = None;
        let mut inner_glow = None;
        let mut outer_glow = None;

        for s in self.styles.iter() {
            if s.state_filter.value & s.state_filter.mask == state & s.state_filter.mask {
                fill = s.fill.clone();
                borders.extend(s.borders.iter().cloned());
                inner_shadow = s.inner_shadow.clone();
                drop_shadow = s.drop_shadow.clone();
                inner_glow = s.inner_glow.clone();
                outer_glow = s.outer_glow.clone();
            }
        }

        // we draw, in order:
        // - the drop shadow
        // - the fill
        // - the outer glow
        // - the inner glow
        // - the inner shadow
        // - the borders

        if drop_shadow.is_some()
            || inner_shadow.is_some()
            || outer_glow.is_some()
            || inner_glow.is_some()
        {
            // TODO
            eprintln!("WARNING: blurring effects not implemented");
        }

        let path_geometry = if let Shape::Path(ref s) = self.shape {
            Some(drawing::PathGeometry::try_from_svg_path(platform, s).expect("invalid SVG path"))
        } else {
            None
        };

        // fill
        if let Some(fill) = fill {
            let brush = make_brush(ctx, bounds, &fill, palette);
            match self.shape {
                Shape::Rect => {
                    ctx.fill_rectangle(*bounds, &brush);
                }
                Shape::RoundedRect(radius) => {
                    let radius = radius.to_dips(ctx);
                    ctx.fill_rounded_rectangle(*bounds, radius, radius, &brush);
                }
                Shape::Path(_) => {
                    ctx.fill_geometry(path_geometry.as_ref().unwrap(), &brush);
                }
                _ => {
                    eprintln!("Unsupported shape");
                }
            }
        }

        // borders
        for b in borders.iter() {
            let width = b.width.to_dips(ctx);
            let rect = match b.position {
                BorderPosition::Inside(x) => {
                    let x = -0.5 - x.to_dips(ctx);
                    bounds.inflate(x, x)
                }
                BorderPosition::Outside(x) => {
                    let x = 0.5 + x.to_dips(ctx);
                    bounds.inflate(x, x)
                }
                BorderPosition::Center => *bounds,
            };
            let brush = make_brush(ctx, &rect, &b.brush, palette);
            if b.blend_mode != BlendMode::Normal {
                eprintln!("unimplemented blend mode {:?}", b.blend_mode);
            }

            match self.shape {
                Shape::Rect => {
                    ctx.draw_rectangle(rect, &brush, width);
                }
                Shape::RoundedRect(radius) => {
                    let radius = radius.to_dips(ctx);
                    ctx.draw_rounded_rectangle(rect, radius, radius, &brush, width);
                }
                Shape::Path(_) => {
                    ctx.draw_geometry(path_geometry.as_ref().unwrap(), &brush, width);
                }
                _ => {
                    eprintln!("unimplemented shape {:?}", self.shape);
                }
            }
        }

        // the rest is unimplemented
    }
}

impl StyleCollection {
    pub fn draw(
        &self,
        platform: &Platform,
        ctx: &mut DrawContext,
        bounds: Bounds,
        style_set: &str,
        state_bits: State,
        palette: PaletteIndex,
    ) {
        // resolve the style set
        let style_set = if let Some(style_set) = self.style_sets.get(style_set) {
            style_set
        } else {
            eprintln!("Style set not found in collection: {}", style_set);
            return;
        };

        // resolve the palette
        let palette = if palette.0 as usize >= self.palettes.len() {
            eprintln!(
                "Invalid palette index (selected #{}, maximum is #{})",
                palette.0,
                self.palettes.len() - 1
            );
            &self.palettes[0]
        } else {
            &self.palettes[palette.0 as usize]
        };

        style_set.draw(platform, ctx, &bounds, state_bits, palette);
    }
}
