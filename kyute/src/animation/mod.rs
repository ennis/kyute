use crate::{drawing::ToSkia, graal, graal::vk::Handle, Rect, Transform};
use kyute_common::SizeI;
use kyute_shell::{
    animation::{Layer, Surface},
    application::Application,
};
use skia_safe as sk;
use std::{fmt, fmt::Formatter};

/// Painting context passed to `LayerDelegate::draw`.
pub struct PaintCtx<'a> {
    parent_layer: &'a Layer,
    layer_transform: Transform,
    //pub overlay_layer: Option<Layer>,
    layer_surface: Surface,
    pub skia_direct_context: sk::gpu::DirectContext,
    finished: bool,
    pub surface: sk::Surface,
    pub scale_factor: f64,
    pub bounds: Rect,
    pub clip_bounds: Rect,
}

impl<'a> fmt::Debug for PaintCtx<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("PaintCtx").finish_non_exhaustive()
    }
}

impl<'a> PaintCtx<'a> {
    pub fn new(layer: &'a Layer, scale_factor: f64, mut skia_direct_context: sk::gpu::DirectContext) -> PaintCtx<'a> {
        let layer_surface = layer.acquire_surface();
        let surface_image_info = layer_surface.image_info();
        let surface_size = layer_surface.size();
        let skia_image_usage_flags = graal::vk::ImageUsageFlags::COLOR_ATTACHMENT
            | graal::vk::ImageUsageFlags::TRANSFER_SRC
            | graal::vk::ImageUsageFlags::TRANSFER_DST;
        // create skia BackendRenderTarget and Surface
        let skia_image_info = sk::gpu::vk::ImageInfo {
            image: surface_image_info.handle.as_raw() as *mut _,
            alloc: Default::default(),
            tiling: sk::gpu::vk::ImageTiling::OPTIMAL,
            layout: sk::gpu::vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            format: sk::gpu::vk::Format::R8G8B8A8_UNORM, // TODO
            image_usage_flags: skia_image_usage_flags.as_raw(),
            sample_count: 1,
            level_count: 1,
            current_queue_family: sk::gpu::vk::QUEUE_FAMILY_IGNORED,
            protected: sk::gpu::Protected::No,
            ycbcr_conversion_info: Default::default(),
            sharing_mode: sk::gpu::vk::SharingMode::EXCLUSIVE,
        };
        let render_target = sk::gpu::BackendRenderTarget::new_vulkan(
            (surface_size.width as i32, surface_size.height as i32),
            1,
            &skia_image_info,
        );
        let mut surface = sk::Surface::from_backend_render_target(
            &mut skia_direct_context,
            &render_target,
            sk::gpu::SurfaceOrigin::TopLeft,
            sk::ColorType::RGBA8888, // TODO
            sk::ColorSpace::new_srgb(),
            Some(&sk::SurfaceProps::new(Default::default(), sk::PixelGeometry::RGBH)),
        )
        .unwrap();

        PaintCtx {
            parent_layer: layer,
            layer_transform: Transform::identity(),
            layer_surface,
            skia_direct_context,
            finished: false,
            surface,
            scale_factor,
            bounds: Default::default(),
            clip_bounds: Default::default(),
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
    pub fn finish(&mut self) {
        let mut gr_ctx = Application::instance().lock_gpu_context();
        let mut frame = gr_ctx.start_frame(Default::default());
        let mut pass = frame.start_graphics_pass("UI render");
        // FIXME we just assume how it's going to be used by skia
        // register the access to the target image
        pass.add_image_dependency(
            self.layer_surface.image_info().id,
            graal::vk::AccessFlags::MEMORY_READ | graal::vk::AccessFlags::MEMORY_WRITE,
            graal::vk::PipelineStageFlags::ALL_COMMANDS,
            graal::vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            graal::vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        );
        // draw callback
        pass.set_submit_callback(move |_cctx, _, _queue| {
            self.surface.flush_and_submit();
        });
        pass.finish();
        frame.finish(&mut ());
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

    /// Enters a layer.
    pub fn layer<R>(&mut self, layer: &Layer, mut f: impl FnMut(&mut PaintCtx) -> R) -> R {
        self.finish();
        self.finished = true;
        layer.remove_all_children();
        self.parent_layer.add_child(layer);

        let mut child_ctx = PaintCtx::new(layer, self.scale_factor, self.skia_direct_context.clone());
        let result = f(&mut child_ctx);
        child_ctx.finish();

        result
    }

    /// Adds a layer as a child of the parent layer, without redrawing it.
    pub fn add_layer(&mut self, layer: &Layer) {
        self.finish();
        self.finished = true;
        self.parent_layer.add_child(layer);
        self.parent_layer.set_transform(&self.layer_transform);
    }
}
