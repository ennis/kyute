use crate::{
    core::PaintDamage, drawing::ToSkia, graal, graal::vk::Handle, style::VisualState, Point, Rect, Size, SizeI,
    Transform,
};
use kyute_shell::{
    animation::{Layer, Surface},
    application::Application,
};
use skia_safe as sk;
use std::{fmt, fmt::Formatter};

// TODO:
// - remove layer_surface: it should only live in PaintCtx::layer; possibly turn `acquire_surface` into a callback-taking function
// - PaintCtx should operate on SkSurfaces
// - make surfaces the primary caching mechanism instead of native layers

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
    pub visual_state: VisualState,
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
            visual_state: Default::default(),
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

    /// Overrides the current visual state flags and calls the provided closure.
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
    pub fn with_visual_state<R>(&mut self, state: VisualState, f: impl FnOnce(&mut PaintCtx) -> R) -> R {
        let prev = self.visual_state;
        self.visual_state |= state;
        let result = f(self);
        self.visual_state = prev;
        result
    }

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
