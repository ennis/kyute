use crate::{
    cache, composable, drawing::ToSkia, graal, graal::vk::Handle, Data, PaintCtx, Point, Rect, Size, SizeI, Transform,
};
use kyute_common::Offset;
use kyute_shell::{
    animation::{CompositionLayer, CompositionSurface},
    application::Application,
};
use skia_safe as sk;
use std::{
    any::{Any, TypeId},
    borrow::Borrow,
    cell::{Cell, Ref, RefCell},
    sync::{Arc, Weak},
};

pub trait LayerDelegate: Any {
    fn draw(&self, ctx: &mut PaintCtx);
}

unsafe fn downcast_layer_delegate_unchecked<T: LayerDelegate>(delegate: &mut dyn LayerDelegate) -> &mut T {
    &mut *(delegate as *mut dyn LayerDelegate as *mut T)
}

struct LayerInner {
    /// Parent layer.
    parent: Weak<LayerImpl>,
    /// Optional surface backing the visual.
    surface: Option<CompositionSurface>,
    /// Child visuals.
    children: Vec<Layer>,
    /// Size of the visual in DIPs.
    size: Size,
    scale_factor: f64,
    /// Transform in parent
    transform: Transform,
    /// Draw callback
    delegate: Option<Box<dyn LayerDelegate>>,
    dirty: bool,
}

pub(crate) struct SurfaceUpdateCtx {
    recording_context: sk::gpu::DirectContext,
    scale_factor: f64,
}

struct LayerImpl {
    /// Backend composition layer.
    layer: CompositionLayer,
    /// Parent layer.
    parent: RefCell<Option<Weak<LayerImpl>>>,
    ///
    surface_backed: Cell<bool>,
    /// Optional surface backing the visual.
    surface: RefCell<Option<CompositionSurface>>,
    /// Child visuals.
    children: RefCell<Vec<Arc<LayerImpl>>>,
    /// Size of the visual in DIPs.
    size: Cell<Size>,
    scale_factor: Cell<f64>,
    /// Transform in parent
    transform: Cell<Transform>,
    /// Draw callback
    delegate: RefCell<Option<Box<dyn LayerDelegate>>>,
    ///
    dirty: Cell<bool>,
    /// Whether this layer must be redrawn from its delegate and non-surface-backed children.
    delegate_dirty: Cell<bool>,
}

impl LayerImpl {
    fn new(composition_layer: CompositionLayer, scale_factor: f64) -> LayerImpl {
        LayerImpl {
            layer: composition_layer,
            parent: Default::default(),
            surface_backed: Cell::new(false),
            surface: RefCell::new(None),
            children: RefCell::new(vec![]),
            size: Default::default(),
            scale_factor: Cell::new(scale_factor),
            transform: Default::default(),
            delegate: RefCell::new(None),
            dirty: Cell::new(false),
            delegate_dirty: Cell::new(false),
        }
    }

    fn set_surface_backed(&self, surface_backed: bool) {
        let was_surface_backed = self.surface_backed.replace(surface_backed);
        if !surface_backed {
            self.surface.replace(None);
        }
        if was_surface_backed != surface_backed {
            self.set_dirty();
            if was_surface_backed && !surface_backed {
                trace!(layer = self as *const _, "layer is now a draw layer");
            } else if !was_surface_backed && surface_backed {
                trace!(layer = self as *const _, "layer is now a surface-backed layer");
            }
        }
    }

    fn ensure_composition_surface(&self, scale_factor: f64) -> Ref<CompositionSurface> {
        {
            let mut surface = self.surface.borrow_mut();
            if surface.is_none() {
                let pixel_size = self.size.get() * scale_factor;
                let pixel_size_i = SizeI::new(pixel_size.width as i32, pixel_size.height as i32);
                trace!(
                    layer = self as *const _,
                    "allocated a composition surface of size {:?}",
                    pixel_size_i
                );
                *surface = Some(CompositionSurface::new(pixel_size_i));
            }
        }

        Ref::map(self.surface.borrow(), |r| r.as_ref().unwrap())
    }

    fn set_scale_factor(&self, scale_factor: f64) {
        let old_scale_factor = self.scale_factor.replace(scale_factor);
        if old_scale_factor != scale_factor {
            if self.surface_backed.get() {
                self.surface.replace(None);
            }
            self.set_dirty();
        }
    }

