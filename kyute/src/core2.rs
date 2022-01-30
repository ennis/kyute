use crate::{
    application::AppCtx,
    bloom::Bloom,
    cache,
    cache::{Cache, Key},
    call_key::CallId,
    event::{InputState, PointerEvent, PointerEventKind},
    region::Region,
    styling::PaintCtxExt,
    BoxConstraints, Data, EnvKey, Environment, Event, InternalEvent, Measurements, Offset, Point,
    Rect,
};
use kyute_macros::composable;
use kyute_shell::{
    drawing::{Color, Path},
    graal,
    graal::{ash::vk, BufferId, ImageId},
    winit::{event_loop::EventLoopWindowTarget, window::WindowId},
};
use std::{
    cell::Cell,
    fmt,
    hash::Hash,
    marker::Unsize,
    ops::{CoerceUnsized, Deref},
    str::FromStr,
    sync::{Arc, Weak},
};
use tracing::{trace, warn};

pub const SHOW_DEBUG_OVERLAY: EnvKey<bool> = EnvKey::new("kyute.show_debug_overlay");

/// Context passed to widgets during the layout pass.
///
/// See [`Widget::layout`].
pub struct LayoutCtx {
    pub scale_factor: f64,
    changed: bool,
}

impl LayoutCtx {
    pub fn new(scale_factor: f64) -> LayoutCtx {
        LayoutCtx {
            scale_factor,
            changed: false,
        }
    }

    pub fn round_to_pixel(&self, dip_length: f64) -> f64 {
        (dip_length * self.scale_factor).round()
    }
}

// TODO make things private
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
    pub measurements: Measurements,
}

impl<'a> PaintCtx<'a> {
    /// Returns the bounds of the node.
    pub fn bounds(&self) -> Rect {
        // FIXME: is the local origin always on the top-left corner?
        Rect::new(Point::origin(), self.window_bounds.size)
    }

