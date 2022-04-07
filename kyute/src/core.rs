use crate::{
    animation::layer::{Layer, LayerHandle},
    application::{AppCtx, ExtEvent},
    bloom::Bloom,
    cache,
    cache::state,
    call_id::CallId,
    composable,
    drawing::ToSkia,
    event::{InputState, PointerEvent, PointerEventKind},
    region::Region,
    shell::{
        graal,
        graal::{ash::vk, BufferId, ImageId},
        winit::{event_loop::EventLoopWindowTarget, window::WindowId},
    },
    style::VisualState,
    widget::{Align, ConstrainedBox, Padding},
    Alignment, BoxConstraints, EnvKey, Environment, Event, InternalEvent, Length, Measurements, Offset, Point, Rect,
    State, Transform, UnitExt,
};
use skia_safe as sk;
use std::{cell::Cell, fmt, hash::Hash, sync::Arc};
use tracing::{trace, warn};

pub const SHOW_DEBUG_OVERLAY: EnvKey<bool> = EnvKey::new("kyute.core.show_debug_overlay");
//pub const SELECTED: EnvKey<bool> = EnvKey::new("kyute.core.selected");
pub const DISABLED: EnvKey<bool> = EnvKey::new("kyute.core.disabled");

/// Context passed to widgets during the layout pass.
///
/// See [`Widget::layout`].
pub struct LayoutCtx<'a> {
    pub scale_factor: f64,
    app_ctx: &'a mut AppCtx,
    changed: bool,
}

impl<'a> LayoutCtx<'a> {
    pub fn round_to_pixel(&self, dip_length: f64) -> f64 {
        (dip_length * self.scale_factor).round()
    }
}

// TODO make things private
pub struct PaintCtx<'a> {
    pub canvas: &'a mut skia_safe::Canvas,
    //pub id: Option<WidgetId>,
    pub window_transform: Transform,
    //pub focus: Option<WidgetId>,
    //pub pointer_grab: Option<WidgetId>,
    //pub hot: Option<WidgetId>,
    //pub inputs: &'a InputState,
    pub scale_factor: f64,
    pub invalid: &'a Region,
    //pub hover: bool,
    pub bounds: Rect,
    //pub active: bool,
}

