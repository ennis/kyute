use crate::{
    application::AppCtx,
    bloom::Bloom,
    cache::{Cache, Key},
    call_key::CallId,
    event::{InputState, PointerEvent, PointerEventKind, PointerEventKind::PointerOut},
    layout::LayoutItem,
    region::Region,
    BoxConstraints, Data, Environment, Event, InternalEvent, Measurements, Offset, Point, Rect,
    Size,
};
use kyute_macros::composable;
use kyute_shell::{
    skia::Matrix,
    winit::{event_loop::EventLoopWindowTarget, window::WindowId},
};
use std::{
    cell::{Cell, RefCell},
    fmt,
    fmt::Formatter,
    hash::{Hash, Hasher},
    num::NonZeroU64,
    ops::{Deref, DerefMut},
    sync::{Arc, Mutex, Weak},
};

/// Context passed to widgets during the layout pass.
///
/// See [`Widget::layout`].
pub struct LayoutCtx {
    changed: bool,
}

impl LayoutCtx {
    pub fn new() -> LayoutCtx {
        LayoutCtx { changed: false }
    }
}

pub struct PaintCtx<'a> {
    pub canvas: &'a mut kyute_shell::skia::Canvas,
    pub id: WidgetId,
    pub window_bounds: Rect,
    pub focus: Option<WidgetId>,
    pub pointer_grab: Option<WidgetId>,
    pub hot: Option<WidgetId>,
    pub inputs: &'a InputState,
    pub scale_factor: f64,
    pub invalid: &'a Region,
    pub hover: bool,
}

impl<'a> PaintCtx<'a> {
    /// Returns the bounds of the node.
    pub fn bounds(&self) -> Rect {
        // FIXME: is the local origin always on the top-left corner?
        Rect::new(Point::origin(), self.window_bounds.size)
    }

    ///
    pub fn is_hovering(&self) -> bool {
        self.hover
    }

    /*/// Returns the size of the node.
    pub fn size(&self) -> Size {
        self.window_bounds.size
    }

    pub fn is_hovering(&self) -> bool {
        self.hover
    }

    pub fn is_focused(&self) -> bool {
        self.focus == Some(self.node_id)
    }

    pub fn is_capturing_pointer(&self) -> bool {
        self.pointer_grab == Some(self.node_id)
    }*/
}

pub struct EventCtx<'a> {
    pub(crate) app_ctx: &'a mut AppCtx,
    pub(crate) event_loop: &'a EventLoopWindowTarget<()>,
    window_position: Point,
    id: WidgetId,
    child_filter: Bloom<WidgetId>,
    handled: bool,
    pub(crate) relayout: bool,
    pub(crate) redraw: bool,
}

impl<'a> EventCtx<'a> {
    fn new(
        app_ctx: &'a mut AppCtx,
        event_loop: &'a EventLoopWindowTarget<()>,
        id: WidgetId,
    ) -> EventCtx<'a> {
        EventCtx {
            app_ctx,
            event_loop,
            window_position: Default::default(),
            id,
            child_filter: Default::default(),
            handled: false,
            relayout: false,
            redraw: false,
        }
    }

    pub fn widget_id(&self) -> WidgetId {
        self.id
    }

    pub fn set_state<T: 'static>(&mut self, key: Key<T>, value: T) {
        self.app_ctx.cache.set_state(key, value).unwrap()
    }

    pub fn register_window(&mut self, window_id: WindowId) {
        self.app_ctx.register_window_widget(window_id, self.id);
    }

    /// Returns the bounds of the current widget.
    // TODO in what space?
    pub fn bounds(&self) -> Rect {
        todo!()
    }

    /// Requests a redraw of the current node and its children.
    pub fn request_redraw(&mut self) {
        self.redraw = true;
    }

    pub fn request_recomposition(&mut self) {
        todo!()
    }

    /// Requests a relayout of the current widget.
    pub fn request_relayout(&mut self) {
        self.relayout = true;
    }

    /// Requests that the current node grabs all pointer events in the parent window.
    pub fn capture_pointer(&mut self) {
        todo!()
    }

    /// Returns whether the current node is capturing the pointer.
    pub fn is_capturing_pointer(&self) -> bool {
        todo!()
    }

    /// Releases the pointer grab, if the current node is holding it.
    pub fn release_pointer(&mut self) {
        todo!()
    }

    /// Acquires the focus.
    pub fn request_focus(&mut self) {
        //todo!()
    }

    /// Returns whether the current node has the focus.
    pub fn has_focus(&self) -> bool {
        todo!()
    }

    /// Signals that the passed event was handled and should not bubble up further.
    pub fn set_handled(&mut self) {
        self.handled = true;
    }

    #[must_use]
    pub fn handled(&self) -> bool {
        self.handled
    }
}

