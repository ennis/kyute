//! Drawing code for GUI elements.

use crate::{env::Environment, EnvKey, EnvValue, Offset, PaintCtx, Rect, SideOffsets};
use approx::ulps_eq;
use kyute_shell::{
    drawing::{Color, RectExt, ToSkia},
    skia as sk,
    skia::{gradient_shader::GradientShaderColors, BlendMode, PaintStyle::Stroke, RRect, Vector},
};
use std::str::FromStr;

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
pub trait IntoDip {
    fn dip(self) -> Length;
}

impl IntoDip for f32 {
    fn dip(self) -> Length {
        Length::Dip(self as f64)
    }
}

impl IntoDip for f64 {
    fn dip(self) -> Length {
        Length::Dip(self)
    }
}

/// Trait for values convertible to inches.
pub trait IntoInches {
    fn inch(self) -> Length;
}

impl IntoInches for i32 {
    fn inch(self) -> Length {
        Length::In(self as f64)
    }
}

impl IntoInches for f64 {
    fn inch(self) -> Length {
        Length::In(self)
    }
}

pub trait IntoAngle {
    fn degrees(self) -> Angle;
    fn radians(self) -> Angle;
}

impl IntoAngle for f32 {
    fn degrees(self) -> Angle {
        Angle::degrees(self as f64)
    }

    fn radians(self) -> Angle {
        Angle::radians(self as f64)
    }
}

impl IntoAngle for f64 {
    fn degrees(self) -> Angle {
        Angle::degrees(self)
    }

    fn radians(self) -> Angle {
        Angle::radians(self)
    }
}

//--------------------------------------------------------------------------------------------------

/// Either a value or a reference to a value in an environment.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ValueRef<T> {
    /// Inline value.
    Inline(T),
    /// Fetch the value from the environment.
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

//--------------------------------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct GradientStop {
    pos: Option<f64>,
    color: ValueRef<Color>,
}

/// Brushes.
pub enum Brush {
    SolidColor {
        color: ValueRef<Color>,
    },
    LinearGradient(LinearGradient),
    Image {
        // TODO
    },
}

impl Brush {
    pub fn to_sk_paint(&self, env: &Environment, bounds: Rect) -> sk::Paint {
        match self {
            Brush::SolidColor { color } => {
                let color = color.resolve(env).unwrap();
                let mut paint = sk::Paint::new(color.to_skia(), None);
                paint.set_anti_alias(true);
                paint
            }
            Brush::LinearGradient(linear_gradient) => {
                let c = bounds.center();
                let w = bounds.width();
                let h = bounds.height();

                let angle = linear_gradient.angle;
                let tan_th = angle.radians.tan();
                let (x, y) = if tan_th > h / w {
                    (h / (2.0 * tan_th), 0.5 * h)
                } else {
                    (0.5 * w, 0.5 * w * tan_th)
                };

                let a = c + Offset::new(-x, y);
                let b = c + Offset::new(x, -y);
                let a = sk::Point::new(a.x as sk::scalar, a.y as sk::scalar);
                let b = sk::Point::new(b.x as sk::scalar, b.y as sk::scalar);

                let mut positions = vec![0.0f32; linear_gradient.stops.len()];

                // resolve positions
                {
                    let mut i = 0;
                    let n = positions.len();
                    while i < n {
                        if let Some(pos) = linear_gradient.stops[i].pos {
                            positions[i] = pos as f32;
                            i += 1;
                        } else {
                            let prev = if i > 0 { positions[i - 1] as f64 } else { 0.0 };
                            // find the next non-empty stop position, return the number of stops between the current stop (i) and the next stop with non-empty position.
                            let (skip, next) = {
                                let mut skip = 1;
                                let mut pos = 1.0;
                                while i + skip <= n {
                                    if let Some(p) = linear_gradient.stops[i + skip].pos {
                                        pos = p;
                                        break;
                                    }
                                    skip += 1;
                                }
                                (skip, pos)
                            };

                            for j in 0..skip {
                                positions[i + j] =
                                    (prev + (next - prev) * j as f64 / skip as f64) as f32;
                            }
                            i += skip;
                        }
                    }
                }

                let colors: Vec<_> = linear_gradient
                    .stops
                    .iter()
                    .map(|stop| stop.color.resolve(env).unwrap().to_skia())
                    .collect();

                let shader = sk::Shader::linear_gradient(
                    (a, b),
                    GradientShaderColors::ColorsInSpace(&colors, sk::ColorSpace::new_srgb()),
                    &positions[..],
                    sk::TileMode::Clamp,
                    None,
                    None,
                )
                .unwrap();

                let mut paint = sk::Paint::default();
                paint.set_shader(shader);
                paint.set_anti_alias(true);
                paint
            }
            Brush::Image { .. } => {
                todo!("images")
            }
        }
    }
}