impl<'a> PaintCtx<'a> {
    pub fn with_transform_and_clip<R>(
        &mut self,
        transform: Transform,
        bounds: Rect,
        clip: Rect,
        f: impl FnOnce(&mut PaintCtx) -> R,
    ) -> R {
        let prev_window_transform = self.window_transform;
        let prev_bounds = self.bounds;

        self.window_transform = transform.then(&self.window_transform);
        self.bounds = bounds;

        self.canvas.save();
        self.canvas.reset_matrix();
        let scale_factor = self.scale_factor as sk::scalar;
        self.canvas.scale((scale_factor, scale_factor));
        self.canvas.concat(&self.window_transform.to_skia());
        self.canvas.clip_rect(clip.to_skia(), None, None);

        let result = f(self);

        self.canvas.restore();

        self.bounds = prev_bounds;
        self.window_transform = prev_window_transform;
        result
    }
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

/*impl FocusState {
    pub fn new() -> FocusState {
        FocusState {
            focus: None,
            pointer_grab: None,
            hot: None,
            popup_target: None,
        }
    }
}*/

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum HitTestResult {
    Failed,
    Passed,
    Skipped,
}

/// Helper function to perform hit-test of a pointer event in the given bounds.
///
/// Returns:
/// - Skipped: if the hit test was skipped, because the kind of pointer event ignores hit test (e.g. pointerout)
/// - Passed:  if the pointer position fell in the given bounds
/// - Failed:  otherwise
fn hit_test_helper(
    pointer_event: &PointerEvent,
    layer: &LayerHandle,
    id: Option<WidgetId>,
    pointer_grab: Option<WidgetId>,
) -> HitTestResult {
    // 1. pointer out events are exempt from hit-test: if the pointer leaves
    // the parent widget, we also want the child elements to know that.
    // 2. if the widget is a pointer-grabbing widget, don't hit test
    if pointer_event.kind == PointerEventKind::PointerOut || (pointer_grab.is_some() && pointer_grab == id) {
        HitTestResult::Skipped
    } else if layer.contains(pointer_event.position) {
        HitTestResult::Passed
    } else {
        HitTestResult::Failed
    }
}

/// Used internally by `route_event`. In charge of converting the event to the target widget's local coordinates,
/// performing hit-testing, calling the `event` method on the widget with the child `EventCtx`, and handling its result.
///
/// # Arguments:
///
/// * `parent_ctx` EventCtx of the calling context
/// * `widget` the widget to send the event to
/// * `widget_id` the ID of the `widget` (equivalent to `widget.widget_id()`, but we pass it as an argument to avoid calling the function again)
/// * `event` the event to proagate
/// * `skip_hit_test`: if true, skip hit-test and unconditionally propagate the event to the widget
/// * `env` current environment
fn do_event(
    parent_ctx: &mut EventCtx,
    widget: &dyn Widget,
    widget_id: Option<WidgetId>,
    event: &mut Event,
    skip_hit_test: bool,
    env: &Environment,
) {
    let target_layer = widget.layer();
    let parent_to_target_transform = parent_ctx.layer.child_transform(target_layer);
    // transform from the visual tree root to the widget's layer
    let window_transform = parent_to_target_transform.then(&parent_ctx.window_transform);

    let mut target_ctx = EventCtx {
        app_ctx: parent_ctx.app_ctx,
        event_loop: parent_ctx.event_loop,
        parent_window: parent_ctx.parent_window.as_deref_mut(),
        focus_state: parent_ctx.focus_state,
        window_transform,
        layer: target_layer,
        scale_factor: parent_ctx.scale_factor,
        id: widget.widget_id(),
        handled: false,
        relayout: false,
        active: None,
    };
    event.with_local_coordinates(parent_to_target_transform, |event| {
        match event {
            Event::Pointer(p) if !skip_hit_test => {
                // pointer events undergo hit-testing, with some exceptions:
                // - pointer out events are exempt from hit-test: if the pointer leaves
                // the parent widget, we also want the child elements to know that.
                // - if the widget is a pointer-grabbing widget, don't hit test
                let exempt_from_hit_test = p.kind == PointerEventKind::PointerOut
                    || (widget_id.is_some() && target_ctx.focus_state.pointer_grab == widget_id);

                if exempt_from_hit_test || target_layer.contains(p.position) {
                    // hit test pass
                    widget.event(&mut target_ctx, event, env);
                } else {
                    // hit test fail, skip
                }
            }
            _ => {
                widget.event(&mut target_ctx, event, env);
            }
        }
    });

    parent_ctx.handled = target_ctx.handled;

    // -- update widget state from ctx
    // relayout and redraws
    /*if ctx.relayout {
        //tracing::trace!(widget_id = ?self.state.id, "requested relayout");
        // relayout requested by the widget: invalidate cached measurements
        self.state.layout_result.set(None);
    }*/
    // -- propagate results to parent
    //parent_ctx.relayout |= ctx.relayout;
}

/// Routes an event to a target widget.
fn route_event(parent_ctx: &mut EventCtx, widget: &dyn Widget, event: &mut Event, env: &Environment) {
    let id = widget.widget_id();

    // ---- Handle internal events (routing mostly) ----
    match *event {
        ////////////////////////////////////////////////////////////////////////////////////////
        // Routed events
        Event::Internal(InternalEvent::RouteWindowEvent {
            target,
            event: ref mut window_event,
        }) => {
            if id == Some(target) {
                do_event(
                    parent_ctx,
                    widget,
                    id,
                    &mut Event::WindowEvent(window_event.clone()),
                    false,
                    env,
                );
            } else {
                do_event(parent_ctx, widget, id, event, false, env);
            }
        }
        Event::Internal(InternalEvent::RouteEvent {
            target,
            event: ref mut inner_event,
        }) => {
            if id == Some(target) {
                do_event(parent_ctx, widget, id, inner_event, false, env);
            } else {
                do_event(parent_ctx, widget, id, event, false, env);
            }
        }
        Event::Internal(InternalEvent::RoutePointerEvent {
            target,
            event: ref mut pointer_event,
        }) => {
            // routed pointer events follow the same logic as routed events (routed to target)
            // and pointer events (converted to local coordinates), except that they are not filtered
            // by hit-testing.
            if id == Some(target) {
                //trace!("pointer event reached {:?}", target);
                do_event(parent_ctx, widget, id, &mut Event::Pointer(*pointer_event), true, env);
            } else {
                do_event(parent_ctx, widget, id, event, true, env);
            }
        }
        // TODO remove? not sure that's still used
        Event::Internal(InternalEvent::RouteRedrawRequest(target)) => {
            if id == target {
                do_event(parent_ctx, widget, id, &mut Event::WindowRedrawRequest, false, env);
            } else {
                do_event(parent_ctx, widget, id, event, false, env);
            }
        }
        Event::Internal(InternalEvent::Traverse { ref mut widgets }) => {
            // T: ?Sized
            // This is problematic: it must clone self, and thus we must either have T == dyn Widget or T:Sized
            //widgets.push(WidgetPod(self.state.this.upgrade().unwrap()));
        }

        ////////////////////////////////////////////////////////////////////////////////////////
        // Other internal events
        Event::Internal(InternalEvent::UpdateChildFilter { ref mut filter }) => {
            if let Some(id) = id {
                filter.add(&id);
            }
            // propagate
            do_event(parent_ctx, widget, id, event, false, env);
        }
        Event::Initialize => {
            // directly pass to widget
            do_event(parent_ctx, widget, id, event, false, env);
        }

        ////////////////////////////////////////////////////////////////////////////////////////
        // Regular event flow
        _ => {
            do_event(parent_ctx, widget, id, event, false, env);
        }
    }
}

pub struct EventCtx<'a> {
    pub(crate) app_ctx: &'a mut AppCtx,
    pub(crate) event_loop: &'a EventLoopWindowTarget<ExtEvent>,
    pub(crate) parent_window: Option<&'a mut kyute_shell::window::Window>,
    pub(crate) focus_state: &'a mut FocusState,
    pub(crate) window_transform: Transform,
    layer: &'a Layer,
    pub(crate) scale_factor: f64,
    pub(crate) id: Option<WidgetId>,
    pub(crate) handled: bool,
    pub(crate) relayout: bool,
    active: Option<bool>,
}