pub struct WindowPaintCtx {}

/// Trait that defines the behavior of a widget.
pub trait Widget {
    /// Implement to give a debug name to your widget. Used only for debugging.
    fn debug_name(&self) -> &str {
        "Widget"
    }

    /// Propagates an event through the widget hierarchy.
    fn event(&self, ctx: &mut EventCtx, event: &Event);

    /// Measures this widget and layouts the children of this widget.
    fn layout(
        &self,
        ctx: &mut LayoutCtx,
        constraints: BoxConstraints,
        env: &Environment,
    ) -> Measurements;

    /// Paints the widget in the given context.
    fn paint(&self, ctx: &mut PaintCtx, bounds: Rect, env: &Environment);

    /// Called only for native window widgets.
    fn window_paint(&self, _ctx: &mut WindowPaintCtx) {}
}

/// ID of a node in the tree.
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
#[repr(transparent)]
pub struct WidgetId(CallId);

impl WidgetId {
    pub(crate) fn from_call_id(call_id: CallId) -> WidgetId {
        WidgetId(call_id)
    }
}

impl fmt::Debug for WidgetId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:04X}", self.0.to_u64())
    }
}

#[derive(Copy, Clone, Debug, Hash)]
struct LayoutResult {
    constraints: BoxConstraints,
    measurements: Measurements,
}

struct WidgetPodInner<T: ?Sized> {
    /// Unique ID of the widget.
    id: WidgetId,
    /// Position of this widget relative to its parent. Set by `WidgetPod::set_child_offset`.
    offset: Cell<Offset>,
    /// Cached layout result.
    layout_result: Cell<Option<LayoutResult>>,
    child_filter: Cell<Option<Bloom<WidgetId>>>,
    /// Indicates that this widget has been initialized.
    initialized: Cell<bool>,
    /// Indicates that the children of this widget have been initialized.
    children_initialized: Cell<bool>,
    /// Any pointer hovering this widget
    /// FIXME: handle multiple pointers?
    pointer_over: Cell<bool>,
    widget: T,
}

fn compute_child_filter<T: Widget>(widget: &T) -> Bloom<WidgetId> {
    // TODO the widget needs to cooperate but there are no suitable functions in the trait
    // (`event` needs an `EventCtx`, which needs an `AppCtx`).
    Default::default()
}

/// Represents a widget.
pub struct WidgetPod<T: ?Sized = dyn Widget>(Arc<WidgetPodInner<T>>);

impl<T: Widget> WidgetPod<T> {
    /// Creates a new `WidgetPod` wrapping the specified widget.
    #[composable(uncached)]
    pub fn new(widget: T) -> WidgetPod<T> {
        let id = WidgetId::from_call_id(Cache::current_call_id());

        // HACK: returns false on first call, true on following calls, so we can use that
        // to determine whether the widget has been initialized.
        let initialized = !Cache::changed(()); // false on first call, true on following calls

        tracing::trace!(
            "WidgetPod::new[{}-{:?}]: initialized={}",
            widget.debug_name(),
            id,
            initialized
        );
        let inner = WidgetPodInner {
            id,
            offset: Cell::new(Offset::zero()),
            layout_result: Cell::new(None),
            child_filter: Cell::new(None),
            widget,
            initialized: Cell::new(initialized),
            // we don't know if all children have been initialized
            children_initialized: Cell::new(false),
            pointer_over: Cell::new(false),
        };
        WidgetPod(Arc::new(inner))
    }
}

impl<T: Widget + ?Sized> Deref for WidgetPod<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0.widget
    }
}

impl<T: ?Sized> Clone for WidgetPod<T> {
    fn clone(&self) -> Self {
        WidgetPod(self.0.clone())
    }
}

impl<T: ?Sized + 'static> Data for WidgetPod<T> {
    fn same(&self, other: &Self) -> bool {
        self.0.same(&other.0)
    }
}

// TODO remove this once we have unsized coercions
impl<T: Widget + 'static> From<WidgetPod<T>> for WidgetPod {
    fn from(other: WidgetPod<T>) -> Self {
        WidgetPod(other.0)
    }
}

impl<T: ?Sized + Widget> WidgetPod<T> {
    /// Returns a reference to the wrapped widget.
    pub fn widget(&self) -> &T {
        &self.0.widget
    }

    pub fn relayout(&self, constraints: BoxConstraints, env: &Environment) -> (Measurements, bool) {
        let mut ctx = LayoutCtx { changed: false };
        let measurements = self.layout(&mut ctx, constraints, env);
        (measurements, ctx.changed)
    }

