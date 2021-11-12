//! Drawing code for GUI elements.

use kyute_shell::drawing::{DrawContext, Color, GradientStopCollection, ColorInterpolationMode, ExtendMode};
use kyute_shell::drawing;
use crate::{EnvKey, Rect, Offset, SideOffsets};
use std::collections::HashMap;
use crate::env::Environment;
use std::sync::Arc;
use crate::data::Data;

/// Unit of length: device-independent pixel.
pub struct Dip;

/// A length in DIPs.
pub type DipLength = euclid::Length<f64, Dip>;
pub type Angle = euclid::Angle<f64>;

/// Length specification.
#[derive(Copy, Clone, Debug, PartialEq)]
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

/*// Provide our own impl for Serialize/Deserialize for anything that has a Color
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
            ColorRef::Palette(c) => SerdeColorRef::Palette { index: c.0 },
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
}*/

/// Describes the shape to draw in the box.
#[derive(Clone, Debug)]
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

/// Blend mode.
// TODO move this in kyute_shell::drawing
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum BlendMode {
    Normal,
    Multiply,
    Screen,
    Overlay,
}

/// Gradient type
// TODO move this in kyute_shell::drawing
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum GradientType {
    Linear,
    Radial,
}

/// Brush description
#[derive(Clone, Debug)]
pub enum Brush {
    /// Solid color
    SolidColor(Color),
    /// Gradient
    Gradient {
        angle: Angle,
        ty: GradientType,
        stops: Vec<(f64, Color)>,
        reverse: bool,
    },
    /// Image pattern
    Pattern {
        url: String,
    },
}

bitflags::bitflags! {
    //#[derive(Serialize, Deserialize)]
    pub struct State: u32 {
        const ACTIVE = 1<<0;
        const DISABLED = 1<<1;
        const HOVER = 1<<2;
        const FOCUS = 1<<3;
    }
}

#[derive(Copy, Clone, Debug)]
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
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum BorderPosition {
    Inside(Length),
    Center,
    Outside(Length),
}

#[derive(Clone, Debug)]
pub struct Border {
    pub opacity: f64,
    pub blend_mode: BlendMode,
    pub position: BorderPosition,
    pub width: Length,
    pub brush: Brush,
}

#[derive(Clone, Debug)]
pub struct Shadow {
    pub opacity: f64,
    pub blend_mode: BlendMode,
    pub color: Color,
    pub angle: Angle,
    pub distance: Length,
    pub spread: f64,
    pub size: Length,
}

#[derive(Clone, Debug)]
pub struct Glow {
    pub opacity: f64,
    pub blend_mode: BlendMode,
    pub color: Color,
    pub size: Length,
    pub choke: f64,
}

#[derive(Clone, Debug)]
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

#[derive(Clone, Debug)]
struct StyleSetImpl {
    content_padding: SideOffsets,
    shape: Shape,
    style: Vec<Style>,
}

#[derive(Clone, Debug)]
pub struct StyleSet(Arc<StyleSetImpl>);

impl Data for StyleSet {
    fn same(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0,&other.0)
    }
}

fn make_gradient_stop_collection(
    ctx: &DrawContext,
    colors: &[(f64, Color)],
) -> GradientStopCollection {
    let colors: Vec<_> = colors
        .iter()
        .map(|(x, c)| (*x, *c))
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
    bounds: &Rect,
    brush: &Brush,
) -> kyute_shell::drawing::Brush
{
    match brush {
        Brush::SolidColor(color) => {
            drawing::Brush::solid_color(ctx, *color)
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

            let gradient_stops = make_gradient_stop_collection(ctx, stops);
            drawing::Brush::linear_gradient(ctx, &gradient_stops, a, b, 1.0)
        }
        Brush::Gradient {
            angle,
            ty: GradientType::Radial,
            stops,
            reverse,
        } => {
            eprintln!("radial gradients not implemented");
            drawing::Brush::solid_color(ctx, Color::default())
        }
        _ => {
            eprintln!("brush type not supported");
            drawing::Brush::solid_color(ctx, Color::default())
        }
    }
}

pub struct StyleSetBuilder(StyleSetImpl);

impl StyleSetBuilder {
    pub fn with_content_padding(mut self, content_padding: SideOffsets) -> Self {
        self.0.content_padding = content_padding;
        self
    }

    pub fn with_shape(mut self, shape: Shape) -> Self {
        self.0.shape = shape;
        self
    }

    pub fn with_style(mut self, style: Style) -> Self {
        self.0.style.push(style);
        self
    }

    pub fn build(self) -> StyleSet {
        StyleSet(Arc::new(self.0))
    }
}

impl StyleSet {
    pub fn builder() -> StyleSetBuilder {
        StyleSetBuilder(StyleSetImpl {
            content_padding: Default::default(),
            shape: Shape::Rect,
            style: vec![]
        })
    }

    pub fn new() -> StyleSet {
        StyleSet(Arc::new(StyleSetImpl {
            content_padding: Default::default(),
            shape: Shape::Rect,
            style: vec![]
        }))
    }

    pub fn content_padding(&self) -> SideOffsets {
        self.0.content_padding
    }

    pub fn draw_box(
        &self,
        ctx: &mut DrawContext,
        bounds: &Rect,
        state: State,
    ) {
        let mut fill = None;
        let mut borders: Vec<Border> = Vec::new();
        let mut inner_shadow = None;
        let mut drop_shadow = None;
        let mut inner_glow = None;
        let mut outer_glow = None;

        for s in self.0.style.iter() {
            if s.state_filter.value & s.state_filter.mask == state & s.state_filter.mask {
                fill = s.fill.clone();
                borders.extend(s.borders.iter().cloned());
                inner_shadow = s.inner_shadow.clone();
                drop_shadow = s.drop_shadow.clone();
                inner_glow = s.inner_glow.clone();
                outer_glow = s.outer_glow.clone();
            }
        }

        ctx.save();
        ctx.transform(&bounds.origin.to_vector().to_transform());
        let bounds = Rect::from_size(bounds.size);

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
            tracing::warn!("shadow/glow effects not implemented");
        }

        let path_geometry = if let Shape::Path(ref s) = self.0.shape {
            Some(drawing::PathGeometry::try_from_svg_path(s).expect("invalid SVG path"))
        } else {
            None
        };

        // fill
        if let Some(fill) = fill {
            let brush = make_brush(ctx, &bounds, &fill);
            match self.0.shape {
                Shape::Rect => {
                    ctx.fill_rectangle(bounds, &brush);
                }
                Shape::RoundedRect(radius) => {
                    let radius = radius.to_dips(ctx);
                    ctx.fill_rounded_rectangle(bounds, radius, radius, &brush);
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
                BorderPosition::Center => bounds,
            };
            let brush = make_brush(ctx, &rect, &b.brush);
            if b.blend_mode != BlendMode::Normal {
                eprintln!("unimplemented blend mode {:?}", b.blend_mode);
            }

            match self.0.shape {
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
                    eprintln!("unimplemented shape {:?}", self.0.shape);
                }
            }
        }

        ctx.restore();
        // the rest is unimplemented
    }
}
