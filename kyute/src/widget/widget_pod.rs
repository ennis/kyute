use crate::{
    application::{AppCtx, ExtEvent},
    cache,
    core::{DebugNode, FocusState, PaintDamage, WindowPaintCtx},
    graal,
    graal::vk::Handle,
    widget::prelude::*,
    Bloom, GpuFrameCtx, InternalEvent, PointerEventKind, WidgetFilter,
};
use kyute_common::SizeI;
use kyute_shell::{animation::Layer, application::Application, winit::event_loop::EventLoopWindowTarget};
use skia_safe as sk;
use std::{
    cell::{Cell, RefCell, RefMut},
    fmt,
    sync::Arc,
};

/*#[derive(Clone)]
pub struct CachedLayout {
    constraints: BoxConstraints,
    scale_factor: f64,
    layout: Option<Measurements>,
}*/

struct PaintSurface {
    sk_surface: RefCell<Option<sk::Surface>>,
    size: Cell<SizeI>,
}

impl PaintSurface {
    fn new() -> PaintSurface {
        PaintSurface {
            sk_surface: RefCell::new(None),
            size: Default::default(),
        }
    }

    fn resize(&self, new_size: SizeI) {
        if self.size.get() == new_size {
            return;
        }
        self.sk_surface.borrow_mut().take();
        self.size.set(new_size);
    }

    fn size(&self) -> SizeI {
        self.size.get()
    }

    fn sk_surface_mut(&self, sk_gpu_context: &mut sk::gpu::DirectContext) -> RefMut<sk::Surface> {
        let mut sk_surface = self.sk_surface.borrow_mut();
        if sk_surface.is_none() {
            let size = self.size.get();
            // TODO expose surface create params
            let s = sk::Surface::new_render_target(
                sk_gpu_context,
                sk::Budgeted::No,
                &sk::ImageInfo::new(
                    (size.width, size.height),
                    sk::ColorType::RGBA8888,
                    sk::AlphaType::Premul,
                    None,
                ),
                None,
                None,
                None,
                None,
            )
            .expect("failed to create skia surface");
            *sk_surface = Some(s);
        }
        RefMut::map(sk_surface, |s| s.as_mut().unwrap())
    }
}

/// Specifies where a WidgetPod will draw its content
enum PaintTarget {
    /// Paint on a native composition layer
    NativeLayer { layer: Layer },
    /// Paint on a skia surface
    Surface { surface: Arc<PaintSurface> },
    /// Paint on the parent layer / surface
    ParentSurface,
}

fn paint_layer(
    layer: &Layer,
    scale_factor: f64,
    skia_direct_context: &mut sk::gpu::DirectContext,
    f: impl FnOnce(&mut PaintCtx),
) {
    // acquire an native surface from the layer
    let layer_surface = layer.acquire_surface();
    let surface_image_info = layer_surface.image_info();
    let surface_size = layer_surface.size();

    // create the skia counterpart of the native surface (BackendRenderTarget and Surface)
    let skia_image_usage_flags = graal::vk::ImageUsageFlags::COLOR_ATTACHMENT
        | graal::vk::ImageUsageFlags::TRANSFER_SRC
        | graal::vk::ImageUsageFlags::TRANSFER_DST;
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
        skia_direct_context,
        &render_target,
        sk::gpu::SurfaceOrigin::TopLeft,
        sk::ColorType::RGBA8888, // TODO
        sk::ColorSpace::new_srgb(),
        Some(&sk::SurfaceProps::new(Default::default(), sk::PixelGeometry::RGBH)),
    )
    .unwrap();

    {
        let mut paint_ctx = PaintCtx::new(&mut surface, layer, scale_factor, skia_direct_context);
        f(&mut paint_ctx);
    }

    let _span = trace_span!("Flush skia surface").entered();
    let mut gr_ctx = Application::instance().lock_gpu_context();
    let mut frame = gr_ctx.start_frame(Default::default());
    let mut pass = frame.start_graphics_pass("UI render");
    // FIXME we just assume how it's going to be used by skia
    // register the access to the target image
    pass.add_image_dependency(
        layer_surface.image_info().id,
        graal::vk::AccessFlags::MEMORY_READ | graal::vk::AccessFlags::MEMORY_WRITE,
        graal::vk::PipelineStageFlags::ALL_COMMANDS,
        graal::vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        graal::vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
    );
    // draw callback
    pass.set_submit_callback(move |_cctx, _, _queue| {
        surface.flush_and_submit();
    });
    pass.finish();
    frame.finish(&mut ());
}