impl<'a> EventCtx<'a> {
    /// Creates the root `EventCtx`
    pub(crate) fn new(
        app_ctx: &'a mut AppCtx,
        focus_state: &'a mut FocusState,
        event_loop: &'a EventLoopWindowTarget<ExtEvent>,
        id: Option<WidgetId>,
        root_layer: &'a Layer,
    ) -> EventCtx<'a> {
        EventCtx {
            app_ctx,
            event_loop,
            parent_window: None,
            focus_state,
            window_transform: Transform::identity(),
            scale_factor: 1.0,
            layer: root_layer,
            id,
            handled: false,
            relayout: false,
            active: None,
        }
    }

    /// Creates a new `EventCtx` to propagate events in a subwindow.
    pub fn with_local_transform<R>(
        &mut self,
        transform: Transform,
        event: &mut Event,
        f: impl FnOnce(&mut EventCtx, &mut Event) -> R,
    ) -> R {
        let prev_window_transform = self.window_transform;
        self.window_transform = transform.then(&self.window_transform);
        let result = event.with_local_coordinates(transform, |event| f(self, event));
        self.window_transform = prev_window_transform;
        result
    }

    /// Creates a new `EventCtx` to propagate events in a subwindow.
    pub(crate) fn new_subwindow<'b>(
        parent: &'b mut EventCtx,
        scale_factor: f64,
        window: &'b mut kyute_shell::window::Window,
        root_layer: &'b Layer,
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
            window_transform: Transform::identity(),
            scale_factor,
            id: parent.id,
            layer: root_layer,
            handled: false,
            relayout: false,
            active: None,
        }
    }

    /// Performs hit-testing of the specified event in the given sub-bounds.
    ///
    /// The behavior of hit-testing is as follows:
    /// - if the event is not a pointer event, the hit-test passes automatically
    /// - otherwise, do the pointer event hit-test
    ///     - TODO details
    ///
    /// The function returns whether the hit-test passed, and, if it was successful and the event
    /// was a pointer event, the pointer event with coordinates relative to the given sub-bounds.
    /*pub fn hit_test(&mut self, pointer_event: &PointerEvent, bounds: Rect) -> HitTestResult {
        hit_test_helper(pointer_event, bounds, self.id, self.focus_state.pointer_grab)
    }*/

    /// Returns the parent widget ID.
    pub fn widget_id(&self) -> Option<WidgetId> {
        self.id
    }

    pub fn window_transform(&self) -> &Transform {
        &self.window_transform
    }

    pub fn register_window(&mut self, window_id: WindowId) {
        if let Some(id) = self.id {
            self.app_ctx.register_window_widget(window_id, id);
        } else {
            warn!("register_window: the widget registering the window must have an ID")
        }
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

    /// Requests a relayout of the current widget.
    pub fn request_relayout(&mut self) {
        self.relayout = true;
    }

    /// Requests that the current node grabs all pointer events in the parent window.
    pub fn capture_pointer(&mut self) {
        if let Some(id) = self.id {
            self.focus_state.pointer_grab = Some(id);
        } else {
            warn!("capture_pointer: the widget capturing the pointer must have an ID")
        }
    }

    /// Returns whether the current node is capturing the pointer.
    #[must_use]
    pub fn is_capturing_pointer(&self) -> bool {
        if let Some(id) = self.id {
            self.focus_state.pointer_grab == Some(id)
        } else {
            false
        }
    }

    /// Releases the pointer grab, if the current node is holding it.
    pub fn release_pointer(&mut self) {
        if let Some(id) = self.id {
            if self.focus_state.pointer_grab == Some(id) {
                trace!("releasing pointer grab");
            } else {
                warn!("pointer capture release requested but the current widget isn't capturing the pointer");
            }
        } else {
            warn!("capture_pointer: the calling widget must have an ID")
        }
    }

    /// Acquires the focus.
    pub fn request_focus(&mut self) {
        if let Some(id) = self.id {
            self.focus_state.focus = Some(id);
        } else {
            warn!("request_focus: the calling widget must have an ID")
        }
    }

    /// Returns whether the current node has the focus.
    #[must_use]
    pub fn has_focus(&self) -> bool {
        if let Some(id) = self.id {
            self.focus_state.focus == Some(id)
        } else {
            false
        }
    }

    pub fn track_popup_menu(&mut self, menu: kyute_shell::Menu, at: Point) {
        if let Some(id) = self.id {
            self.focus_state.popup_target = Some(id);
            let at = ((at.x * self.scale_factor) as i32, (at.y * self.scale_factor) as i32);
            self.parent_window
                .as_mut()
                .expect("EventCtx::track_popup_menu called without a parent window")
                .show_context_menu(menu, at);
        } else {
            warn!("track_popup_menu: the calling widget must have an ID")
        }
    }

    /// Signals that the passed event was handled and should not bubble up further.
    pub fn set_handled(&mut self) {
        self.handled = true;
    }

    /// Signals that the widget became active or inactive.
    pub fn set_active(&mut self, active: bool) {
        self.active = Some(active);
    }

    #[must_use]
    pub fn handled(&self) -> bool {
        self.handled
    }

    /// Route event to a child widget.
    pub fn default_route_event(&mut self, widget: &dyn Widget, event: &mut Event, env: &Environment) {
        route_event(self, widget, event, env)
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

    #[must_use]
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
    /// Returns the widget identity.
    fn widget_id(&self) -> Option<WidgetId>;

    /// Returns this widget's animation layer.
    fn layer(&self) -> &LayerHandle;

    /// Measures this widget and layouts the children of this widget.
    fn layout(&self, ctx: &mut LayoutCtx, constraints: BoxConstraints, env: &Environment) -> Measurements;

    /// Propagates an event through the widget hierarchy.
    fn event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment);

    ///
    fn route_event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        ctx.default_route_event(self, event, env)
    }

    /// Called only for native window widgets.
    fn window_paint(&self, _ctx: &mut WindowPaintCtx) {}

    /// Called for custom GPU operations
    fn gpu_frame<'a, 'b>(&'a self, _ctx: &mut GpuFrameCtx<'a, 'b>) {}

    /// Implement to give a debug name to your widget. Used only for debugging.
    fn debug_name(&self) -> &str {
        std::any::type_name::<Self>()
    }
}