    /// Sets the size of this layer.
    ///
    /// If the layer has a backing surface, this surface is deleted and will be recreated.
    fn set_size(&self, size: Size) {
        let old_size = self.size.replace(size);
        if old_size != size {
            if self.surface_backed.get() {
                self.surface.replace(None);
            }
            self.set_dirty();
        }
    }

    fn set_dirty(&self) {
        let was_dirty = self.dirty.replace(true);
        if !was_dirty && !self.surface_backed.get() {
            if let Some(parent) = self.parent.borrow().upgrade() {
                parent.set_dirty()
            }
        }
    }

    /// Sets the delegate used to paint this layer.
    fn set_delegate(&self, delegate: impl LayerDelegate + 'static) {
        self.delegate.replace(Some(Box::new(delegate)));
        trace!(layer = self as *const _, "delegate changed (set_delegate)");
        self.delegate_dirty.set(true);
        self.set_dirty();
    }

    fn update_delegate<T: LayerDelegate + Default>(&self, update_fn: impl FnOnce(&mut T) -> bool) {
        let mut delegate = self.delegate.borrow_mut();
        let delegate = &mut *delegate;
        let changed = if let Some(ref mut delegate) = delegate {
            let delegate = &mut **delegate; // &mut Box<T> -> &mut <T>
            if delegate.type_id() == TypeId::of::<T>() {
                unsafe { update_fn(downcast_layer_delegate_unchecked(delegate)) }
            } else {
                let mut d = T::default();
                update_fn(&mut d);
                *delegate = Some(d);
                true
            }
        } else {
            let mut d = T::default();
            update_fn(&mut d);
            *delegate = Some(d);
            true
        };

        if changed {
            trace!(layer = self as *const _, "delegate changed");
            self.delegate_dirty.set(true);
            self.set_dirty();
        }
    }

    /// Sets the transform of the visual.
    fn set_transform(&self, transform: Transform) {
        let old_transform = self.transform.replace(transform);
        if old_transform != transform {
            trace!(layer = self as *const _, "transform changed");
            self.set_dirty();
        }
    }

    /// Adds a child layer.
    fn add_child(self: &Arc<Self>, layer: &Arc<LayerImpl>) {
        assert!(layer.parent.borrow().is_none(), "Layer already has a parent");

        let mut children = self.children.borrow_mut();
        if children.iter().find(|child| Arc::ptr_eq(child, layer)).is_some() {
            // already there
            return;
        }

        self.layer.add_child(&layer.layer);
        children.push(layer.clone());
        layer.parent.replace(Some(Arc::downgrade(self)));
        self.set_dirty();
    }

    /// Removes a child layer.
    fn remove_child(&self, layer: &Arc<LayerImpl>) {
        let mut children = self.children.borrow_mut();
        let pos = children
            .iter()
            .position(|child| Arc::ptr_eq(child, layer))
            .expect("Layer not a child");
        let child = children.remove(pos);
        child.parent.replace(None);
        self.set_dirty();
    }

    fn draw(&self, ctx: &mut PaintCtx) {
        let delegate = self.delegate.borrow();
        if let Some(delegate) = &*delegate {
            delegate.draw(ctx);
        }
        let children = self.children.borrow();
        for child in children.iter() {
            if !child.surface_backed.get() {
                let bounds = Rect::new(Point::origin(), child.size.get());
                ctx.with_transform_and_clip(child.transform.get(), bounds, bounds, |ctx| {
                    child.draw(ctx);
                })
            }
        }
    }