/// A container for a widget.
pub struct WidgetPod<T: ?Sized = dyn Widget> {
    /// Unique ID of the widget, if it has one.
    id: Option<WidgetId>,
    paint_target: PaintTarget,
    /// Transform.
    transform: Cell<Transform>,
    /// Bloom filter to filter child widgets.
    child_filter: Cell<Option<WidgetFilter>>,
    /// Paint damage done to the content of the widget pod.
    paint_damage: Cell<PaintDamage>,
    cached_constraints: Cell<BoxConstraints>,
    cached_scale_factor: Cell<f64>,
    /// Cached layout result.
    cached_measurements: Cell<Option<Measurements>>,

    /// Inner widget
    content: T,
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Constructor impls
////////////////////////////////////////////////////////////////////////////////////////////////////

impl<T: Widget + 'static> WidgetPod<T> {
    /// Creates a new `WidgetPod` wrapping the specified widget.
    #[composable]
    pub fn new(widget: T) -> WidgetPod<T> {
        Self::new_inner(widget, PaintTarget::ParentSurface)
    }

    /// Creates a new widgetpod backed by a native compositor layer.
    #[composable]
    pub fn with_native_layer(widget: T) -> WidgetPod<T> {
        let layer = cache::once(Layer::new);
        Self::new_inner(widget, PaintTarget::NativeLayer { layer })
    }

    /// Creates a new widgetpod backed by a surface object.
    #[composable]
    pub fn with_surface(widget: T) -> WidgetPod<T> {
        let surface = cache::once(|| Arc::new(PaintSurface::new()));
        Self::new_inner(widget, PaintTarget::Surface { surface })
    }

