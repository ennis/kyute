use crate::{
    cache,
    core::{DebugNode, LayerPaintCtx, PaintDamage},
    drawing::ToSkia,
    widget::prelude::*,
    Bloom, InternalEvent, LayoutParams, PointerEventKind, SizeI, WidgetFilter,
};
use kyute_common::{Color, RectExt};
use kyute_shell::animation::Layer;
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

/*fn paint_layer(
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
}*/

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
    cached_constraints: Cell<LayoutParams>,
    /// Cached layout result.
    layout_invalid: Cell<bool>,
    cached_layout: Cell<Option<Geometry>>,

    /// Inner widget
    content: T,
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Constructor impls
////////////////////////////////////////////////////////////////////////////////////////////////////

impl<T: Widget + 'static> WidgetPod<T> {
    /// Creates a new `WidgetPod` wrapping the specified widget.
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

    fn new_inner(widget: T, paint_target: PaintTarget) -> WidgetPod<T> {
        let id = widget.widget_id();
        WidgetPod {
            id,
            paint_target,
            transform: Cell::new(Default::default()),
            child_filter: Cell::new(None),
            paint_damage: Cell::new(PaintDamage::Repaint),
            cached_constraints: Cell::new(Default::default()),
            content: widget,
            cached_layout: Cell::new(None),
            layout_invalid: Cell::new(true),
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
        crate::core::send_utility_event(
            &self.content,
            &mut Event::Internal(InternalEvent::UpdateLayers { skia_direct_context }),
            &Environment::default(),
        );
    }

    pub(crate) fn repaint_layer(&self, skia_gpu_context: &mut sk::gpu::DirectContext) -> bool {
        if let PaintTarget::NativeLayer { ref layer } = self.paint_target {
            assert!(self.cached_layout.get().is_some(), "repaint called before layout");
            match self.paint_damage.replace(PaintDamage::None) {
                PaintDamage::Repaint => {
                    // straight recursive repaint
                    let _span = trace_span!("Repaint layer", id=?self.id).entered();
                    layer.remove_all_children();
                    let mut layer_paint_ctx = LayerPaintCtx { skia_gpu_context };
                    // use the scale factor we got from the last layout
                    self.content
                        .layer_paint(&mut layer_paint_ctx, layer, self.cached_constraints.get().scale_factor);
                    true
                }
                PaintDamage::SubLayers => {
                    let _span = trace_span!("Update layer", id=?self.id).entered();
                    self.update_child_layers(skia_gpu_context);
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

    fn layout(&self, ctx: &mut LayoutCtx, constraints: &LayoutParams, env: &Environment) -> Geometry {
        // we need to differentiate between two cases:
        // 1. we recalculated because the cached value has been invalidated because a child requested a relayout during eval
        // 2. we recalculated because constraints have changed
        //
        // If 2., then we can skip repaint if the resulting measurements are the same.

        if self.cached_constraints.get() == *constraints && !self.layout_invalid.get() {
            if let Some(layout) = self.cached_layout.get() {
                // same constraints & cached measurements still valid (no child widget requested a relayout) => skip layout & repaint
                trace!(
                    "[{:?}] WidgetPod returning cached layout ({:?})",
                    self.widget_id(),
                    layout
                );
                return layout;
            }
        }

        let name = self.debug_name();
        /*let _span = trace_span!("WidgetPod layout",
                    id = ?self.id,
                    name = name)
        .entered();*/

        // child layout

        trace!(
            "[{:?}] enter layout (speculative={:?})",
            self.widget_id(),
            ctx.speculative
        );

        let layout = self.content.layout(ctx, constraints, env);

        // also check for invalid size values while we're at it, but that's only for debugging convenience.
        if !layout.measurements.size.width.is_finite() || !layout.measurements.size.height.is_finite() {
            warn!(
                "layout[{:?}({})] returned non-finite measurements: {:?}",
                self.id, name, layout
            );
        }

        // if we are painting on our own layer OR surface, now we need to decide if we need to repaint it

        if !ctx.speculative {
            if self.cached_layout.get() != Some(layout) {
                // resize the underlying native layer or surface
                // TODO take bounds into account

                // size of the content box in physical pixels
                let size = SizeI::new(
                    (layout.measurements.size.width * ctx.scale_factor) as i32,
                    (layout.measurements.size.height * ctx.scale_factor) as i32,
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
                    warn!(
                        "empty layer or surface contents: {:?} ({:?})",
                        self.inner().widget_id(),
                        self.inner().debug_name()
                    );
                }
                self.paint_damage.set(PaintDamage::Repaint)
            }

            // update cached layout
            self.cached_constraints.set(*constraints);
            self.cached_layout.set(Some(layout));
            self.layout_invalid.set(false);
        }

        layout
    }

    fn route_event(&self, parent_ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        // WidgetPod plays an important role during event propagation:
        // First, it maintains a "child filter": a bloom filter containing the set of child widget IDs.
        // It uses this filter to stop propagation of routed events if the target is not in the child set.
        //
        // Second, it also handles hit-testing of pointer events,
        // and stops propagation to child widgets if the hit-test fails.

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
            // hit-test
            Event::Internal(InternalEvent::HitTest {
                ref mut position,
                ref mut hovered,
                ref mut hot,
            }) => {
                /*let local_pos = self.transform.get().inverse().unwrap().transform_point(*position);

                if !self
                    .cached_layout
                    .get()
                    .expect("hit test request received before layout")
                    .measurements
                    .local_bounds()
                    .contains(local_pos)
                {
                    trace!(
                        "InternalEvent::HitTest: FAIL @ {:?}{:?}",
                        self.content.debug_name(),
                        position
                    );
                    // the default value of the flag is true (hit-test is successful by default), so inhibit.
                    *hit = false;
                    return;
                } else {
                    // update position for descendants
                    *position = local_pos;
                    if let Some(id) = self.id {
                        hovered.insert(id);
                        **hot = Some(id);
                    }
                }*/
            }

            _ => {}
        }

        // continue with default routing behavior
        // hit-testing is done in the main `event` method so that `default_route_event` can do hover processing
        parent_ctx.default_route_event(self, event, &self.transform.get(), self.cached_layout.get(), env);
    }

    fn event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        match event {
            Event::Pointer(p)
                if p.kind == PointerEventKind::PointerUp
                    || p.kind == PointerEventKind::PointerDown
                    || p.kind == PointerEventKind::PointerMove =>
            {
                // pointer input events undergo hit-testing, with some exceptions: if the widget is a pointer-grabbing widget, don't hit test
                let exempt_from_hit_test = self.id.is_some() && ctx.pointer_capturing_widget() == self.id;

                if !exempt_from_hit_test {
                    if !self
                        .cached_layout
                        .get()
                        .expect("pointer event received before layout")
                        .measurements
                        .local_bounds()
                        .contains(p.position)
                    {
                        trace!(
                            "WidgetPod: pointer event FAIL @ {:?}{:?}",
                            self.content.debug_name(),
                            p.position,
                        );
                        ctx.hit_test_pass = false;
                        return;
                    }
                }
            }
            _ => {}
        }

        self.content.route_event(ctx, event, env);

        // handle event result
        if ctx.relayout {
            // a child widget (or ourselves) requested a relayout during event handling;
            // invalidate the cached layout, if any. However, don't clear the cached layout just yet,
            // because we may need it to handle additional pointer events that are delivered before a relayout can be done.
            // For example, it's possible for a child widget to receive a PointerOver event,
            // which causes it to request a relayout, and _at the same time_, a PointerEnter event,
            // which needs the cached layout in order to be delivered properly.

            //eprintln!("inner: {:?}, relayout requested", self.content.debug_name());
            self.layout_invalid.set(true);
        }

        // update damage
        let mut current_damage = self.paint_damage.get();
        current_damage.merge_up(ctx.paint_damage);
        self.paint_damage.set(current_damage);
        /*eprintln!(
            "inner:{:?}, incoming damage: {:?},  {:?} => {:?}",
            self.content.debug_name(),
            event_result.paint_damage,
            self.layer.as_ref().unwrap().size(),
            current_damage
        );*/

        // Downgrade `Repaint` to `SubLayers`:
        // if the contents of a layer need to be redrawn, its parent doesn't necessarily need to.
        // As such, a layered WidgetPod acts as a "repaint barrier".
        if ctx.paint_damage == PaintDamage::Repaint {
            ctx.paint_damage = PaintDamage::SubLayers;
        }
    }

    fn paint(&self, ctx: &mut PaintCtx) {
        let layout = self.cached_layout.get().expect("paint called before layout");

        match self.paint_target {
            PaintTarget::NativeLayer { ref layer } => {
                match self.paint_damage.replace(PaintDamage::None) {
                    PaintDamage::Repaint => {
                        // the contents of the layer are dirty
                        let mut layer_paint_ctx = LayerPaintCtx {
                            skia_gpu_context: ctx.skia_direct_context,
                        };
                        layer.remove_all_children();
                        self.content.layer_paint(&mut layer_paint_ctx, layer, ctx.scale_factor);
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
                        let mut child_ctx = PaintCtx::new(
                            &mut *surface,
                            ctx.parent_layer(),
                            ctx.scale_factor,
                            ctx.skia_direct_context,
                        );
                        child_ctx.surface.canvas().clear(sk::Color4f::new(0.0, 0.0, 0.0, 0.0));
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
                    layout.measurements.local_bounds(),
                    layout.measurements.clip_bounds,
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
                    layout.measurements.local_bounds(),
                    layout.measurements.clip_bounds,
                    |ctx| self.content.paint(ctx),
                )
            }
        }

        if ctx.debug {
            // print widgets ID in the top-right corner
            let mut font = sk::Font::default();
            font.set_size(9.0);
            let mut paint = sk::Paint::new(Color::from_hex("#FFFF00").to_skia(), None);
            paint.set_style(sk::PaintStyle::Fill);
            paint.set_blend_mode(sk::BlendMode::SrcOver);

            ctx.surface.canvas().draw_str_align(
                format!("{:?}", WidgetId::dbg_option(self.widget_id())),
                (layout.measurements.local_bounds().top_right() + Offset::new(0.0, 9.0)).to_skia(),
                &font,
                &paint,
                sk::utils::text_utils::Align::Right,
            );
        }
    }

    fn debug_node(&self) -> DebugNode {
        match self.paint_target {
            PaintTarget::NativeLayer { ref layer } => DebugNode::new(format!("native layer {:?} px", layer.size())),
            PaintTarget::Surface { ref surface } => DebugNode::new(format!("surface {:?} px", surface.size())),
            PaintTarget::ParentSurface => DebugNode::default(),
        }
    }
}

impl fmt::Debug for WidgetPod {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // TODO
        f.debug_tuple("WidgetPod").finish()
    }
}

impl<T: ?Sized> WidgetPod<T> {
    /// Returns a reference to the wrapped widgets.
    pub fn inner(&self) -> &T {
        &self.content
    }

    /// Returns a mutable reference to the wrapped widgets.
    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.content
    }

    /// Returns the widgets id.
    pub fn id(&self) -> Option<WidgetId> {
        self.id
    }
}
