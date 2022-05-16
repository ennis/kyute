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
        /*if let Some(ref layer) = self.overlay_layer {
            layer
        } else {
            self.parent_layer
        }*/
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

    /*/// Returns a reference to the current skia painting canvas.
    pub fn canvas(&mut self) -> &mut skia_safe::Canvas {
        if let Some(ref mut paint_target) = self.paint_target {
            paint_target.canvas()
        } else {
            // create the paint target
            let surface = if self.needs_overlay {
                panic!("implicit layer creation is disabled for now");
                // `needs_overlay` flag is true, which means that all subsequent drawing operations must
                // happen on a separate layer above the "main" one (`self.parent_layer`).
                let overlay_layer = Layer::new();
                overlay_layer.set_size(SizeI::new(
                    self.clip_bounds.size.width as i32,
                    self.clip_bounds.size.height as i32,
                ));
                overlay_layer.set_transform(&self.layer_transform);
                let surface = overlay_layer.acquire_surface();
                //self.overlay_layer = Some(overlay_layer);
                surface
            } else {
            };

            let mut paint_target = PaintTarget::new(surface, self.direct_context.clone());
            let canvas = paint_target.canvas();
            canvas.scale((self.scale_factor as sk::scalar, self.scale_factor as sk::scalar));
            canvas.concat(&self.layer_transform.to_skia());
            self.paint_target.insert(paint_target).canvas()
        }
    }*/

    ///

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
        clip: Rect,
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
        canvas.clip_rect(clip.to_skia(), None, None);
        let result = f(self);
        self.surface.canvas().restore();
        self.bounds = prev_bounds;
        self.layer_transform = prev_layer_transform;
        result
    }

    /*/// Paint on a separate surface.
    pub fn surface<R>(&mut self, surface: &sk::Surface, mut f: impl FnMut(&mut PaintCtx) -> R) -> R {
        {
            let _span = trace_span!("PaintCtx paint surface").entered();
            let mut child_ctx = PaintCtx::new(
                surface.clone(),
                self.parent_layer,
                self.scale_factor,
                self.skia_direct_context,
            );
            f(&mut child_ctx)
        }
    }*/

    /*///
    pub fn draw_surface<R>(&mut self, surface: &sk::Surface) {

        self.surface.canvas().draw_drawable()

        {
            let _span = trace_span!("PaintCtx paint surface").entered();
            let mut child_ctx = PaintCtx::new(
                surface.clone(),
                self.parent_layer,
                self.scale_factor,
                self.skia_direct_context.clone(),
            );
            f(&mut child_ctx)
        }
    }*/

    /*/// Paint on a native composition layer.
    pub fn layer<R>(&mut self, layer: &Layer, mut f: impl FnMut(&mut PaintCtx) -> R) -> R {
        //self.finish();

        layer.remove_all_children();
        self.parent_layer.add_child(layer);
        layer.set_transform(&self.layer_transform);

        {
            let _span = trace_span!("PaintCtx paint layer").entered();
            let mut child_ctx = PaintCtx::new(layer, self.scale_factor, self.skia_direct_context.clone());
            let result = f(&mut child_ctx);
            child_ctx.finish();
            result
        }
    }*/

    /*/// Adds a layer as a child of the parent layer, without redrawing it.
    pub fn add_layer(&mut self, layer: &Layer) {
        //self.finish();
        self.parent_layer.add_child(layer);
        layer.set_transform(&self.layer_transform);
    }*/
}