    #[composable]
    fn new_inner(widget: T, paint_target: PaintTarget) -> WidgetPod<T> {
        let id = widget.widget_id();
        WidgetPod {
            id,
            paint_target,
            transform: Cell::new(Default::default()),
            child_filter: Cell::new(None),
            paint_damage: Cell::new(PaintDamage::Repaint),
            cached_constraints: Cell::new(Default::default()),
            cached_scale_factor: Cell::new(0.0),
            content: widget,
            cached_measurements: Cell::new(None),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Methods
////////////////////////////////////////////////////////////////////////////////////////////////////

impl<T: Widget + ?Sized> WidgetPod<T> {
    /// Computes the child bloom filter.
    fn compute_child_filter(&self, parent_ctx: &mut EventCtx, env: &Environment) -> Bloom<WidgetId> {
        if let Some(filter) = self.child_filter.get() {
            // already computed
            filter
        } else {
            //tracing::trace!("computing child filter");
            let mut filter = Default::default();
            self.content.route_event(
                parent_ctx,
                &mut Event::Internal(InternalEvent::UpdateChildFilter { filter: &mut filter }),
                env,
            );
            self.child_filter.set(Some(filter));
            filter
        }
    }

    /// Returns whether this widget may contain the specified widget as a child (direct or not).
    fn may_contain(&self, widget: WidgetId) -> bool {
        if let Some(filter) = self.child_filter.get() {
            filter.may_contain(&widget)
        } else {
            warn!("`may_contain` called but child filter not initialized");
            true
        }
    }

    pub fn set_offset(&self, offset: Offset) {
        self.transform.set(offset.to_transform());
    }

    pub fn set_transform(&self, transform: Transform) {
        self.transform.set(transform)
    }

    pub fn transform(&self) -> Transform {
        self.transform.get()
    }

    /// Returns the layer.
    pub fn layer(&self) -> Option<&Layer> {
        if let PaintTarget::NativeLayer { ref layer } = self.paint_target {
            Some(layer)
        } else {
            None
        }
    }

    fn update_child_layers(&self, skia_direct_context: &mut sk::gpu::DirectContext) {
        // "skip" this layer's items and repaint internal layers
        let mut event_ctx = EventCtx::new();
        self.content.route_event(
            &mut event_ctx,
            &mut Event::Internal(InternalEvent::UpdateLayers { skia_direct_context }),
            &Environment::new(),
        );
    }

    pub(crate) fn repaint_layer(&self, skia_direct_context: &mut sk::gpu::DirectContext) -> bool {
        if let PaintTarget::NativeLayer { ref layer } = self.paint_target {
            assert!(self.cached_measurements.get().is_some(), "repaint called before layout");
            match self.paint_damage.replace(PaintDamage::None) {
                PaintDamage::Repaint => {
                    // straight recursive repaint
                    let _span = trace_span!("Repaint layer", id=?self.id).entered();
                    layer.remove_all_children();
                    paint_layer(layer, self.cached_scale_factor.get(), skia_direct_context, |ctx| {
                        ctx.surface.canvas().clear(sk::Color4f::new(0.0, 0.0, 0.0, 0.0));
                        self.content.paint(ctx);
                    });
                    true
                }
                PaintDamage::SubLayers => {
                    let _span = trace_span!("Update layer", id=?self.id).entered();
                    self.update_child_layers(skia_direct_context);
                    true
                }
                PaintDamage::None => false,
            }
        } else {
            warn!("repaint_layer called on non-layered WidgetPod");
            false
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Widget impl
////////////////////////////////////////////////////////////////////////////////////////////////////

impl<T: Widget + ?Sized> Widget for WidgetPod<T> {
    fn widget_id(&self) -> Option<WidgetId> {
        self.id
    }

    fn layout(&self, ctx: &mut LayoutCtx, constraints: BoxConstraints, env: &Environment) -> Measurements {
        // we need to differentiate between two cases:
        // 1. we recalculated because the cached value has been invalidated because a child requested a relayout during eval
        // 2. we recalculated because constraints have changed
        //
        // If 2., then we can skip repaint if the resulting measurements are the same.

        if self.cached_constraints.get() == constraints && self.cached_scale_factor.get() == ctx.scale_factor {
            if let Some(measurements) = self.cached_measurements.get() {
                // same constraints & cached measurements still valid (no child widget requested a relayout) => skip layout & repaint
                return measurements;
            }
        }

        let name = self.debug_name();
        let _span = trace_span!("WidgetPod layout", 
                    id = ?self.id,
                    name = name)
        .entered();

        // child layout
        let measurements = self.content.layout(ctx, constraints, env);

        // also check for invalid size values while we're at it, but that's only for debugging convenience.
        if !measurements.size.width.is_finite() || !measurements.size.height.is_finite() {
            warn!(
                "layout[{:?}({})] returned non-finite measurements: {:?}",
                self.id, name, measurements
            );
        }

        // if we are painting on our own layer OR surface, now we need to decide if we need to repaint it

        if !ctx.speculative {
            if self.cached_measurements.get() != Some(measurements) {
                let size = SizeI::new(
                    (measurements.clip_bounds.size.width * ctx.scale_factor) as i32,
                    (measurements.clip_bounds.size.height * ctx.scale_factor) as i32,
                );
                if !size.is_empty() {
                    match self.paint_target {
                        PaintTarget::NativeLayer { ref layer } => {
                            layer.set_size(size);
                        }
                        PaintTarget::Surface { ref surface } => {
                            surface.resize(size);
                        }
                        _ => {}
                    }
                } else {
                    warn!("empty layer or surface: {:?}", self.debug_name());
                }
                self.paint_damage.set(PaintDamage::Repaint)
            }

            // update cached layout
            self.cached_constraints.set(constraints);
            self.cached_scale_factor.set(ctx.scale_factor);
            self.cached_measurements.set(Some(measurements));
        }

        measurements
    }

    fn event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        self.content.route_event(ctx, event, env)
    }

    fn paint(&self, ctx: &mut PaintCtx) {
        let measurements = self.cached_measurements.get().expect("paint called before layout");

        match self.paint_target {
            PaintTarget::NativeLayer { ref layer } => {
                match self.paint_damage.replace(PaintDamage::None) {
                    PaintDamage::Repaint => {
                        // the contents of the layer are dirty
                        paint_layer(layer, ctx.scale_factor, &mut ctx.skia_direct_context, |ctx| {
                            ctx.surface.canvas().clear(sk::Color4f::new(0.0, 0.0, 0.0, 0.0));
                            self.content.paint(ctx);
                        });
                    }
                    PaintDamage::SubLayers => {
                        // this layer's contents are still valid, but some sublayers may need to be repainted.
                        self.update_child_layers(ctx.skia_direct_context);
                    }
                    PaintDamage::None => {}
                }
                ctx.parent_layer().add_child(layer);
                layer.set_transform(ctx.layer_transform());
            }
            PaintTarget::Surface { ref surface } => {
                // ...
                let mut surface = surface.sk_surface_mut(ctx.skia_direct_context);
                match self.paint_damage.replace(PaintDamage::None) {
                    PaintDamage::Repaint => {
                        // the contents of the surface are dirty
                        ctx.surface.canvas().clear(sk::Color4f::new(0.0, 0.0, 0.0, 0.0));
                        let mut child_ctx = PaintCtx::new(
                            &mut *surface,
                            ctx.parent_layer(),
                            ctx.scale_factor,
                            ctx.skia_direct_context,
                        );
                        self.content.paint(&mut child_ctx);
                    }
                    PaintDamage::SubLayers => {
                        // this surface's contents are still valid, but some child surfaces or layers may need to be repainted.
                        self.update_child_layers(ctx.skia_direct_context);
                    }
                    PaintDamage::None => {}
                }

                ctx.with_transform_and_clip(
                    &self.transform.get(),
                    measurements.local_bounds(),
                    measurements.clip_bounds,
                    |ctx| {
                        surface.draw(
                            ctx.surface.canvas(),
                            (0, 0),
                            sk::SamplingOptions::new(sk::FilterMode::Nearest, sk::MipmapMode::None),
                            None,
                        );
                    },
                )
            }
            PaintTarget::ParentSurface => {
                // --- Direct paint on parent surface ---
                ctx.with_transform_and_clip(
                    &self.transform.get(),
                    measurements.local_bounds(),
                    measurements.clip_bounds,
                    |ctx| self.content.paint(ctx),
                )
            }
        }
    }

    fn route_event(&self, parent_ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        // ensure that the child filter has been computed and the child widgets are initialized
        self.compute_child_filter(parent_ctx, env);
        match *event {
            // do not propagate routed events that are not directed to us, or to one of our children;
            // use the child filter to determine if we may contain a specific children; it might be a false
            // positive, but on average it saves some unnecessary traversals.
            Event::Internal(InternalEvent::RouteWindowEvent { target, .. })
            | Event::Internal(InternalEvent::RouteEvent { target, .. })
            | Event::Internal(InternalEvent::RoutePointerEvent { target, .. })
            | Event::Internal(InternalEvent::RouteRedrawRequest(target)) => {
                if Some(target) != self.id && !self.may_contain(target) {
                    return;
                }
            }
            // for UpdateChildFilter, if we already have computed and cached the child filter, use that
            // instead of propagating down the tree.
            Event::Internal(InternalEvent::UpdateChildFilter { ref mut filter }) => {
                if let Some(id) = self.id {
                    filter.add(&id);
                }
                let child_filter = self.compute_child_filter(parent_ctx, env);
                filter.extend(&child_filter);
                return;
            }

            // pointer events undergo hit-testing, with some exceptions:
            // - pointer out events are exempt from hit-test: if the pointer leaves
            // the parent widget, we also want the child elements to know that.
            // - if the widget is a pointer-grabbing widget, don't hit test
            Event::Pointer(p) => {
                let exempt_from_hit_test = p.kind == PointerEventKind::PointerOut
                    || (self.id.is_some() && parent_ctx.focus_state.as_deref().unwrap().pointer_grab == self.id);

                if !exempt_from_hit_test {
                    let local_pointer_pos = self.transform.get().inverse().unwrap().transform_point(p.position);

                    if !self
                        .cached_measurements
                        .get()
                        .expect("pointer event received before layout")
                        .local_bounds()
                        .contains(local_pointer_pos)
                    {
                        // hit test pass
                        trace!(
                            "do_event: pointer event FAIL @ {:?}{:?}",
                            self.content.debug_name(),
                            p.position,
                        );
                        return;
                    }
                }
            }

            _ => {}
        }

        // continue with default routing behavior
        let event_result =
            parent_ctx.default_route_event(self, event, &self.transform.get(), self.cached_measurements.get(), env);

        // handle event result
        if let Some(mut event_result) = event_result {
            if event_result.relayout {
                // a child widget (or ourselves) requested a relayout during event handling;
                // clear the cached layout, if any
                //eprintln!("inner: {:?}, relayout requested", self.content.debug_name());
                self.cached_measurements.set(None);
            }

            // update damage
            let mut current_damage = self.paint_damage.get();
            current_damage.merge_up(event_result.paint_damage);
            self.paint_damage.set(current_damage);
            /*eprintln!(
                "inner:{:?}, incoming damage: {:?},  {:?} => {:?}",
                self.content.debug_name(),
                event_result.paint_damage,
                self.layer.as_ref().unwrap().size(),
                current_damage
            );*/

            // We propagate the damage, but we downgrade `Repaint` to `SubLayers`:
            // if the contents of a layer need to be redrawn, its parent doesn't necessarily need to.
            // As such, a layered WidgetPod acts as a "repaint barrier".
            if event_result.paint_damage == PaintDamage::Repaint {
                event_result.paint_damage = PaintDamage::SubLayers;
            }

            parent_ctx.merge_event_result(event_result);
        }
    }

    fn debug_node(&self) -> DebugNode {
        DebugNode {
            content: match self.paint_target {
                PaintTarget::NativeLayer { ref layer } => Some(format!("native layer {:?} px", layer.size())),
                PaintTarget::Surface { ref surface } => Some(format!("surface {:?} px", surface.size())),
                PaintTarget::ParentSurface => None,
            },
        }
    }
}

impl fmt::Debug for WidgetPod {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // TODO
        f.debug_tuple("WidgetPod").finish()
    }
}

impl<T: ?Sized + Widget> WidgetPod<T> {
    /// Returns a reference to the wrapped widget.
    pub fn inner(&self) -> &T {
        &self.content
    }

    /// Returns a mutable reference to the wrapped widget.
    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.content
    }

    /// Returns the widget id.
    pub fn id(&self) -> Option<WidgetId> {
        self.id
    }
}