    /// Called to measure this widget and layout the children of this widget.
    pub fn layout(
        &self,
        ctx: &mut LayoutCtx,
        constraints: BoxConstraints,
        env: &Environment,
    ) -> Measurements {
        if let Some(layout_result) = self.0.layout_result.get() {
            if layout_result.constraints.same(&constraints) {
                return layout_result.measurements;
            }
        }

        let measurements = self.0.widget.layout(ctx, constraints, env);
        tracing::trace!(
            "layout[{}-{:?}]: {:?}",
            self.0.widget.debug_name(),
            self.0.id,
            measurements
        );
        self.0.layout_result.set(Some(LayoutResult {
            constraints,
            measurements,
        }));
        ctx.changed = true;
        measurements
    }

    pub fn set_child_offset(&self, offset: Offset) {
        self.0.offset.set(offset)
    }

    /// Paints the widget.
    pub fn paint(&self, ctx: &mut PaintCtx, bounds: Rect, env: &Environment) {
        let offset = self.0.offset.get();
        let measurements = if let Some(layout_result) = self.0.layout_result.get() {
            layout_result.measurements
        } else {
            tracing::warn!("`paint` called before layout");
            return;
        };
        let size = measurements.size;
        // bounds of this widget in window space
        let window_bounds = Rect::new(ctx.window_bounds.origin + offset, size);
        if !ctx.invalid.intersects(window_bounds) {
            tracing::trace!("not repainting valid region");
            // not invalidated, no need to redraw
            return;
        }

        /*let _span = trace_span!(
            "paint",
            ?self.id,
            ?offset,
            ?measurements,
        ).entered();*/
        // trace!(?ctx.scale_factor, ?ctx.inputs.pointers, ?window_bounds, "paint");

        let hover = ctx
            .inputs
            .pointers
            .iter()
            .any(|(_, state)| window_bounds.contains(state.position));

        ctx.canvas.save();
        ctx.canvas.translate(kyute_shell::skia::Vector::new(
            offset.x as f32,
            offset.y as f32,
        ));

        {
            let mut child_ctx = PaintCtx {
                canvas: ctx.canvas,
                window_bounds,
                focus: ctx.focus,
                pointer_grab: ctx.pointer_grab,
                hot: ctx.hot,
                inputs: ctx.inputs,
                scale_factor: ctx.scale_factor,
                id: self.0.id,
                hover,
                invalid: &ctx.invalid,
            };
            self.0
                .widget
                .paint(&mut child_ctx, Rect::new(Point::origin(), size), env);
        }

        ctx.canvas.restore();
    }

    pub(crate) fn compute_child_filter(&self, parent_ctx: &mut EventCtx) -> Bloom<WidgetId> {
        if let Some(filter) = self.0.child_filter.get() {
            // already computed
            filter
        } else {
            tracing::trace!("computing child filter");
            // not computed: compute by sending the `UpdateChildFilter` message to the widget,
            // which will be forwarded to all children, which in turn will update `ctx.child_filter`.
            // NOTE: we ignore any relayout/repaint requests during UpdateChildFilter
            let mut ctx = EventCtx::new(parent_ctx.app_ctx, parent_ctx.event_loop, self.0.id);
            self.0
                .widget
                .event(&mut ctx, &Event::Internal(InternalEvent::UpdateChildFilter));
            self.0.child_filter.set(Some(ctx.child_filter));
            ctx.child_filter
        }
    }

    /// Returns whether this widget may contain the specified widget as a child (direct or not).
    fn may_contain(&self, widget: WidgetId) -> bool {
        if let Some(filter) = self.0.child_filter.get() {
            filter.may_contain(&widget)
        } else {
            tracing::warn!("`may_contain` called but child filter not initialized");
            true
        }
    }

    fn do_event(&self, parent_ctx: &mut EventCtx, event: &Event) {
        let offset = self.0.offset.get();
        let window_position = parent_ctx.window_position + offset;
        let mut ctx = EventCtx {
            app_ctx: parent_ctx.app_ctx,
            event_loop: parent_ctx.event_loop,
            window_position,
            id: self.0.id,
            child_filter: Default::default(),
            handled: false,
            relayout: false,
            redraw: false,
        };
        self.0.widget.event(&mut ctx, event);
        if ctx.relayout {
            tracing::trace!(widget_id = ?self.0.id, "requested relayout");
            // relayout requested by the widget: invalidate cached measurements and offset
            self.0.layout_result.set(None);
            self.0.offset.set(Offset::zero());
            // propagate relayout request to parent
            parent_ctx.relayout = true;
        }
        parent_ctx.redraw |= ctx.redraw;
    }