impl From<Color> for Brush {
    fn from(color: Color) -> Self {
        Brush::SolidColor {
            color: ValueRef::Inline(color),
        }
    }
}

impl From<EnvKey<Color>> for Brush {
    fn from(color: EnvKey<Color>) -> Self {
        Brush::SolidColor {
            color: ValueRef::Env(color),
        }
    }
}

#[derive(Clone, Debug)]
pub struct LinearGradient {
    angle: Angle,
    stops: Vec<GradientStop>,
}

impl LinearGradient {
    pub fn new() -> LinearGradient {
        LinearGradient {
            angle: Default::default(),
            stops: vec![],
        }
    }

    pub fn angle(mut self, angle: Angle) -> Self {
        self.angle = angle;
        self
    }

    pub fn stop(mut self, color: impl Into<ValueRef<Color>>, pos: impl Into<Option<f64>>) -> Self {
        self.stops.push(GradientStop {
            color: color.into(),
            pos: pos.into(),
        });
        self
    }
}

pub fn linear_gradient() -> LinearGradient {
    LinearGradient::new()
}

impl From<LinearGradient> for Brush {
    fn from(a: LinearGradient) -> Self {
        Brush::LinearGradient(a)
    }
}

/*//--------------------------------------------------------------------------------------------------
pub trait Modifier {
    fn draw(&self, ctx: &mut PaintCtx, bounds: Rect, shape: &Shape, env: &Environment);
}

pub struct ModifierChain<A, B>(A, B);

impl<A, B> Modifier for ModifierChain<A, B>
where
    A: Modifier,
    B: Modifier,
{
    fn draw(&self, ctx: &mut PaintCtx, bounds: Rect, shape: &Shape, env: &Environment) {
        self.0.draw(ctx, bounds, shape, env);
        self.1.draw(ctx, bounds, shape, env);
    }
}

pub struct NullModifier;

impl Modifier for NullModifier {
    fn draw(&self, _ctx: &mut PaintCtx, _bounds: Rect, _shape: &Shape, _env: &Environment) {}
}*/

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
        Vector::new(radii_dips[0] as sk::scalar, radii_dips[0] as sk::scalar),
        Vector::new(radii_dips[1] as sk::scalar, radii_dips[1] as sk::scalar),
        Vector::new(radii_dips[2] as sk::scalar, radii_dips[2] as sk::scalar),
        Vector::new(radii_dips[3] as sk::scalar, radii_dips[3] as sk::scalar),
    ]
}

