use crate::{
    application::{AppCtx, ExtEvent},
    cache,
    core::{DebugNode, FocusState, PaintDamage, WindowPaintCtx},
    widget::prelude::*,
    Bloom, GpuFrameCtx, InternalEvent, PointerEventKind, WidgetFilter,
};
use kyute_common::SizeI;
use kyute_shell::{animation::Layer, winit::event_loop::EventLoopWindowTarget};
use skia_safe as sk;
use std::{
    cell::{Cell, RefCell},
    fmt,
};

/*#[derive(Clone)]
pub struct CachedLayout {
    constraints: BoxConstraints,
    scale_factor: f64,
    layout: Option<Measurements>,
}*/

/// A container for a widget.
pub struct WidgetPod<T: ?Sized = dyn Widget> {
    /// Unique ID of the widget, if it has one.
    id: Option<WidgetId>,
    layer: Option<Layer>,
    /// Transform.
    transform: Cell<Transform>,
    /// Bloom filter to filter child widgets.
    child_filter: Cell<Option<WidgetFilter>>,
    /// Damage done to the contents of the layer.
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
        Self::new_inner(widget, None)
    }

    #[composable]
    pub fn layered(widget: T) -> WidgetPod<T> {
        let layer = cache::once(Layer::new);
        Self::new_inner(widget, Some(layer))
    }

    #[composable]
    fn new_inner(widget: T, layer: Option<Layer>) -> WidgetPod<T> {
        let id = widget.widget_id();
        WidgetPod {
            id,
            layer,
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
        self.layer.as_ref()
    }

    fn update_child_layers<'a>(&self, skia_direct_context: sk::gpu::DirectContext) {
        // "skip" this layer's items and repaint internal layers
        let mut event_ctx = EventCtx::new();
        self.content.route_event(
            &mut event_ctx,
            &mut Event::Internal(InternalEvent::UpdateLayers { skia_direct_context }),
            &Environment::new(),
        );
    }

    pub(crate) fn repaint_layer(&self, skia_direct_context: sk::gpu::DirectContext) -> bool {
        if let Some(ref layer) = self.layer {
            assert!(self.cached_measurements.get().is_some(), "repaint called before layout");
            match self.paint_damage.replace(PaintDamage::None) {
                PaintDamage::Repaint => {
                    // straight recursive repaint
                    let _span = trace_span!("Repaint layer", id=?self.id).entered();
                    layer.remove_all_children();
                    let mut ctx = PaintCtx::new(layer, self.cached_scale_factor.get(), skia_direct_context);
                    ctx.surface.canvas().clear(sk::Color4f::new(0.0, 0.0, 0.0, 0.0));
                    self.content.paint(&mut ctx);
                    ctx.finish();
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

        // if we are painting on our own layer, now we need to decide if we need to repaint it
        if let Some(ref layer) = self.layer {
            if self.cached_measurements.get() != Some(measurements) {
                // resize layer
                if !ctx.speculative {
                    let size = SizeI::new(
                        (measurements.clip_bounds.size.width * ctx.scale_factor) as i32,
                        (measurements.clip_bounds.size.height * ctx.scale_factor) as i32,
                    );
                    if !size.is_empty() {
                        layer.set_size(size);
                    } else {
                        warn!("empty layer: {:?}", self.debug_name());
                    }
                    /*eprintln!(
                        "Layout {:?} => set damage to repaint (cached_measurements={:?}, measurements={:?})",
                        self.content.debug_name(),
                        self.cached_measurements.get(),
                        measurements
                    );*/
                    self.paint_damage.set(PaintDamage::Repaint)
                }
            }
        }

        // update cached layout
        if !ctx.speculative {
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

        if let Some(ref layer) = self.layer {
            // --- LAYER PAINT ---
            match self.paint_damage.replace(PaintDamage::None) {
                PaintDamage::Repaint => {
                    // the contents of the layer are dirty
                    ctx.layer(layer, |ctx| {
                        ctx.surface.canvas().clear(sk::Color4f::new(0.0, 0.0, 0.0, 0.0));
                        self.content.paint(ctx);
                    });
                }
                PaintDamage::SubLayers => {
                    // this layer's contents are still valid, but some sublayers may need to be repainted.
                    ctx.add_layer(layer);
                    self.update_child_layers(ctx.skia_direct_context.clone());
                }
                PaintDamage::None => {
                    ctx.add_layer(layer);
                }
            }
        } else {
            // --- DIRECT PAINT ---
            ctx.with_transform_and_clip(
                &self.transform.get(),
                measurements.local_bounds(),
                measurements.clip_bounds,
                |ctx| self.content.paint(ctx),
            )
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
            if self.layer.is_some() {
                // update layer damage
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
            }

            parent_ctx.merge_event_result(event_result);
        }
    }

    fn debug_node(&self) -> DebugNode {
        DebugNode {
            content: if let Some(ref layer) = self.layer {
                Some(format!("layered {:?} px", layer.size()))
            } else {
                None
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