/// Arc'd widgets.
impl<T: Widget + ?Sized> Widget for Arc<T> {
    fn widget_id(&self) -> Option<WidgetId> {
        Widget::widget_id(&**self)
    }

    fn layer(&self) -> &LayerHandle {
        Widget::layer(&**self)
    }

    fn debug_name(&self) -> &str {
        Widget::debug_name(&**self)
    }

    fn layout(&self, ctx: &mut LayoutCtx, constraints: BoxConstraints, env: &Environment) -> Measurements {
        Widget::layout(&**self, ctx, constraints, env)
    }

    fn event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        Widget::event(&**self, ctx, event, env)
    }

    fn window_paint(&self, ctx: &mut WindowPaintCtx) {
        Widget::window_paint(&**self, ctx)
    }

    fn gpu_frame<'a, 'b>(&'a self, ctx: &mut GpuFrameCtx<'a, 'b>) {
        Widget::gpu_frame(&**self, ctx)
    }
}

/// Extension methods on widgets.
pub trait WidgetExt: Widget + Sized + 'static {
    /*/// Wraps the widget in a `ConstrainedBox` that constrains the width of the widget.
    #[composable]
    fn constrain_width(self, width: impl RangeBounds<f64>) -> ConstrainedBox<Self> {
        ConstrainedBox::new(BoxConstraints::new(width, ..), self)
    }

    /// Wraps the widget in a `ConstrainedBox` that constrains the height of the widget.
    #[composable]
    fn constrain_height(self, height: impl RangeBounds<f64>) -> ConstrainedBox<Self> {
        ConstrainedBox::new(BoxConstraints::new(.., height), self)
    }*/

    /// Wraps the widget in a `ConstrainedBox` that constrains the width of the widget.
    #[must_use]
    fn fix_width(self, width: impl Into<Length>) -> ConstrainedBox<Self> {
        let width = width.into();
        ConstrainedBox::new(self).min_width(width).max_width(width)
    }

    /// Wraps the widget in a `ConstrainedBox` that constrains the height of the widget.
    #[must_use]
    fn fix_height(self, height: impl Into<Length>) -> ConstrainedBox<Self> {
        let height = height.into();
        ConstrainedBox::new(self).min_height(height).max_height(height)
    }
    /// Wraps the widget in a `ConstrainedBox` that constrains the size of the widget.
    #[must_use]
    fn fix_size(self, width: impl Into<Length>, height: impl Into<Length>) -> ConstrainedBox<Self> {
        let width = width.into();
        let height = height.into();
        ConstrainedBox::new(self)
            .min_width(width)
            .max_width(width)
            .min_height(height)
            .max_height(height)
    }

    /// Wraps the widget in a `ConstrainedBox` that fills the available space in the parent widget.
    #[must_use]
    fn fill(self) -> ConstrainedBox<Self> {
        self.fix_size(100.percent(), 100.percent())
    }

    /// Centers the widget in the available space.
    #[must_use]
    fn centered(self) -> Align<Self> {
        Align::new(Alignment::CENTER, self)
    }

    /// Aligns the widget in the available space.
    #[must_use]
    fn aligned(self, alignment: Alignment) -> Align<Self> {
        Align::new(alignment, self)
    }

    /// Adds padding around the widget.
    #[must_use]
    fn padding_left(self, left: impl Into<Length>) -> Padding<Self> {
        Padding::new(0.dip(), 0.dip(), 0.dip(), left, self)
    }

    /// Adds padding around the widget.
    #[must_use]
    fn padding_right(self, right: impl Into<Length>) -> Padding<Self> {
        Padding::new(0.dip(), right, 0.dip(), 0.dip(), self)
    }

    /// Adds padding around the widget.
    #[must_use]
    fn padding_top(self, top: impl Into<Length>) -> Padding<Self> {
        Padding::new(top, 0.dip(), 0.dip(), 0.dip(), self)
    }

    /// Adds padding around the widget.
    #[must_use]
    fn padding_bottom(self, bottom: impl Into<Length>) -> Padding<Self> {
        Padding::new(0.dip(), 0.dip(), bottom, 0.dip(), self)
    }

    /// Adds padding around the widget.
    #[must_use]
    fn padding(
        self,
        top: impl Into<Length>,
        right: impl Into<Length>,
        bottom: impl Into<Length>,
        left: impl Into<Length>,
    ) -> Padding<Self> {
        Padding::new(top, right, bottom, left, self)
    }
}

