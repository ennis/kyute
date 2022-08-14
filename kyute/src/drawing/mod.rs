//! Drawing-related wrappers and helpers for use with skia.
use crate::{style::WidgetState, Color, Offset, Point, Rect, Size, Transform};
use kyute_shell::animation::Layer;
use skia_safe as sk;
use std::fmt;

mod border;
mod box_shadow;
mod image;
mod paint;
mod path;
mod svg_path;

use crate::application::AppCtx;
pub use border::{Border, BorderStyle};
pub use box_shadow::BoxShadow;
pub use image::{Image, ImageCache, IMAGE_CACHE};
pub use paint::{ColorStop, LinearGradient, Paint, RepeatMode, UniformData};
pub use path::Path;
pub(crate) use svg_path::svg_path_to_skia;

/// Types that can be converted to their skia equivalent.
pub trait ToSkia {
    type Target;
    fn to_skia(&self) -> Self::Target;
}

/// Types that can be converted from their skia equivalent.
pub trait FromSkia {
    type Source;
    fn from_skia(value: Self::Source) -> Self;
}

impl ToSkia for Rect {
    type Target = skia_safe::Rect;

    fn to_skia(&self) -> Self::Target {
        skia_safe::Rect {
            left: self.origin.x as f32,
            top: self.origin.y as f32,
            right: (self.origin.x + self.size.width) as f32,
            bottom: (self.origin.y + self.size.height) as f32,
        }
    }
}

impl FromSkia for Rect {
    type Source = skia_safe::Rect;

    fn from_skia(value: Self::Source) -> Self {
        Rect {
            origin: Point::new(value.left as f64, value.top as f64),
            size: Size::new((value.right - value.left) as f64, (value.bottom - value.top) as f64),
        }
    }
}

impl ToSkia for Point {
    type Target = skia_safe::Point;

    fn to_skia(&self) -> Self::Target {
        skia_safe::Point {
            x: self.x as f32,
            y: self.y as f32,
        }
    }
}

impl ToSkia for Offset {
    type Target = skia_safe::Vector;

    fn to_skia(&self) -> Self::Target {
        skia_safe::Vector {
            x: self.x as f32,
            y: self.y as f32,
        }
    }
}

impl FromSkia for Point {
    type Source = skia_safe::Point;

    fn from_skia(value: Self::Source) -> Self {
        Point::new(value.x as f64, value.y as f64)
    }
}

impl ToSkia for Color {
    type Target = sk::Color4f;

    fn to_skia(&self) -> Self::Target {
        let (r, g, b, a) = self.to_rgba();
        skia_safe::Color4f { r, g, b, a }
    }
}

impl FromSkia for Color {
    type Source = skia_safe::Color4f;

    fn from_skia(value: Self::Source) -> Self {
        Color::new(value.r, value.g, value.b, value.a)
    }
}

impl ToSkia for Transform {
    type Target = sk::Matrix;

    fn to_skia(&self) -> Self::Target {
        sk::Matrix::new_all(
            self.m11 as sk::scalar,
            self.m21 as sk::scalar,
            self.m31 as sk::scalar,
            self.m12 as sk::scalar,
            self.m22 as sk::scalar,
            self.m32 as sk::scalar,
            0.0,
            0.0,
            1.0,
        )
    }
}

//--------------------------------------------------------------------------------------------------

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

////////////////////////////////////////////////////////////////////////////////////////////////////
// Shapes
////////////////////////////////////////////////////////////////////////////////////////////////////