    fn update(&self, ctx: &SurfaceUpdateCtx, parent_transform: &Transform) {
        // skip if we're not dirty
        if !self.dirty.get() {
            return;
        }

        if self.surface_backed.get() {
            // only redraw if:
            // - the delegate has changed
            // - any non-surface-backed child is dirty
            let non_surface_backed_child_dirty = self
                .children
                .borrow()
                .iter()
                .any(|layer| layer.surface_backed.get() && layer.dirty.get());

            if non_surface_backed_child_dirty || self.delegate_dirty.get() {
                let surface = self.ensure_composition_surface(ctx.scale_factor);
                surface.draw(|_, target| {
                    let mut gr_ctx = Application::instance().lock_gpu_context();
                    let mut frame = gr_ctx.start_frame(Default::default());
                    let skia_image_usage_flags = graal::vk::ImageUsageFlags::COLOR_ATTACHMENT
                        | graal::vk::ImageUsageFlags::TRANSFER_SRC
                        | graal::vk::ImageUsageFlags::TRANSFER_DST;

                    // create the skia render pass
                    {
                        let mut ui_render_pass = frame.start_graphics_pass("UI render");
                        // FIXME we just assume how it's going to be used by skia
                        // register the access to the target image
                        ui_render_pass.add_image_dependency(
                            target.id,
                            graal::vk::AccessFlags::MEMORY_READ | graal::vk::AccessFlags::MEMORY_WRITE,
                            graal::vk::PipelineStageFlags::ALL_COMMANDS,
                            graal::vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                            graal::vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                        );

                        // draw callback
                        ui_render_pass.set_submit_callback(move |_cctx, _, _queue| {
                            // create skia BackendRenderTarget and Surface
                            let skia_image_info = sk::gpu::vk::ImageInfo {
                                image: target.handle.as_raw() as *mut _,
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
                            let render_target =
                                sk::gpu::BackendRenderTarget::new_vulkan((512 as i32, 512 as i32), 1, &skia_image_info);
                            let mut surface = sk::Surface::from_backend_render_target(
                                &mut recording_context,
                                &render_target,
                                sk::gpu::SurfaceOrigin::TopLeft,
                                sk::ColorType::RGBA8888, // TODO
                                sk::ColorSpace::new_srgb(),
                                Some(&sk::SurfaceProps::new(Default::default(), sk::PixelGeometry::RGBH)),
                            )
                            .unwrap();

                            let canvas = surface.canvas();
                            let mut ctx = PaintCtx {
                                canvas,
                                window_transform: Transform::identity(),
                                scale_factor,
                                invalid: &Default::default(),
                                bounds: Rect::new(Point::origin(), self.size.get()),
                            };
                            self.draw(&mut ctx);

                            surface.flush_and_submit();
                        });

                        ui_render_pass.finish();
                        frame.finish(&mut ());
                    }
                });
            }
        }

        let children = self.children.borrow();
        for layer in children.iter() {
            layer.update(ctx, parent_transform);
        }

        self.dirty.set(false);
        self.delegate_dirty.set(false);
    }
}

/// A visual element in the visual tree, possibly backed by a composition surface.
#[derive(Clone, Data)]
pub struct Layer(Arc<LayerImpl>);

impl Layer {
    #[composable]
    pub fn new() -> Layer {
        cache::state(|| Layer::new_from_composition_layer(CompositionLayer::new(), 1.0)).get()
    }

    /// Creates a new layer.
    pub(crate) fn new_from_composition_layer(composition_layer: CompositionLayer, scale_factor: f64) -> Layer {
        Layer(Arc::new(LayerImpl::new(composition_layer, scale_factor)))
    }

    /// Sets the delegate used to paint this layer.
    pub fn set_delegate(&self, delegate: impl LayerDelegate + 'static) {
        self.0.set_delegate(delegate);
    }

    pub fn update_delegate<T: LayerDelegate + Default>(&self, update_fn: impl FnOnce(&mut T) -> bool) {
        self.0.update_delegate(update_fn)
    }

    /// Sets the size of the visual.
    pub fn set_size(&self, size: Size) {
        self.0.set_size(size);
    }

    /// Returns the size of the visual.
    pub fn size(&self) -> Size {
        self.0.size.get()
    }

    /// Sets the scale factor.
    pub fn set_scale_factor(&self, scale_factor: f64) {
        self.0.set_scale_factor(scale_factor);
    }

    /// Sets the transform of the visual.
    pub fn set_transform(&self, transform: Transform) {
        self.0.set_transform(transform);
    }

    pub fn transform(&self) -> Transform {
        self.0.transform.get()
    }

    pub fn set_offset(&self, offset: Offset) {
        self.set_transform(offset.to_transform());
    }

    /// Adds a child visual.
    pub fn add_child(&self, layer: &Layer) {
        self.0.add_child(&layer.0)
    }

    /// Remove all child visuals
    pub fn remove_all(&self) {
        todo!()
    }

    pub fn remove_child(&self, layer: &Layer) {
        todo!()
    }

    /// Ensures that this visual is backed by a composition surface.
    pub fn set_surface_backed(&self, surface_backed: bool) {
        self.0.set_surface_backed(surface_backed)
    }

    /// Called internally.
    pub(crate) fn update(&self, ctx: &SurfaceUpdateCtx) {
        self.0.update(ctx, &Transform::identity())
    }
}