impl<W: Widget + 'static> WidgetExt for W {}

/// ID of a node in the tree.
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
#[repr(transparent)]
pub struct WidgetId(CallId);

impl WidgetId {
    pub(crate) fn from_call_id(call_id: CallId) -> WidgetId {
        WidgetId(call_id)
    }

    #[composable]
    pub fn here() -> WidgetId {
        WidgetId(cache::current_call_id())
    }
}

impl fmt::Debug for WidgetId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:04X}", self.0.to_u64())
    }
}

pub type WidgetFilter = Bloom<WidgetId>;

#[derive(Copy, Clone, Debug, Hash)]
struct LayoutResult {
    constraints: BoxConstraints,
    measurements: Measurements,
}

#[derive(Clone)]
struct WidgetPodState {
    /// Unique ID of the widget, if it has one.
    id: Option<WidgetId>,

    /// Layout result.
    // TODO(layers): remove?
    layout_result: Cell<Option<LayoutResult>>,

    /// Any pointer hovering this widget
    pointer_over: State<bool>,

    /// Bloom filter to filter child widgets.
    child_filter: Cell<Option<WidgetFilter>>,
}

/// A container for a widget.
/// TODO fix the docs.
/// TODO I'm not sure that we should allow it to be Clone-able
#[derive(Clone)]
pub struct WidgetPod<T: ?Sized = dyn Widget> {
    state: WidgetPodState,
    widget: T,
}