fn radii_to_skia(radii: &[Offset; 4]) -> [sk::Vector; 4] {
    [
        sk::Vector::new(radii[0].x as sk::scalar, radii[0].y as sk::scalar),
        sk::Vector::new(radii[1].x as sk::scalar, radii[1].y as sk::scalar),
        sk::Vector::new(radii[2].x as sk::scalar, radii[2].y as sk::scalar),
        sk::Vector::new(radii[3].x as sk::scalar, radii[3].y as sk::scalar),
    ]
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct RoundedRect {
    pub rect: Rect,
    pub radii: [Offset; 4],
}

impl Default for RoundedRect {
    fn default() -> Self {
        RoundedRect {
            rect: Rect::default(),
            radii: [Offset::zero(); 4],
        }
    }
}

impl ToSkia for RoundedRect {
    type Target = skia_safe::RRect;

    fn to_skia(&self) -> Self::Target {
        if self.is_rounded() {
            sk::RRect::new_rect_radii(self.rect.to_skia(), &radii_to_skia(&self.radii))
        } else {
            sk::RRect::new_rect(self.rect.to_skia())
        }
    }
}

impl RoundedRect {
    pub fn translate(&self, offset: Offset) -> RoundedRect {
        RoundedRect {
            rect: self.rect.translate(offset),
            ..*self
        }
    }

    pub fn is_rounded(&self) -> bool {
        !(self.radii[0].x == 0.0
            && self.radii[1].x == 0.0
            && self.radii[2].x == 0.0
            && self.radii[3].x == 0.0
            && self.radii[0].y == 0.0
            && self.radii[1].y == 0.0
            && self.radii[2].y == 0.0
            && self.radii[3].y == 0.0)
    }

    pub fn inset(&self, dx: f64, dy: f64) -> RoundedRect {
        let rect = self.rect.inflate(-dx, -dy);
        let radii = [
            Offset::new((self.radii[0].x - dx).max(0.0), (self.radii[0].y - dy).max(0.0)),
            Offset::new((self.radii[1].x - dx).max(0.0), (self.radii[1].y - dy).max(0.0)),
            Offset::new((self.radii[2].x - dx).max(0.0), (self.radii[2].y - dy).max(0.0)),
            Offset::new((self.radii[3].x - dx).max(0.0), (self.radii[3].y - dy).max(0.0)),
        ];

        RoundedRect { rect, radii }
    }

    pub fn outset(&self, dx: f64, dy: f64) -> RoundedRect {
        self.inset(-dx, -dy)
    }

    pub fn contract(&self, widths: [f64; 4]) -> RoundedRect {
        let [t, r, b, l] = widths;
        let inset_x = 0.5 * (l + r);
        let offset_x = 0.5 * (l - r);
        let inset_y = 0.5 * (t + b);
        let offset_y = 0.5 * (t - b);
        self.translate(Offset::new(offset_x, offset_y)).inset(inset_x, inset_y)
    }
}

impl From<Rect> for RoundedRect {
    fn from(rect: Rect) -> Self {
        RoundedRect {
            rect,
            radii: [Offset::zero(), Offset::zero(), Offset::zero(), Offset::zero()],
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Shape {
    RoundedRect(RoundedRect),
}

impl Default for Shape {
    fn default() -> Self {
        Shape::RoundedRect(Default::default())
    }
}

impl Shape {
    pub fn fill(&self, ctx: &mut PaintCtx, paint: &Paint) {
        match self {
            Shape::RoundedRect(rrect) => {
                let mut paint = paint.to_sk_paint(rrect.rect);
                paint.set_style(sk::PaintStyle::Fill);
                ctx.surface.canvas().draw_rrect(rrect.to_skia(), &paint);
            }
        }
    }
}

impl From<Rect> for Shape {
    fn from(rect: Rect) -> Self {
        Shape::RoundedRect(RoundedRect {
            rect,
            radii: [Offset::zero(), Offset::zero(), Offset::zero(), Offset::zero()],
        })
    }
}

impl From<RoundedRect> for Shape {
    fn from(rrect: RoundedRect) -> Self {
        Shape::RoundedRect(rrect)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// PaintCtx
////////////////////////////////////////////////////////////////////////////////////////////////////

/// Painting context passed to `LayerDelegate::draw`.
pub struct PaintCtx<'a> {
    /// Parent native composition layer.
    parent_layer: &'a Layer,
    /// Transform to parent_layer.
    layer_transform: Transform,
    pub skia_direct_context: &'a mut sk::gpu::DirectContext,
    finished: bool,
    pub surface: &'a mut sk::Surface,
    pub scale_factor: f64,
    pub bounds: Rect,
    pub clip_bounds: Rect,
    pub(crate) debug: bool,
}

impl<'a> fmt::Debug for PaintCtx<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("PaintCtx").finish_non_exhaustive()
    }
}

impl<'a> PaintCtx<'a> {
    /// Creates a PaintCtx to draw on the specified surface.
    pub fn new(
        surface: &'a mut sk::Surface,
        parent_layer: &'a Layer,
        scale_factor: f64,
        skia_direct_context: &'a mut sk::gpu::DirectContext,
    ) -> PaintCtx<'a> {
        let width = parent_layer.size().width as f64 / scale_factor;
        let height = parent_layer.size().height as f64 / scale_factor;
        let bounds = Rect::new(Point::origin(), Size::new(width, height));
        PaintCtx {
            parent_layer,
            layer_transform: Transform::identity(),
            skia_direct_context,
            finished: false,
            surface,
            scale_factor,
            bounds,
            clip_bounds: bounds,
            debug: true,
        }
    }

    /// Returns the transform to the parent layer's coordinate space.
    pub fn layer_transform(&self) -> &Transform {
        &self.layer_transform
    }

    /// Returns the parent layer.
    pub fn parent_layer(&self) -> &'a Layer {
        self.parent_layer
    }

    /// Enable visual debugging information
    pub fn set_debug(&mut self, enabled: bool) {
        self.debug = enabled;
    }

    /*/// Overrides the current visual state flags and calls the provided closure.
    ///
    /// # Examples
    ///
    /// ```
    /// use kyute::PaintCtx;
    /// use kyute::style::VisualState;
    /// use kyute::widget::Button;
    ///
    /// fn paint_disabled_button(ctx: &mut PaintCtx, button: &Button) {
    ///     ctx.with_visual_state(VisualState::DISABLED, |ctx| button.paint(ctx));
    /// }
    /// ```
    pub fn with_visual_state<R>(&mut self, state: WidgetState, f: impl FnOnce(&mut PaintCtx) -> R) -> R {
        let prev = self.visual_state;
        self.visual_state |= state;
        let result = f(self);
        self.visual_state = prev;
        result
    }*/

    /// Calls the specified closure with a copy of the current painting context, with the specified
    /// transform and clip bounds applied.
    ///
    /// # Arguments
    /// - `transform` the transform to apply
    /// - `bounds` the bounds of the inner element (`PaintCtx::bounds`). This does not affect painting.
    /// - `clip` clipping rectangle to apply
    /// - `f` the closure to call with the modified painting context
    pub fn with_transform_and_clip<R>(
        &mut self,
        transform: &Transform,
        bounds: Rect,
        clip: Option<Rect>,
        f: impl FnOnce(&mut PaintCtx) -> R,
    ) -> R {
        let prev_layer_transform = self.layer_transform;
        let prev_bounds = self.bounds;
        self.layer_transform = transform.then(&self.layer_transform);
        self.bounds = bounds;
        let canvas = self.surface.canvas();
        let scale_factor = self.scale_factor as sk::scalar;
        canvas.save();
        canvas.reset_matrix();
        canvas.scale((scale_factor, scale_factor));
        canvas.concat(&self.layer_transform.to_skia());
        if let Some(clip) = clip {
            canvas.clip_rect(clip.to_skia(), None, None);
        }
        let result = f(self);
        self.surface.canvas().restore();
        self.bounds = prev_bounds;
        self.layer_transform = prev_layer_transform;
        result
    }
}

//--------------------------------------------------------------------------------------------------
pub trait PaintCtxExt {
    fn draw_box_shadow(&mut self, shape: &Shape, box_shadow: &BoxShadow);
    fn draw_border(&mut self, shape: &Shape, border: &Border);
    fn fill_shape(&mut self, shape: &Shape, paint: &Paint);
}

impl<'a> PaintCtxExt for PaintCtx<'a> {
    fn draw_box_shadow(&mut self, shape: &Shape, box_shadow: &BoxShadow) {
        box_shadow.draw(self, shape);
    }

    fn draw_border(&mut self, shape: &Shape, border: &Border) {
        border.draw(self, shape);
    }

    fn fill_shape(&mut self, shape: &Shape, paint: &Paint) {
        shape.fill(self, paint);
    }
}