    /// Returns the measurements computed during layout.
    pub fn measurements(&self) -> Measurements {
        self.measurements
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

#[derive(Debug, Default)]
pub struct GpuResourceReferences {
    pub images: Vec<ImageAccess>,
    pub buffers: Vec<BufferAccess>,
}

impl GpuResourceReferences {
    pub fn new() -> GpuResourceReferences {
        GpuResourceReferences {
            images: vec![],
            buffers: vec![],
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct EventResult {
    pub handled: bool,
    pub relayout: bool,
    pub redraw: bool,
}

#[derive(Copy, Clone, Debug)]
pub struct WindowInfo {
    pub scale_factor: f64,
}

/// Global state related to focus and pointer grab.
#[derive(Clone, Debug, Default)]
pub struct FocusState {
    pub(crate) focus: Option<WidgetId>,
    pub(crate) pointer_grab: Option<WidgetId>,
    pub(crate) hot: Option<WidgetId>,
    /// Target of popup menu events
    pub(crate) popup_target: Option<WidgetId>,
}

impl FocusState {
    pub fn new() -> FocusState {
        FocusState {
            focus: None,
            pointer_grab: None,
            hot: None,
            popup_target: None,
        }
    }
}

pub struct EventCtx<'a> {
    pub(crate) app_ctx: &'a mut AppCtx,
    pub(crate) event_loop: &'a EventLoopWindowTarget<()>,
    pub(crate) parent_window: Option<&'a mut kyute_shell::window::Window>,
    pub(crate) focus_state: &'a mut FocusState,
    pub(crate) window_position: Point,
    pub(crate) scale_factor: f64,
    pub(crate) id: WidgetId,
    pub(crate) handled: bool,
    pub(crate) relayout: bool,
    pub(crate) redraw: bool,
}

impl<'a> EventCtx<'a> {
    /// Creates the root `EventCtx`
    fn new(
        app_ctx: &'a mut AppCtx,
        focus_state: &'a mut FocusState,
        event_loop: &'a EventLoopWindowTarget<()>,
        id: WidgetId,
    ) -> EventCtx<'a> {
        EventCtx {
            app_ctx,
            event_loop,
            parent_window: None,
            focus_state,
            window_position: Default::default(),
            scale_factor: 1.0,
            id,
            handled: false,
            relayout: false,
            redraw: false,
        }
    }

    /// Creates a new `EventCtx` to propagate events in a subwindow.
    pub(crate) fn new_subwindow<'b>(
        parent: &'b mut EventCtx,
        scale_factor: f64,
        window: &'b mut kyute_shell::window::Window,
        focus_state: &'b mut FocusState,
    ) -> EventCtx<'b>
    where
        'a: 'b,
    {
        EventCtx {
            app_ctx: parent.app_ctx,
            event_loop: parent.event_loop,
            parent_window: Some(window),
            focus_state,
            // reset window pos because we're entering a child window
            window_position: Point::origin(),
            scale_factor,
            id: parent.id,
            handled: false,
            relayout: false,
            redraw: false,
        }
    }

    pub fn widget_id(&self) -> WidgetId {
        self.id
    }

    pub fn set_state<T: 'static>(&mut self, key: Key<T>, value: T) {
        self.app_ctx.cache.set_state(key, value)
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
        self.focus_state.pointer_grab = Some(self.id);
    }

    /// Returns whether the current node is capturing the pointer.
    pub fn is_capturing_pointer(&self) -> bool {
        self.focus_state.pointer_grab == Some(self.id)
    }

    /// Releases the pointer grab, if the current node is holding it.
    pub fn release_pointer(&mut self) {
        if self.focus_state.pointer_grab == Some(self.id) {
            trace!("releasing pointer grab");
        } else {
            warn!("pointer capture release requested but the current widget isn't capturing the pointer");
        }
    }

    /// Acquires the focus.
    pub fn request_focus(&mut self) {
        trace!("acquiring focus");
        self.focus_state.focus = Some(self.id);
    }

    pub fn track_popup_menu(&mut self, menu: kyute_shell::Menu, at: Point) {
        self.focus_state.popup_target = Some(self.id);
        self.parent_window
            .as_mut()
            .expect("EventCtx::track_popup_menu called without a parent window")
            .show_context_menu(menu, at);
    }

    /// Returns whether the current node has the focus.
    pub fn has_focus(&self) -> bool {
        self.focus_state.focus == Some(self.id)
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

#[derive(Debug)]
pub struct ImageAccess {
    pub id: ImageId,
    pub initial_layout: vk::ImageLayout,
    pub final_layout: vk::ImageLayout,
    pub access_mask: vk::AccessFlags,
    pub stage_mask: vk::PipelineStageFlags,
}

#[derive(Debug)]
pub struct BufferAccess {
    pub id: BufferId,
    pub access_mask: vk::AccessFlags,
    pub stage_mask: vk::PipelineStageFlags,
}

pub struct GpuFrameCtx<'a, 'b> {
    /// graal context in frame recording state.
    pub(crate) frame: &'b mut graal::Frame<'a, ()>,
    pub(crate) resource_references: GpuResourceReferences,
    pub(crate) measurements: Measurements,
    pub(crate) scale_factor: f64,
}

impl<'a, 'b> GpuFrameCtx<'a, 'b> {
    /// Returns a ref to the frame.
    pub fn frame(&mut self) -> &mut graal::Frame<'a, ()> {
        self.frame
    }

    pub fn measurements(&self) -> Measurements {
        self.measurements
    }

    /// Registers an image that will be accessed during paint.
    pub fn reference_paint_image(
        &mut self,
        id: ImageId,
        access_mask: vk::AccessFlags,
        stage_mask: vk::PipelineStageFlags,
        initial_layout: vk::ImageLayout,
        final_layout: vk::ImageLayout,
    ) {
        self.resource_references.images.push(ImageAccess {
            id,
            initial_layout,
            final_layout,
            access_mask,
            stage_mask,
        })
    }

    /// Registers a buffer that will be accessed during paint.
    pub fn reference_paint_buffer(
        &mut self,
        id: BufferId,
        access_mask: vk::AccessFlags,
        stage_mask: vk::PipelineStageFlags,
    ) {
        self.resource_references.buffers.push(BufferAccess {
            id,
            access_mask,
            stage_mask,
        })
    }
}

/// Trait that defines the behavior of a widget.
pub trait Widget {
    /// Implement to give a debug name to your widget. Used only for debugging.
    fn debug_name(&self) -> &str {
        "Widget"
    }

    /// Propagates an event through the widget hierarchy.
    fn event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment);

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

    /// Called for custom GPU operations
    fn gpu_frame<'a, 'b>(&'a self, _ctx: &mut GpuFrameCtx<'a, 'b>) {}
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

/*// TODO It would be good to have a visual tree (or quadtree) so that we can do optimized hit-tests and
// event delivery.
struct VisualNode {
    widget: WidgetPod,
    measurements: Measurements,
    children: Vec<Arc<VisualNode>>,
}
*/

#[derive(Copy, Clone, Debug, Hash)]
struct LayoutResult {
    constraints: BoxConstraints,
    measurements: Measurements,
}

struct WidgetPodState {
    /// Weak ref to self as a `dyn Widget`, used to collect widgets in `InternalEvent::Traverse`
    this: Weak<WidgetPodInner<dyn Widget>>,
    // TODO add a flag for paint invalidation?
    /// Unique ID of the widget.
    id: WidgetId,
    /// Position of this widget relative to its parent. Set by `WidgetPod::set_child_offset`.
    offset: Cell<Offset>,
    /// Cached layout result.
    layout_result: Cell<Option<LayoutResult>>,
    /// Indicates that this widget should be repainted.
    /// Set by `layout` if the layout has changed somehow, after event handling if `EventCtx::request_redraw` was called,
    /// and by `set_child_offset`.
    paint_invalid: Cell<bool>,
    child_filter: Cell<Option<Bloom<WidgetId>>>,
    /// Indicates that this widget has been initialized.
    initialized: Cell<bool>,
    /// Indicates that the children of this widget have been initialized.
    children_initialized: Cell<bool>,
    /// Any pointer hovering this widget
    /// FIXME: handle multiple pointers?
    pointer_over: Cell<bool>,

    /// The revision in which the WidgetPod was created.
    created: usize,
    /// Debugging: flag indicating whether this WidgetPod was recreated since the last
    /// debug paint.
    created_since_debug_paint: Cell<bool>,
}

impl WidgetPodState {
    /// Sets the offset of this widget relative to its parent.
    pub fn set_child_offset(&self, offset: Offset) {
        self.offset.set(offset);
        self.paint_invalid.set(true);
    }
}

struct WidgetPodInner<T: ?Sized> {
    state: WidgetPodState,
    widget: T,
}

impl<T: Widget + ?Sized> WidgetPodInner<T> {
    fn compute_child_filter(
        &self,
        parent_ctx: &mut EventCtx,
        env: &Environment,
    ) -> Bloom<WidgetId> {
        if let Some(filter) = self.state.child_filter.get() {
            // already computed
            filter
        } else {
            //tracing::trace!("computing child filter");
            let mut filter = Default::default();
            self.do_event(
                parent_ctx,
                &mut Event::Internal(InternalEvent::UpdateChildFilter {
                    filter: &mut filter,
                }),
                env,
            );
            self.state.child_filter.set(Some(filter));
            filter
        }
    }

    /// Returns whether this widget may contain the specified widget as a child (direct or not).
    fn may_contain(&self, widget: WidgetId) -> bool {
        if let Some(filter) = self.state.child_filter.get() {
            filter.may_contain(&widget)
        } else {
            tracing::warn!("`may_contain` called but child filter not initialized");
            true
        }
    }

    /// Used internally by `event`. In charge of calling the `event` method on the widget with
    /// the child `EventCtx`, and handling its result.
    fn do_event(&self, parent_ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        let offset = self.state.offset.get();
        let window_position = parent_ctx.window_position + offset;
        let mut ctx = EventCtx {
            app_ctx: parent_ctx.app_ctx,
            event_loop: parent_ctx.event_loop,
            parent_window: parent_ctx.parent_window.as_deref_mut(),
            focus_state: parent_ctx.focus_state,
            window_position,
            scale_factor: parent_ctx.scale_factor,
            id: self.state.id,
            handled: false,
            relayout: false,
            redraw: false,
        };
        self.widget.event(&mut ctx, event, env);
        if ctx.relayout {
            //tracing::trace!(widget_id = ?self.state.id, "requested relayout");
            // relayout requested by the widget: invalidate cached measurements and offset
            self.state.layout_result.set(None);
            self.state.offset.set(Offset::zero());
            self.state.paint_invalid.set(true);
        } else if ctx.redraw {
            //tracing::trace!(widget_id = ?self.state.id, "requested redraw");
            self.state.paint_invalid.set(true);
        }

        // propagate results to parent
        parent_ctx.relayout |= ctx.relayout;
        parent_ctx.redraw |= ctx.redraw;
        parent_ctx.handled = ctx.handled;
    }

    fn layout(
        &self,
        ctx: &mut LayoutCtx,
        constraints: BoxConstraints,
        env: &Environment,
    ) -> Measurements {
        // FIXME also compare env
        if let Some(layout_result) = self.state.layout_result.get() {
            if layout_result.constraints.same(&constraints) {
                return layout_result.measurements;
            }
        }

        let measurements = self.widget.layout(ctx, constraints, env);
        /*tracing::trace!(
            "layout[{}-{:?}]: {:?}",
            self.widget.debug_name(),
            self.state.id,
            measurements
        );*/
        self.state.layout_result.set(Some(LayoutResult {
            constraints,
            measurements,
        }));
        self.state.paint_invalid.set(true);
        ctx.changed = true;
        measurements
    }

    fn paint(&self, ctx: &mut PaintCtx, bounds: Rect, env: &Environment) {
        /*if !self.0.paint_invalid.get() {
            // no need to repaint
            return;
        }*/

        let offset = self.state.offset.get();
        let measurements = if let Some(layout_result) = self.state.layout_result.get() {
            layout_result.measurements
        } else {
            tracing::warn!(id=?self.state.id, "`paint` called with invalid layout");
            return;
        };
        let size = measurements.size();
        // bounds of this widget in window space
        let window_bounds = Rect::new(ctx.window_bounds.origin + offset, size);
        if !ctx.invalid.intersects(window_bounds) {
            //tracing::trace!("not repainting valid region");
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
                id: self.state.id,
                hover,
                invalid: &ctx.invalid,
                measurements,
            };
            self.widget
                .paint(&mut child_ctx, Rect::new(Point::origin(), size), env);
        }

        if !env.get(SHOW_DEBUG_OVERLAY).unwrap_or_default() {
            use crate::styling::*;
            use kyute_shell::{drawing::ToSkia, skia as sk};

            if self.state.created_since_debug_paint.take() {
                ctx.draw_styled_box(
                    measurements.bounds,
                    rectangle().with(
                        border(1.0)
                            .inside(0.0)
                            .brush(Color::new(0.9, 0.8, 0.0, 1.0)),
                    ),
                    env,
                );
                ctx.canvas.draw_line(
                    Point::new(0.5, 0.5).to_skia(),
                    Point::new(6.5, 0.5).to_skia(),
                    &sk::Paint::new(Color::new(1.0, 0.0, 0.0, 1.0).to_skia(), None),
                );
                ctx.canvas.draw_line(
                    Point::new(0.5, 0.5).to_skia(),
                    Point::new(0.5, 6.5).to_skia(),
                    &sk::Paint::new(Color::new(0.0, 1.0, 0.0, 1.0).to_skia(), None),
                );

                {
                    let w = measurements.bounds.width() as sk::scalar;
                    let mut font: sk::Font = sk::Font::new(sk::Typeface::default(), Some(10.0));
                    font.set_edging(sk::font::Edging::Alias);
                    let text = format!("{}", self.state.created);
                    let text_blob =
                        sk::TextBlob::from_str(&text, &font).unwrap();
                    let text_paint: sk::Paint =
                        sk::Paint::new(sk::Color4f::new(0.0, 0.0, 0.0, 1.0), None);
                    let bg_paint: sk::Paint =
                        sk::Paint::new(sk::Color4f::new(0.9, 0.8, 0.0, 1.0), None);
                    let (_, bounds) = font.measure_str(&text, Some(&text_paint));
                    ctx.canvas.draw_rect(
                        sk::Rect::new(w - bounds.width(), 0.0, w, bounds.height()),
                        &bg_paint,
                    );
                    ctx.canvas
                        .draw_text_blob(text_blob, (w - bounds.width(), -bounds.y()), &text_paint);
                    //let bounds = Rect::from_skia(bounds);
                }
            }
        }

        ctx.canvas.restore();
        self.state.paint_invalid.set(false);
    }

    /*fn propagate_event(&self, parent_ctx: &mut EventCtx, path: &[WidgetId], event: &mut Event, env: &Environment) {

    }*/

    /// Propagates an event to the wrapped widget.
    fn event(&self, parent_ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        if parent_ctx.handled {
            tracing::warn!("event already handled");
            return;
        }

        // first, ensure that the child filter has been computed and the child widgets are initialized
        self.compute_child_filter(parent_ctx, env);

        // ---- Handle internal events (routing mostly) ----
        match *event {
            Event::Internal(InternalEvent::RouteWindowEvent {
                target,
                ref mut event,
            }) => {
                // routing of `winit::WindowEvent`s to the corresponding window widget.
                if target == self.state.id {
                    self.do_event(parent_ctx, &mut Event::WindowEvent(event.clone()), env);
                    return;
                }
                if !self.may_contain(target) {
                    return;
                }
            }
            Event::Internal(InternalEvent::RouteEvent {
                target,
                ref mut event,
            }) => {
                if target == self.state.id {
                    self.event(parent_ctx, event, env);
                    return;
                }
                if !self.may_contain(target) {
                    return;
                }
                // otherwise, recurse
            }
            Event::Internal(InternalEvent::Traverse { ref mut widgets }) => {
                // T: ?Sized
                // This is problematic: it must clone self, and thus we must either have T == dyn Widget or T:Sized
                widgets.push(WidgetPod(self.state.this.upgrade().unwrap()));
            }
            Event::Internal(InternalEvent::RouteRedrawRequest(target)) => {
                if target == self.state.id {
                    self.do_event(parent_ctx, &mut Event::WindowRedrawRequest, env);
                    return;
                }
                if !self.may_contain(target) {
                    return;
                }
            }
            Event::Internal(InternalEvent::UpdateChildFilter { ref mut filter }) => {
                filter.add(&self.state.id);
                let child_filter = self.compute_child_filter(parent_ctx, env);
                filter.extend(&child_filter);
                return;
            }
            Event::Internal(InternalEvent::RouteInitialize) | Event::Initialize => {
                // TODO explain the logic here
                let init = self.state.initialized.get();
                let child_init = self.state.children_initialized.get();
                match (init, child_init) {
                    (false, _) => self.do_event(parent_ctx, &mut Event::Initialize, env),
                    (true, false) => self.do_event(
                        parent_ctx,
                        &mut Event::Internal(InternalEvent::RouteInitialize),
                        env,
                    ),
                    _ => {}
                }
                self.state.initialized.set(true);
                self.state.children_initialized.set(true);
                return;
            }
            _ => {}
        }

        // ---- Handle pointer events, for which we must do hit-test ----
        let mut position_adjusted_event;
        let offset = self.state.offset.get();
        let measurements = if let Some(layout_result) = self.state.layout_result.get() {
            layout_result.measurements
        } else {
            tracing::warn!("`event` called before layout ({:?})", event);
            return;
        };
        let window_position = parent_ctx.window_position + offset;
        let bounds = Rect::new(window_position, measurements.size());

        let modified_event = match event {
            Event::Pointer(pointer_event) => {
                let adjusted_pointer_event = PointerEvent {
                    position: (pointer_event.window_position - window_position).to_point(),
                    ..*pointer_event
                };
                position_adjusted_event = Event::Pointer(adjusted_pointer_event);

                // TODO (?): perform hit-test before sending the pointer event instead of
                // doing it "on-the-fly" when propagating the pointer event?
                // this would make the code cleaner, at the cost of two traversals instead of one.

                // pass hit test if we have pointer capture
                if !bounds.contains(pointer_event.window_position)
                    && parent_ctx.focus_state.pointer_grab != Some(self.state.id)
                {
                    // pointer hit-test fail; if we were hovering the widget, send pointerout
                    if self.state.pointer_over.get() {
                        self.state.pointer_over.set(false);
                        self.do_event(
                            parent_ctx,
                            &mut Event::Pointer(PointerEvent {
                                kind: PointerEventKind::PointerOut,
                                ..adjusted_pointer_event
                            }),
                            env,
                        );
                    }
                    // pointer hit-test fail, don't recurse
                    return;
                } else {
                    // pointer hit-test pass; send pointerover
                    if !self.state.pointer_over.get() {
                        self.state.pointer_over.set(true);
                        self.do_event(
                            parent_ctx,
                            &mut Event::Pointer(PointerEvent {
                                kind: PointerEventKind::PointerOver,
                                ..adjusted_pointer_event
                            }),
                            env,
                        );
                    }
                    // pointer event is modified
                    &mut position_adjusted_event
                }
            }
            // send event as-is
            _ => event,
        };

        // --- propagate to the widget inside ---
        self.do_event(parent_ctx, modified_event, env);
    }
}

/// Represents a widget.
pub struct WidgetPod<T: ?Sized = dyn Widget>(Arc<WidgetPodInner<T>>);

// Unsized coercions
impl<T, U> CoerceUnsized<WidgetPod<U>> for WidgetPod<T>
where
    T: Unsize<U> + ?Sized,
    U: ?Sized,
{
}

impl fmt::Debug for WidgetPod {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // TODO
        f.debug_tuple("WidgetPod").finish()
    }
}

impl<T: Widget + 'static> WidgetPod<T> {
    /// Creates a new `WidgetPod` wrapping the specified widget.
    #[composable(uncached)]
    pub fn new(widget: T) -> WidgetPod<T> {
        let id = WidgetId::from_call_id(cache::current_call_id());

        // HACK: returns false on first call, true on following calls, so we can use that
        // to determine whether the widget has been initialized.
        let initialized = !cache::changed(());
        let created = cache::revision();

        WidgetPod(Arc::new_cyclic(|this: &Weak<WidgetPodInner<T>>| {
            WidgetPodInner {
                state: WidgetPodState {
                    this: this.clone(),
                    id,
                    offset: Cell::new(Offset::zero()),
                    layout_result: Cell::new(None),
                    paint_invalid: Cell::new(true),
                    child_filter: Cell::new(None),
                    initialized: Cell::new(initialized),
                    // we don't know if all children have been initialized
                    children_initialized: Cell::new(false),
                    pointer_over: Cell::new(false),
                    created,
                    created_since_debug_paint: Cell::new(true),
                },
                widget,
            }
        }))
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

/*// TODO remove this once we have unsized coercions
impl<T: Widget + 'static> From<WidgetPod<T>> for WidgetPod {
    fn from(other: WidgetPod<T>) -> Self {
        WidgetPod(other.0)
    }
}*/

impl<T: ?Sized + Widget> WidgetPod<T> {
    /// Returns a reference to the wrapped widget.
    pub fn widget(&self) -> &T {
        &self.0.widget
    }

    /// Returns the widget id.
    pub fn id(&self) -> WidgetId {
        self.0.state.id
    }

    /// Returns previously set child offset. See `set_child_offset`.
    pub fn child_offset(&self) -> Offset {
        self.0.state.offset.get()
    }

    /// TODO documentation
    /// Sets the offset of this widget relative to its parent. Should be called during widget layout.
    pub fn set_child_offset(&self, offset: Offset) {
        self.0.state.set_child_offset(offset);
    }

    /// Returns whether the widget should be repainted.
    pub fn invalidated(&self) -> bool {
        self.0.state.paint_invalid.get()
    }

    /// Computes the layout of this widget and its children. Returns the measurements, and whether
    /// the measurements have changed since last layout.
    pub fn relayout(
        &self,
        constraints: BoxConstraints,
        scale_factor: f64,
        env: &Environment,
    ) -> (Measurements, bool) {
        let mut ctx = LayoutCtx {
            scale_factor,
            changed: false,
        };
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
        self.0.layout(ctx, constraints, env)
    }

    /// Paints the widget.
    pub fn paint(&self, ctx: &mut PaintCtx, bounds: Rect, env: &Environment) {
        self.0.paint(ctx, bounds, env)
    }

    /// Propagates an event to the wrapped widget.
    pub fn event(&self, parent_ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        self.0.event(parent_ctx, event, env)
    }

    /// Prepares the root `EventCtx` and calls `self.event()`.
    pub(crate) fn send_root_event(
        &self,
        app_ctx: &mut AppCtx,
        event_loop: &EventLoopWindowTarget<()>,
        event: &mut Event,
        env: &Environment,
    ) {
        // FIXME callId?

        // The dummy `FocusState` for the root `EventCtx`. It is eventually replaced with the `FocusState`
        // managed by `Window` widgets.
        let mut dummy_focus_state = FocusState::default();
        let mut event_ctx = EventCtx::new(
            app_ctx,
            &mut dummy_focus_state,
            event_loop,
            WidgetId::from_call_id(CallId(0)),
        );
        //tracing::trace!("event={:?}", event);
        self.event(&mut event_ctx, event, env);
    }

    /// Initializes and layouts the widget if necessary (propagates the `Initialize` event and
    /// calls `root_layout`.
    pub(crate) fn initialize(
        &self,
        app_ctx: &mut AppCtx,
        event_loop: &EventLoopWindowTarget<()>,
        env: &Environment,
    ) {
        self.send_root_event(
            app_ctx,
            event_loop,
            &mut Event::Internal(InternalEvent::RouteInitialize),
            env,
        );
    }

    /*pub(crate) fn root_layout(&self, app_ctx: &mut AppCtx, env: &Environment) -> bool {
        let mut ctx = LayoutCtx { changed: false };
        self.layout(
            &mut ctx,
            BoxConstraints {
                min: Size::new(0.0, 0.0),
                max: Size::new(f64::INFINITY, f64::INFINITY),
            },
            env,
        );
        ctx.changed
    }*/
}