impl<T: Widget + ?Sized> WidgetPod<T> {
    fn compute_child_filter(&self, parent_ctx: &mut EventCtx, env: &Environment) -> Bloom<WidgetId> {
        if let Some(filter) = self.state.child_filter.get() {
            // already computed
            filter
        } else {
            //tracing::trace!("computing child filter");
            let mut filter = Default::default();
            do_event(
                parent_ctx,
                &self.widget,
                self.state.id,
                &mut Event::Internal(InternalEvent::UpdateChildFilter { filter: &mut filter }),
                false,
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
            warn!("`may_contain` called but child filter not initialized");
            true
        }
    }
}

impl<T: Widget + ?Sized> Widget for WidgetPod<T> {
    fn widget_id(&self) -> Option<WidgetId> {
        self.state.id
    }

    fn debug_name(&self) -> &str {
        self.widget().debug_name()
    }

    fn layer(&self) -> &LayerHandle {
        todo!()
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
                if Some(target) != self.state.id && !self.may_contain(target) {
                    return;
                }
            }
            // for UpdateChildFilter, if we already have computed and cached the child filter, use that
            // instead of propagating down the tree.
            Event::Internal(InternalEvent::UpdateChildFilter { ref mut filter }) => {
                if let Some(id) = self.state.id {
                    filter.add(&id);
                }
                let child_filter = self.compute_child_filter(parent_ctx, env);
                filter.extend(&child_filter);
                return;
            }
            _ => {}
        }