    /// Propagates an event to the wrapped widget.
    pub fn event(&self, parent_ctx: &mut EventCtx, event: &Event) {
        if parent_ctx.handled {
            tracing::warn!("event already handled");
            return;
        }

        // ---- Handle internal events (routing mostly) ----
        match event {
            Event::Internal(InternalEvent::RouteWindowEvent { target, event }) => {
                if *target == self.0.id {
                    self.do_event(parent_ctx, &Event::WindowEvent(event.clone()));
                    return;
                }
                if !self.may_contain(*target) {
                    return;
                }
            }
            Event::Internal(InternalEvent::RouteRedrawRequest(target)) => {
                if *target == self.0.id {
                    self.do_event(parent_ctx, &Event::WindowRedrawRequest);
                    return;
                }
                if !self.may_contain(*target) {
                    return;
                }
            }
            Event::Internal(InternalEvent::UpdateChildFilter) => {
                parent_ctx.child_filter.add(&self.0.id);
                let child_filter = self.compute_child_filter(parent_ctx);
                parent_ctx.child_filter.extend(&child_filter);
                return;
            }
            Event::Internal(InternalEvent::RouteInitialize) | Event::Initialize => {
                // TODO explain the logic here
                let init = self.0.initialized.get();
                let child_init = self.0.children_initialized.get();
                match (init, child_init) {
                    (false, _) => self.do_event(parent_ctx, &Event::Initialize),
                    (true, false) => {
                        self.do_event(parent_ctx, &Event::Internal(InternalEvent::RouteInitialize))
                    }
                    _ => {}
                }
                self.0.initialized.set(true);
                self.0.children_initialized.set(true);
                return;
            }
            _ => {}
        }

        // ---- Handle pointer events, for which we must do hit-test ----
        let position_adjusted_event;
        let offset = self.0.offset.get();
        let measurements = if let Some(layout_result) = self.0.layout_result.get() {
            layout_result.measurements
        } else {
            tracing::warn!("`event` called before layout ({:?})", event);
            return;
        };
        let window_position = parent_ctx.window_position + offset;
        let bounds = Rect::new(window_position, measurements.size);

        let modified_event = match event {
            Event::Pointer(pointer_event) => {
                let adjusted_pointer_event = PointerEvent {
                    position: (pointer_event.window_position - window_position).to_point(),
                    ..*pointer_event
                };
                position_adjusted_event = Event::Pointer(adjusted_pointer_event);

                if !bounds.contains(pointer_event.window_position) {
                    // pointer hit-test fail; if we were hovering the widget, send pointerout
                    if self.0.pointer_over.get() {
                        self.0.pointer_over.set(false);
                        self.do_event(
                            parent_ctx,
                            &Event::Pointer(PointerEvent {
                                kind: PointerEventKind::PointerOut,
                                ..adjusted_pointer_event
                            }),
                        );
                    }
                    // pointer hit-test fail, don't recurse
                    return;
                } else {
                    // pointer hit-test pass; send pointerover
                    if !self.0.pointer_over.get() {
                        self.0.pointer_over.set(true);
                        self.do_event(
                            parent_ctx,
                            &Event::Pointer(PointerEvent {
                                kind: PointerEventKind::PointerOver,
                                ..adjusted_pointer_event
                            }),
                        );
                    }
                    // pointer event is modified
                    &position_adjusted_event
                }
            }
            // send event as-is
            _ => event,
        };

        // --- propagate to the widget inside ---
        self.do_event(parent_ctx, modified_event);
    }

    /// Prepares the root `EventCtx` and calls `self.event()`.
    pub(crate) fn send_root_event(
        &self,
        app_ctx: &mut AppCtx,
        event_loop: &EventLoopWindowTarget<()>,
        event: &Event,
    ) {
        // FIXME callId?
        let mut event_ctx = EventCtx::new(app_ctx, event_loop, WidgetId::from_call_id(CallId(0)));
        //tracing::trace!("event={:?}", event);
        self.event(&mut event_ctx, event);
    }

    pub(crate) fn ensure_initialized(
        &self,
        app_ctx: &mut AppCtx,
        event_loop: &EventLoopWindowTarget<()>,
    ) {
        let mut event_ctx = EventCtx::new(app_ctx, event_loop, WidgetId::from_call_id(CallId(0)));
        self.compute_child_filter(&mut event_ctx);
        self.send_root_event(
            app_ctx,
            event_loop,
            &Event::Internal(InternalEvent::RouteInitialize),
        );
        self.root_layout(app_ctx);
    }

    pub(crate) fn root_layout(&self, app_ctx: &mut AppCtx) -> bool {
        let mut ctx = LayoutCtx { changed: false };
        let env = Environment::new();
        self.layout(
            &mut ctx,
            BoxConstraints {
                min: Size::new(0.0, 0.0),
                max: Size::new(f64::INFINITY, f64::INFINITY),
            },
            &env,
        );
        ctx.changed
    }
}