//--------------------------------------------------------------------------------------------------
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct BoxShadowParams {
    offset_x: ValueRef<Length>,
    offset_y: ValueRef<Length>,
    blur_radius: ValueRef<Length>,
    spread_radius: ValueRef<Length>,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum BoxShadow {
    Drop(BoxShadowParams),
    Inset(BoxShadowParams),
}

//--------------------------------------------------------------------------------------------------

/// Border reference position
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum BorderPosition {
    Inside(ValueRef<Length>),
    Center,
    Outside(ValueRef<Length>),
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum BorderStyle {
    Solid,
    Dotted,
}

pub struct Border {
    /// Left,top,right,bottom border widths.
    widths: [ValueRef<Length>; 4],
    radii: [ValueRef<Length>; 4],
    /// Position of the border relative to the bounds.
    position: BorderPosition,
    brush: Brush,
    /// Border line style.
    style: BorderStyle,
    opacity: f64,
    blend_mode: BlendMode,
    enabled: bool,
}

impl Border {
    pub fn new(width: impl Into<ValueRef<Length>>) -> Border {
        let width = width.into();
        Border {
            widths: [width; 4],
            radii: [ValueRef::Inline(Length::Dip(0.0)); 4],
            position: BorderPosition::Center,
            brush: Brush::SolidColor {
                color: ValueRef::Inline(Color::new(0.0, 0.0, 0.0, 1.0)),
            },
            style: BorderStyle::Solid,
            opacity: 1.0,
            blend_mode: sk::BlendMode::SrcOver,
            enabled: true,
        }
    }

    pub fn inside(mut self, pos: impl Into<ValueRef<Length>>) -> Self {
        self.position = BorderPosition::Inside(pos.into());
        self
    }

    pub fn outside(mut self, pos: impl Into<ValueRef<Length>>) -> Self {
        self.position = BorderPosition::Outside(pos.into());
        self
    }

    pub fn center(mut self) -> Self {
        self.position = BorderPosition::Center;
        self
    }

    pub fn brush(mut self, brush: impl Into<Brush>) -> Self {
        self.brush = brush.into();
        self
    }

    pub fn opacity(mut self, opacity: f64) -> Self {
        self.opacity = opacity;
        self
    }

    pub fn blend(mut self, blend_mode: sk::BlendMode) -> Self {
        self.blend_mode = blend_mode;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    fn draw(&self, ctx: &mut PaintCtx, bounds: Rect, radii: [sk::Vector; 4], env: &Environment) {
        let mut paint = self.brush.to_sk_paint(env, bounds);
        paint.set_style(sk::PaintStyle::Stroke);
        paint.set_blend_mode(self.blend_mode);
        paint.set_alpha_f(self.opacity as sk::scalar);

        let widths = [
            self.widths[0]
                .resolve_or_default(env)
                .to_dips(ctx.scale_factor) as sk::scalar,
            self.widths[1]
                .resolve_or_default(env)
                .to_dips(ctx.scale_factor) as sk::scalar,
            self.widths[2]
                .resolve_or_default(env)
                .to_dips(ctx.scale_factor) as sk::scalar,
            self.widths[3]
                .resolve_or_default(env)
                .to_dips(ctx.scale_factor) as sk::scalar,
        ];
        let uniform_border = widths.iter().all(|&w| ulps_eq!(w, widths[0]));

        let rect = match self.position {
            BorderPosition::Inside(x) => {
                let x = -0.5 - x.resolve_or_default(env).to_dips(ctx.scale_factor);
                bounds.inflate(x, x)
            }
            BorderPosition::Outside(x) => {
                let x = 0.5 + x.resolve_or_default(env).to_dips(ctx.scale_factor);
                bounds.inflate(x, x)
            }
            BorderPosition::Center => bounds,
        };

        if !uniform_border {
            // draw lines, ignore radii
            // TODO border colors

            // left
            if !ulps_eq!(widths[0], 0.0) {
                paint.set_stroke_width(widths[0]);
                ctx.canvas.draw_line(
                    rect.top_left().to_skia(),
                    rect.bottom_left().to_skia(),
                    &paint,
                );
            }

            // top
            if !ulps_eq!(widths[1], 0.0) {
                paint.set_stroke_width(widths[1]);
                ctx.canvas.draw_line(
                    rect.top_left().to_skia(),
                    rect.top_right().to_skia(),
                    &paint,
                );
            }

            // right
            if !ulps_eq!(widths[2], 0.0) {
                paint.set_stroke_width(widths[2]);
                ctx.canvas.draw_line(
                    rect.top_right().to_skia(),
                    rect.bottom_right().to_skia(),
                    &paint,
                );
            }

            // bottom
            if !ulps_eq!(widths[3], 0.0) {
                paint.set_stroke_width(widths[3]);
                ctx.canvas.draw_line(
                    rect.bottom_left().to_skia(),
                    rect.bottom_right().to_skia(),
                    &paint,
                );
            }
        } else {
            if radii[0].is_zero() && radii[1].is_zero() && radii[2].is_zero() && radii[3].is_zero()
            {
                ctx.canvas.draw_rect(rect.to_skia(), &paint);
            } else {
                let rrect = RRect::new_rect_radii(rect.to_skia(), &radii);
                ctx.canvas.draw_rrect(rrect, &paint);
            }
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
    fn draw(&self, ctx: &mut PaintCtx, bounds: Rect, env: &Environment) {}
}

/// Style of a container.
pub struct Style<V> {
    // width/height: stretched, fixed, etc.
    // content alignment
    // baseline alignment
    pub baseline: Option<ValueRef<Length>>,
    // padding
    pub padding: [ValueRef<Length>; 4],
    // visual
    pub visual: V,
}

impl<V: Visual> Style<V> {
    /// Adds a visual to be drawn.
    pub fn visual<VN: Visual>(mut self, visual: VN) -> Style<(V, VN)> {
        Style {
            baseline: self.baseline,
            padding: self.padding,
            visual: (self.visual, visual),
        }
    }

    pub fn resolve_padding(&self, scale_factor: f64, env: &Environment) -> SideOffsets {
        SideOffsets::new(
            self.padding[0]
                .resolve_or_default(env)
                .to_dips(scale_factor),
            self.padding[1]
                .resolve_or_default(env)
                .to_dips(scale_factor),
            self.padding[2]
                .resolve_or_default(env)
                .to_dips(scale_factor),
            self.padding[3]
                .resolve_or_default(env)
                .to_dips(scale_factor),
        )
    }

    pub fn resolve_baseline(&self, scale_factor: f64, env: &Environment) -> Option<f64> {
        self.baseline
            .map(|x| x.resolve_or_default(env).to_dips(scale_factor))
    }
}

//--------------------------------------------------------------------------------------------------

/// Path visual.
pub struct Path {
    path: kyute_shell::drawing::Path,
    stroke: Option<Brush>,
    fill: Option<Brush>,
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
    pub fn fill(mut self, brush: impl Into<Brush>) -> Self {
        self.fill = Some(brush.into());
        self
    }

    /// Sets the brush used to stroke the path.
    pub fn stroke(mut self, brush: impl Into<Brush>) -> Self {
        self.fill = Some(brush.into());
        self
    }
}

impl Visual for Path {
    fn draw(&self, ctx: &mut PaintCtx, bounds: Rect, env: &Environment) {
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

        // TODO draw box shadow
    }
}

//--------------------------------------------------------------------------------------------------

/// Rectangle, possibly with rounded corners.
pub struct Rectangle {
    radii: [ValueRef<Length>; 4],
    fill: Option<Brush>,
    border: Option<Border>,
    box_shadow: Option<BoxShadow>,
}

impl Rectangle {
    /// Creates a new rectangle visual.
    pub fn new() -> Rectangle {
        Rectangle {
            radii: [ValueRef::Inline(Length::Dip(0.0)); 4],
            fill: None,
            border: None,
            box_shadow: None,
        }
    }

    /// Creates a new rectangle with rounded corners.
    pub fn new_rounded(radii: [ValueRef<Length>; 4]) -> Rectangle {
        Rectangle {
            radii,
            fill: None,
            border: None,
            box_shadow: None,
        }
    }

    /// Sets the brush used to fill the rectangle.
    pub fn fill(mut self, brush: impl Into<Brush>) -> Self {
        self.fill = Some(brush.into());
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
}

impl Visual for Rectangle {
    fn draw(&self, ctx: &mut PaintCtx, bounds: Rect, env: &Environment) {
        let radii = [
            self.radii[0].resolve_or_default(env),
            self.radii[1].resolve_or_default(env),
            self.radii[2].resolve_or_default(env),
            self.radii[3].resolve_or_default(env),
        ];
        let radii = radii_to_skia(ctx, &radii);

        // fill
        if let Some(ref brush) = self.fill {
            let mut paint = brush.to_sk_paint(env, ctx.bounds());
            paint.set_style(sk::PaintStyle::Fill);
            let rrect = RRect::new_rect_radii(bounds.to_skia(), &radii);
            ctx.canvas.draw_rrect(rrect, &paint);
        }

        // borders
        if let Some(ref border) = self.border {
            border.draw(ctx, bounds, radii, env);
        }

        // TODO draw box shadow
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
}

impl<'a> PaintCtxExt for PaintCtx<'a> {
    fn draw_visual<V: Visual>(&mut self, bounds: Rect, visual: &V, env: &Environment) {
        visual.draw(self, bounds, env)
    }
}