        // continue with default routing behavior
        parent_ctx.default_route_event(self, event, env)
    }

    fn event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        self.widget.event(ctx, event, env)
    }

    fn layout(&self, ctx: &mut LayoutCtx, constraints: BoxConstraints, env: &Environment) -> Measurements {
        // we just forward to the inner widget; we also check for invalid size values while we're at
        // it, but that's only for debugging convenience.
        let measurements = self.widget.layout(ctx, constraints, env);

        if !measurements.size.width.is_finite() || !measurements.size.height.is_finite() {
            warn!(
                "layout[{:?}({})] returned non-finite measurements: {:?}",
                self.state.id,
                self.widget.debug_name(),
                measurements
            );
        }

        measurements
    }

    fn window_paint(&self, ctx: &mut WindowPaintCtx) {
        self.widget.window_paint(ctx);
    }

    fn gpu_frame<'a, 'b>(&'a self, _ctx: &mut GpuFrameCtx<'a, 'b>) {
        todo!()
    }
}

impl fmt::Debug for WidgetPod {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // TODO
        f.debug_tuple("WidgetPod").finish()
    }
}

impl<T: Widget + 'static> WidgetPod<T> {
    /// Creates a new `WidgetPod` wrapping the specified widget.
    #[composable]
    pub fn new(widget: T) -> WidgetPod<T> {
        let id = widget.widget_id();

        WidgetPod {
            state: WidgetPodState {
                id,
                pointer_over: state(|| false),
                layout_result: Cell::new(None),
                child_filter: Cell::new(None),
            },
            widget,
        }
    }
}

/*impl<T: Widget + ?Sized> Deref for WidgetPod<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.widget
    }
}

impl<T: Widget + ?Sized> DerefMut for WidgetPod<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.widget
    }
}*/

/*// TODO remove this once we have unsized coercions
impl<T: Widget + 'static> From<WidgetPod<T>> for WidgetPod {
    fn from(other: WidgetPod<T>) -> Self {
        WidgetPod(other.0)
    }
}*/

impl<T: ?Sized + Widget> WidgetPod<T> {
    /// Returns a reference to the wrapped widget.
    pub fn widget(&self) -> &T {
        &self.widget
    }

    /// Returns the widget id.
    pub fn id(&self) -> Option<WidgetId> {
        self.state.id
    }

    /// Prepares the root `EventCtx` and calls `self.event()`.
    pub(crate) fn send_root_event(
        &self,
        app_ctx: &mut AppCtx,
        event_loop: &EventLoopWindowTarget<ExtEvent>,
        event: &mut Event,
        env: &Environment,
    ) {
        // FIXME callId?
        // The dummy `FocusState` for the root `EventCtx`. It is eventually replaced with the `FocusState`
        // managed by `Window` widgets.
        let mut dummy_focus_state = FocusState::default();
        let mut event_ctx = EventCtx::new(app_ctx, &mut dummy_focus_state, event_loop, None);
        //tracing::trace!("event={:?}", event);
        self.event(&mut event_ctx, event, env);
    }

    /// Initializes and layouts the widget if necessary (propagates the `Initialize` event and
    /// calls `root_layout`.
    pub(crate) fn initialize(
        &self,
        app_ctx: &mut AppCtx,
        event_loop: &EventLoopWindowTarget<ExtEvent>,
        env: &Environment,
    ) {
        let mut dummy_focus_state = FocusState::default();
        let mut event_ctx = EventCtx::new(app_ctx, &mut dummy_focus_state, event_loop, None, self.layer());
        self.event(&mut event_ctx, &mut Event::Initialize, env);
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
