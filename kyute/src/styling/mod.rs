//! Drawing code for GUI elements.

use crate::{env::Environment, EnvKey, EnvValue, Offset, PaintCtx, Rect};
use approx::ulps_eq;
use kyute_shell::{
    drawing::{Color, Path, RectExt, ToSkia},
    skia as sk,
    skia::{gradient_shader::GradientShaderColors, BlendMode, PaintStyle::Stroke, RRect, Vector},
};

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

    pub fn to_dips(self, ctx: &PaintCtx) -> f64 {
        match self {
            Length::Px(x) => x / ctx.scale_factor,
            Length::In(x) => 96.0 * x,
            Length::Dip(x) => x,
        }
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
#[derive(Copy, Clone, Debug)]
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

//--------------------------------------------------------------------------------------------------
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
}

//--------------------------------------------------------------------------------------------------
/// Fill modifier.
///
/// Fills a shape with a brush.
pub struct Fill {
    brush: Brush,
    enabled: bool,
}

impl Fill {
    pub fn enabled(mut self, enabled: bool) -> Fill {
        self.enabled = enabled;
        self
    }
}

/// Creates a fill modifier.
pub fn fill(brush: impl Into<Brush>) -> Fill {
    Fill {
        brush: brush.into(),
        enabled: true,
    }
}

fn radii_to_skia(ctx: &mut PaintCtx, radii: &[Length; 4]) -> [sk::Vector; 4] {
    let radii_dips = [
        radii[0].to_dips(ctx),
        radii[1].to_dips(ctx),
        radii[2].to_dips(ctx),
        radii[3].to_dips(ctx),
    ];

    // TODO x,y radii
    [
        Vector::new(radii_dips[0] as sk::scalar, radii_dips[0] as sk::scalar),
        Vector::new(radii_dips[1] as sk::scalar, radii_dips[1] as sk::scalar),
        Vector::new(radii_dips[2] as sk::scalar, radii_dips[2] as sk::scalar),
        Vector::new(radii_dips[3] as sk::scalar, radii_dips[3] as sk::scalar),
    ]
}

impl Modifier for Fill {
    fn draw(&self, ctx: &mut PaintCtx, bounds: Rect, shape: &Shape, env: &Environment) {
        if !self.enabled {
            return;
        }

        let mut paint = self.brush.to_sk_paint(env, ctx.bounds());
        paint.set_style(sk::PaintStyle::Fill);

        match shape {
            Shape::Path(path) => {
                let sk_path = path.to_skia();
                ctx.canvas.save();
                ctx.canvas.translate(bounds.top_left().to_skia());
                ctx.canvas.draw_path(&sk_path, &paint);
                ctx.canvas.restore();
            }
            Shape::RoundedRect { radii } => {
                let rrect = RRect::new_rect_radii(bounds.to_skia(), &radii_to_skia(ctx, radii));
                ctx.canvas.draw_rrect(rrect, &paint);
            }
        }
    }
}

//--------------------------------------------------------------------------------------------------
/// Border reference position
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum BorderPosition {
    Inside(Length),
    Center,
    Outside(Length),
}

pub struct Border {
    /// Left,top,right,bottom border widths.
    widths: [Length; 4],
    position: BorderPosition,
    brush: Brush,
    opacity: f64,
    blend_mode: BlendMode,
    enabled: bool,
}

impl Border {
    pub fn inside(mut self, pos: impl Into<Length>) -> Self {
        self.position = BorderPosition::Inside(pos.into());
        self
    }

    pub fn outside(mut self, pos: impl Into<Length>) -> Self {
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
}

pub fn border(widths: impl CornerLengths) -> Border {
    Border {
        widths: widths.into_corner_lengths(),
        position: BorderPosition::Center,
        brush: Brush::SolidColor {
            color: ValueRef::Inline(Color::new(0.0, 0.0, 0.0, 1.0)),
        },
        opacity: 1.0,
        blend_mode: sk::BlendMode::SrcOver,
        enabled: true,
    }
}

impl Modifier for Border {
    fn draw(&self, ctx: &mut PaintCtx, bounds: Rect, shape: &Shape, env: &Environment) {
        if !self.enabled {
            return;
        }

        let mut paint = self.brush.to_sk_paint(env, bounds);
        paint.set_style(sk::PaintStyle::Stroke);
        paint.set_blend_mode(self.blend_mode);
        paint.set_alpha_f(self.opacity as sk::scalar);

        let widths = [
            self.widths[0].to_dips(ctx) as sk::scalar,
            self.widths[1].to_dips(ctx) as sk::scalar,
            self.widths[2].to_dips(ctx) as sk::scalar,
            self.widths[3].to_dips(ctx) as sk::scalar,
        ];
        let uniform_border = widths.iter().all(|&w| ulps_eq!(w, widths[0]));

        let rect = match self.position {
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

        match shape {
            Shape::Path(path) => {
                // just stroke the path?
                let sk_path = path.to_skia();
                ctx.canvas.save();
                ctx.canvas.translate(bounds.top_left().to_skia());
                ctx.canvas.draw_path(&sk_path, &paint);
                ctx.canvas.restore();
            }
            Shape::RoundedRect { radii } => {
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
                    let radii = radii_to_skia(ctx, radii);
                    if radii[0].is_zero()
                        && radii[1].is_zero()
                        && radii[2].is_zero()
                        && radii[3].is_zero()
                    {
                        ctx.canvas.draw_rect(rect.to_skia(), &paint);
                    } else {
                        let rrect = RRect::new_rect_radii(rect.to_skia(), &radii);
                        ctx.canvas.draw_rrect(rrect, &paint);
                    }
                }
            }
        }
    }
}

//--------------------------------------------------------------------------------------------------
pub enum Shape {
    Path(Path),
    RoundedRect { radii: [Length; 4] },
}

//--------------------------------------------------------------------------------------------------
pub struct DrawItem<M> {
    shape: Shape,
    modifiers: M,
}

impl<M> DrawItem<M> {
    pub fn with<M2: Modifier>(self, next_modifier: M2) -> DrawItem<ModifierChain<M, M2>> {
        DrawItem {
            modifiers: ModifierChain(self.modifiers, next_modifier),
            shape: self.shape,
        }
    }
}

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

pub fn rounded_rectangle(radii: impl CornerLengths) -> DrawItem<NullModifier> {
    DrawItem {
        shape: Shape::RoundedRect {
            radii: radii.into_corner_lengths(),
        },
        modifiers: NullModifier,
    }
}

pub fn rectangle() -> DrawItem<NullModifier> {
    rounded_rectangle(0.0)
}

pub fn path(path: Path) -> DrawItem<NullModifier> {
    DrawItem {
        shape: Shape::Path(path),
        modifiers: NullModifier,
    }
}

//--------------------------------------------------------------------------------------------------
pub trait PaintCtxExt {
    fn draw_styled_box<M: Modifier>(&mut self, bounds: Rect, item: DrawItem<M>, env: &Environment);
}

impl<'a> PaintCtxExt for PaintCtx<'a> {
    fn draw_styled_box<M: Modifier>(&mut self, bounds: Rect, item: DrawItem<M>, env: &Environment) {
        let shape = &item.shape;
        item.modifiers.draw(self, bounds, shape, env);
    }
}
