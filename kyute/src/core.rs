use crate::{
    application::{AppCtx, ExtEvent},
    bloom::Bloom,
    cache,
    call_id::CallId,
    composable,
    drawing::PaintCtx,
    graal::vk::Handle,
    shell::{
        graal,
        winit::{event_loop::EventLoopWindowTarget, window::WindowId},
    },
    EnvKey, Environment, Event, InternalEvent, Layout, LayoutConstraints, Point, PointI, Rect, Transform,
};
use kyute::window::WindowState;
use kyute_shell::{animation::Layer, application::Application, winit};
use skia_safe as sk;
use std::{
    cell::{Ref, RefCell},
    fmt,
    hash::Hash,
    sync::Arc,
};
use tracing::{trace, warn};

pub const SHOW_DEBUG_OVERLAY: EnvKey<bool> = EnvKey::new("kyute.core.show_debug_overlay");
//pub const SELECTED: EnvKey<bool> = EnvKey::new("kyute.core.selected");
//pub const DISABLED: EnvKey<bool> = EnvKey::new("kyute.core.disabled");

#[derive(Clone, Debug)]
pub struct DebugWidgetTreeNode {
    pub name: String,
    pub debug_node: DebugNode,
    pub id: Option<WidgetId>,
    pub cached_layout: Option<Layout>,
    pub transform: Option<Transform>,
    pub children: Vec<DebugWidgetTreeNode>,
}

impl DebugWidgetTreeNode {
    /// Try to extract the base widget type name (e.g. `Container` in `kyute::widgets::Container<...>`).
    pub fn base_type_name(&self) -> &str {
        let first_angle_bracket = self.name.find('<');
        let last_double_colon = if let Some(p) = first_angle_bracket {
            self.name[0..p].rfind("::").map(|p| p + 2)
        } else {
            self.name.rfind("::").map(|p| p + 2)
        };
        &self.name[last_double_colon.unwrap_or(0)..first_angle_bracket.unwrap_or(self.name.len())]
    }
}

/// Context passed to widgets during the layout pass.
///
/// See [`Widget::layout`].
pub struct LayoutCtx {
    pub scale_factor: f64,
    pub speculative: bool,
    pub paint_damage: Option<PaintDamage>,
}

impl LayoutCtx {
    /// Creates a new `LayoutCtx`.
    pub fn new(scale_factor: f64) -> LayoutCtx {
        LayoutCtx {
            scale_factor,
            speculative: false,
            paint_damage: None,
        }
    }

    /// Signals that the current widget should be repainted as a result of a layout change.
    pub fn request_repaint(&mut self) {
        self.paint_damage = Some(PaintDamage::Repaint);
    }
}

impl LayoutCtx {
    pub fn round_to_pixel(&self, dip_length: f64) -> f64 {
        (dip_length * self.scale_factor).round()
    }
}

#[derive(Copy, Clone, Debug)]
pub struct EventResult {
    pub handled: bool,
    pub relayout: bool,
    pub paint_damage: PaintDamage,
    pub focus_change: Option<FocusChange>,
}

impl Default for EventResult {
    fn default() -> Self {
        EventResult {
            handled: false,
            relayout: false,
            paint_damage: Default::default(),
            focus_change: None,
        }
    }
}

/// Per-window state related to focus and pointer grab.
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

/*/// Helper function to perform hit-test of a pointer event in the given bounds.
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
}*/

/// Used internally by `route_event`. In charge of converting the event to the target widget's local coordinates,
/// performing hit-testing, calling the `event` method on the widget with the child `EventCtx`, and handling its result.
///
/// # Arguments:
///
/// * `parent_ctx` EventCtx of the calling context
/// * `widget` the widget to send the event to
/// * `transform` parent to target transform
/// * `widget_id` the ID of the `widget` (equivalent to `widget.widget_id()`, but we pass it as an argument to avoid calling the function again)
/// * `event` the event to propagate
/// * `skip_hit_test`: if true, skip hit-test and unconditionally propagate the event to the widget
/// * `env` current environment
fn do_event<W: Widget + ?Sized>(
    parent_ctx: &mut EventCtx,
    widget: &W,
    _widget_id: Option<WidgetId>,
    event: &mut Event,
    transform: &Transform,
    env: &Environment,
) {
    /*let target_layer = widget.layer();

    let parent_to_target_transform = if let Some(parent_layer) = parent_ctx.layer {
        if let Some(transform) = parent_layer.child_transform(target_layer) {
            transform
        } else {
            // no transform yet, the layer may be orphaned, possible if we haven't called layout yet
            // This is OK, because some events are sent before layout (e.g. Initialize). For those, we don't care
            // about the transform.
            warn!(
                "orphaned layer during event propagation: parent widget={:?}, target widget={:?}({}), event={:?}",
                parent_ctx.id,
                widget_id,
                widget.debug_name(),
                event,
            );
            //assert!(matches!(event, Event::Initialize));
            Transform::identity()
        }
    } else {
        // no parent layer, this is the root layer; it might have a transform though, so apply it
        target_layer.transform()
    };*/

    match event {
        Event::Pointer(p) => {
            trace!(
                "do_event: target={:?} pointer kind={:?} position={:?} transform offset={},{}",
                widget.debug_name(),
                p.kind,
                p.position,
                transform.m31,
                transform.m32,
            )
        }
        _ => {}
    }

    // window_transform == transform from window coordinates to widget local coordinates
    let window_transform = transform.then(&parent_ctx.window_transform);

    // setup the EventCtx for the target widget, and invoke the `Widget::event` handler.
    let mut target_ctx = EventCtx {
        app_ctx: parent_ctx.app_ctx.as_deref_mut(),
        event_loop: parent_ctx.event_loop.as_deref(),
        window_state: parent_ctx.window_state.as_deref_mut(),
        window_transform,
        id: widget.widget_id(),
        handled: false,
        relayout: false,
        paint_damage: PaintDamage::None,
        focus_change: None,
    };
    event.with_local_coordinates(transform, |event| {
        widget.event(&mut target_ctx, event, env);
    });

    // merge the results of event delivery to the parent EventCtx
    let handled = target_ctx.handled;
    let relayout = target_ctx.relayout;
    let paint_damage = target_ctx.paint_damage;
    let focus_change = target_ctx.focus_change;

    parent_ctx.relayout |= relayout;
    parent_ctx.handled |= handled;
    parent_ctx.paint_damage.merge_up(paint_damage);
    if let Some(focus_change) = focus_change {
        parent_ctx.focus_change = Some(focus_change);
    }
}

/// Damage done to the contents of a layer that possibly justifies a repaint.
///
/// TODO: check documentation and wording (do layers still exist?)
#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum PaintDamage {
    /// This layer and its sublayers are undamaged and do not need a repaint.
    None,
    /// This layer is undamaged, but one or more of its sublayers are.
    SubLayers,
    /// This layer is damaged and needs to be repainted.
    Repaint,
}

impl Default for PaintDamage {
    fn default() -> Self {
        PaintDamage::None
    }
}

impl PaintDamage {
    pub fn merge_up(&mut self, down: PaintDamage) {
        match (*self, down) {
            (PaintDamage::None, _) | (PaintDamage::SubLayers, PaintDamage::Repaint) => {
                *self = down;
            }
            _ => {}
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum FocusChange {
    MoveNext,
    MovePrev,
    MoveTo(WidgetId),
}

/// Event propagation context.
///
/// Widgets receive an `EventCtx` object in `Widget::event`, which can be used to control what should happen as a result of the event, such as:
///  - changing the currently focused widget (`request_focus`, `focus_next`, `focus_prev`).
///  - capturing the mouse pointer
///  - requesting a relayout (`request_relayout`)
///  - requesting the widget to be repainted
///  - stopping event propagation (`set_handled`)
pub struct EventCtx<'a> {
    // The Option fields are here because we sometimes send "utility events" that, for practical reasons,
    // we'd like to send without having a parent window (`parent_window`, `focus_state`) or an event loop in context (`event_loop`).
    // For those, we create an EventCtx with those fields set to None.
    //
    // Unfortunately this leads to cancerous code in the methods that access those fields (`as_deref_mut` and other atrocities).
    //
    // Alternatively, we could use another type of context for those utility events,
    // but this adds another event propagation path to the `Widget` trait which, from an ergonomic standpoint, isn't very good.
    pub(crate) app_ctx: Option<&'a mut AppCtx>,
    pub(crate) event_loop: Option<&'a EventLoopWindowTarget<ExtEvent>>,
    /// Focus state of the parent window.
    pub(crate) window_state: Option<&'a mut WindowState>,
    pub(crate) window_transform: Transform,
    pub(crate) id: Option<WidgetId>,

    // event result
    pub(crate) handled: bool,
    pub(crate) relayout: bool,
    pub(crate) paint_damage: PaintDamage,
    pub(crate) focus_change: Option<FocusChange>,
}

/// Sends an event to the specified root widget.
pub(crate) fn send_root_event(
    app_ctx: &mut AppCtx,
    event_loop: &EventLoopWindowTarget<ExtEvent>,
    widget: &dyn Widget,
    event: &mut Event,
    env: &Environment,
) -> EventResult {
    let mut ctx = EventCtx {
        app_ctx: Some(app_ctx),
        event_loop: Some(event_loop),
        window_state: None,
        window_transform: Transform::identity(),
        id: widget.widget_id(),
        handled: false,
        relayout: false,
        paint_damage: PaintDamage::None,
        focus_change: None,
    };
    widget.route_event(&mut ctx, event, env);
    EventResult {
        handled: ctx.handled,
        relayout: ctx.relayout,
        paint_damage: ctx.paint_damage,
        focus_change: ctx.focus_change,
    }
}

pub(crate) fn send_event_with_parent_window<W: Widget + ?Sized>(
    ctx: &mut EventCtx,
    window_state: &mut WindowState,
    widget: &W,
    event: &mut Event,
    env: &Environment,
) -> EventResult {
    let mut child_ctx = EventCtx {
        app_ctx: ctx.app_ctx.as_deref_mut(),
        event_loop: ctx.event_loop,
        window_state: Some(window_state),
        window_transform: Transform::identity(),
        id: ctx.id,
        handled: false,
        relayout: false,
        paint_damage: PaintDamage::None,
        focus_change: None,
    };
    widget.route_event(&mut child_ctx, event, env);
    EventResult {
        handled: child_ctx.handled,
        relayout: child_ctx.relayout,
        paint_damage: child_ctx.paint_damage,
        focus_change: child_ctx.focus_change,
    }
}

/// Sends an event to the specified root widget.
pub(crate) fn send_utility_event<W: Widget + ?Sized>(widget: &W, event: &mut Event, env: &Environment) {
    let mut ctx = EventCtx {
        app_ctx: None,
        event_loop: None,
        window_state: None,
        window_transform: Transform::identity(),
        id: widget.widget_id(),
        handled: false,
        relayout: false,
        paint_damage: PaintDamage::None,
        focus_change: None,
    };
    widget.route_event(&mut ctx, event, env);
}

impl<'a> EventCtx<'a> {
    /*///
    pub fn merge_event_result(&mut self, event_result: EventResult) {
        self.relayout |= event_result.relayout;
        self.handled |= event_result.handled;
        self.paint_damage.merge_up(event_result.paint_damage);
        if let Some(focus_change) = event_result.focus_change {
            self.focus_change = Some(focus_change);
        }
    }*/

    /// Returns the parent widget ID.
    pub fn widget_id(&self) -> Option<WidgetId> {
        self.id
    }

    pub fn window_transform(&self) -> &Transform {
        &self.window_transform
    }

    /// Requests a repaint of the widget.
    pub fn request_repaint(&mut self) {
        self.paint_damage = PaintDamage::Repaint;
    }

    /*pub fn request_layer_repaint(&mut self) {
        if self.paint_damage.is_none() {
            self.paint_damage = Some(PaintDamage::SubLayers);
        }
    }*/

    pub fn register_window(&mut self, window_id: WindowId) {
        if let Some(id) = self.id {
            self.app_ctx
                .as_deref_mut()
                .expect("invalid EventCtx call")
                .register_window_widget(window_id, id);
        } else {
            warn!("register_window: the widget registering the window must have an ID")
        }
    }

    /// Returns the bounds of the current widget.
    // TODO in what space?
    pub fn bounds(&self) -> Rect {
        todo!()
    }

    /*/// Requests a redraw of the current node and its children.
    pub fn request_redraw(&mut self) {
        self.redraw = true;
    }*/

    /// Requests a relayout of the current widget.
    pub fn request_relayout(&mut self) {
        self.relayout = true;
    }

    pub fn set_cursor_icon(&mut self, cursor_icon: winit::window::CursorIcon) {
        if let Some(window_state) = self.window_state.as_mut() {
            if let Some(window) = window_state.window.as_mut() {
                window.set_cursor_icon(cursor_icon)
            }
        }
    }

    #[track_caller]
    fn window_state(&self) -> &WindowState {
        // TODO better panic message
        self.window_state
            .as_deref()
            .expect("this method can only be called when the current widget is contained in a parent window")
    }

    #[track_caller]
    fn window_state_mut(&mut self) -> &mut WindowState {
        self.window_state
            .as_deref_mut()
            .expect("this method can only be called when the current widget is contained in a parent window")
    }

    /// Requests that the current node grabs all pointer events in the parent window.
    pub fn capture_pointer(&mut self) {
        if let Some(id) = self.id {
            // TODO this should be a request
            self.window_state_mut().focus_state.pointer_grab = Some(id);
        } else {
            warn!("capture_pointer: the widget capturing the pointer must have an ID")
        }
    }

    /// Returns whether the current node is capturing the pointer.
    #[must_use]
    pub fn is_capturing_pointer(&self) -> bool {
        if let Some(id) = self.id {
            self.window_state().focus_state.pointer_grab == Some(id)
        } else {
            false
        }
    }

    /// Returns the current pointer-grabbing widget ID.
    pub fn pointer_capturing_widget(&self) -> Option<WidgetId> {
        self.window_state().focus_state.pointer_grab
    }

    /// Releases the pointer grab, if the current node is holding it.
    pub fn release_pointer(&mut self) {
        if let Some(id) = self.id {
            if self.window_state().focus_state.pointer_grab == Some(id) {
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
            self.focus_change = Some(FocusChange::MoveTo(id));
        } else {
            warn!("request_focus: the calling widget must have an ID")
        }
    }

    /// Moves the focus to the next element in the focus chain.
    pub fn focus_next(&mut self) {
        self.focus_change = Some(FocusChange::MoveNext);
    }

    /// Moves the focus to the previous element in the focus chain.
    pub fn focus_prev(&mut self) {
        self.focus_change = Some(FocusChange::MovePrev);
    }

    /// Returns whether the current node has the focus.
    #[must_use]
    pub fn has_focus(&self) -> bool {
        if let Some(id) = self.id {
            self.window_state().focus_state.focus == Some(id)
        } else {
            false
        }
    }

    pub fn track_popup_menu(&mut self, menu: kyute_shell::Menu, at: Point) {
        if let Some(id) = self.id {
            let window_state = self.window_state_mut();
            let parent_window = window_state.window.as_mut().expect("window has not been created yet");
            window_state.focus_state.popup_target = Some(id);
            let scale_factor = parent_window.scale_factor();
            let at = PointI::new((at.x * scale_factor) as i32, (at.y * scale_factor) as i32);
            parent_window.show_context_menu(menu, at);
        } else {
            warn!("track_popup_menu: the calling widget must have an ID")
        }
    }

    /// Signals that the passed event was handled and should not bubble up further.
    pub fn set_handled(&mut self) {
        self.handled = true;
    }

    #[must_use]
    pub fn handled(&self) -> bool {
        self.handled
    }

    /// Routes an event to a target widget.
    // TODO: we could use `dyn Widget` but them we can't call the function
    // in generic contexts (e.g. with `W: Widget + ?Sized`, no way to get a `&dyn Widget` from a `&W`)
    pub fn default_route_event<W: Widget + ?Sized>(
        &mut self,
        widget: &W,
        event: &mut Event,
        transform: &Transform,
        cached_layout: Option<Layout>,
        env: &Environment,
    ) {
        let id = widget.widget_id();

        match *event {
            ////////////////////////////////////////////////////////////////////////////////////////
            // Routed events
            Event::Internal(InternalEvent::RouteWindowEvent {
                target,
                event: ref mut window_event,
            }) => {
                if id == Some(target) {
                    do_event(
                        self,
                        widget,
                        id,
                        &mut Event::WindowEvent(window_event.clone()),
                        transform,
                        env,
                    )
                } else {
                    do_event(self, widget, id, event, transform, env)
                }
            }
            Event::Internal(InternalEvent::RouteEvent {
                target,
                event: ref mut inner_event,
            }) => {
                if id == Some(target) {
                    do_event(self, widget, id, inner_event, transform, env)
                } else {
                    do_event(self, widget, id, event, transform, env)
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
                    do_event(self, widget, id, &mut Event::Pointer(*pointer_event), transform, env)
                } else {
                    do_event(self, widget, id, event, transform, env)
                }
            }
            // TODO remove? not sure that's still used
            Event::Internal(InternalEvent::RouteRedrawRequest(target)) => {
                if id == Some(target) {
                    do_event(self, widget, id, &mut Event::WindowRedrawRequest, transform, env)
                } else {
                    do_event(self, widget, id, event, transform, env)
                }
            }

            ////////////////////////////////////////////////////////////////////////////////////////
            // Debug events
            Event::Internal(InternalEvent::DumpTree { ref mut nodes }) => {
                let mut children = Vec::new();

                {
                    let mut child_event = Event::Internal(InternalEvent::DumpTree { nodes: &mut children });
                    do_event(self, widget, id, &mut child_event, transform, env);
                }

                nodes.push(DebugWidgetTreeNode {
                    name: widget.debug_name().to_string(),
                    debug_node: widget.debug_node(),
                    id: widget.widget_id(),
                    cached_layout,
                    transform: Some(transform.clone()),
                    children,
                });
            }

            ////////////////////////////////////////////////////////////////////////////////////////
            // Other internal events
            Event::Internal(InternalEvent::UpdateChildFilter { ref mut filter }) => {
                if let Some(id) = id {
                    filter.add(&id);
                }
                // propagate
                do_event(self, widget, id, event, transform, env)
            }
            Event::Initialize => {
                // directly pass to widget
                do_event(self, widget, id, event, transform, env)
            }

            /*////////////////////////////////////////////////////////////////////////////////////////
            // Non-propagating pointer events
            Event::Pointer(PointerEvent { kind, .. })
                if kind == PointerEventKind::PointerOver
                    || kind == PointerEventKind::PointerOut
                    || kind == PointerEventKind::PointerEnter
                    || kind == PointerEventKind::PointerExit =>
            {
                // A widget may choose to not handle those messages, and forward it to its children.
                // However,
                return;
            }*/

            ////////////////////////////////////////////////////////////////////////////////////////
            // Regular event flow
            _ => do_event(self, widget, id, event, transform, env),
        };
    }
}

/// Used to collect and report debug information for a widget.
///
/// See [`Widget::debug_node`].
#[derive(Clone, Debug)]
pub struct DebugNode {
    content: Option<String>,
}

impl Default for DebugNode {
    fn default() -> Self {
        DebugNode { content: None }
    }
}

impl DebugNode {
    /// Creates a new `DebugNode` that carries a description of the content of the widget, as a string.
    pub fn new(content_description: impl Into<String>) -> DebugNode {
        DebugNode {
            content: Some(content_description.into()),
        }
    }
}

pub struct LayerPaintCtx<'a> {
    pub skia_gpu_context: &'a mut sk::gpu::DirectContext,
}

impl<'a> LayerPaintCtx<'a> {
    /// Creates a painting context on the layer and paints the content it using the specified closure.
    pub fn paint_layer(&mut self, layer: &Layer, scale_factor: f64, f: impl FnOnce(&mut PaintCtx)) {
        // `layer.size()` is zero initially, and can stay that way if we did not call set_size.
        // In this case, there's nothing to paint, and return early.
        if layer.size().is_empty() {
            return;
        }

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
            self.skia_gpu_context,
            &render_target,
            sk::gpu::SurfaceOrigin::TopLeft,
            sk::ColorType::RGBA8888, // TODO
            sk::ColorSpace::new_srgb(),
            Some(&sk::SurfaceProps::new(Default::default(), sk::PixelGeometry::RGBH)),
        )
        .unwrap();
        surface.canvas().clear(sk::Color4f::new(0.0, 0.0, 0.0, 0.0));

        // invoke the provided closure
        {
            let mut paint_ctx = PaintCtx::new(&mut surface, layer, scale_factor, self.skia_gpu_context);
            f(&mut paint_ctx);
        }

        // flush the GPU frame
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
}

/// Trait that defines the behavior of a widget.
pub trait Widget {
    /// Returns the widget identity.
    fn widget_id(&self) -> Option<WidgetId>;

    fn speculative_layout(&self, ctx: &mut LayoutCtx, constraints: &LayoutConstraints, env: &Environment) -> Layout {
        let was_speculative = ctx.speculative;
        ctx.speculative = true;
        let layout = self.layout(ctx, constraints, env);
        ctx.speculative = was_speculative;
        layout
    }

    /// Measures this widget and layouts the children of this widget.
    fn layout(&self, ctx: &mut LayoutCtx, constraints: &LayoutConstraints, env: &Environment) -> Layout;

    /// Routes an event from a parent widget to this widget.
    ///
    /// This method should be called by parent widgets to propagate events to their children, instead of directly
    /// calling `event` on them. It determines whether the event is targeting the widget, and if so, invokes the `event`
    /// method. Otherwise it skips propagation.
    ///
    /// It's possible to override it to inhibit event propagation in certain cases (see WidgetPod).
    /// However, you should always call `ctx.default_route_event()`.
    fn route_event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        ctx.default_route_event(self, event, &Transform::identity(), None, env)
    }

    /// Event callback. Implement to respond to event that target this widget.
    ///
    /// All events received through this method should be routed to child widgets with `child.route_event()`.
    fn event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment);

    /// Paints this widget on the given context.
    fn paint(&self, ctx: &mut PaintCtx);

    /// Paints this widget on a native composition layer.
    fn layer_paint(&self, ctx: &mut LayerPaintCtx, layer: &Layer, scale_factor: f64) {
        ctx.paint_layer(layer, scale_factor, |ctx| self.paint(ctx))
    }

    /// Implement to give a debug name to your widget. Used only for debugging.
    fn debug_name(&self) -> &str {
        std::any::type_name::<Self>()
    }

    /// Implement to give a debug name to your widget. Used only for debugging.
    fn debug_node(&self) -> DebugNode {
        DebugNode { content: None }
    }
}

/// Arc'd widgets.
impl<T: Widget + ?Sized> Widget for Arc<T> {
    fn widget_id(&self) -> Option<WidgetId> {
        Widget::widget_id(&**self)
    }

    fn layout(&self, ctx: &mut LayoutCtx, constraints: &LayoutConstraints, env: &Environment) -> Layout {
        Widget::layout(&**self, ctx, constraints, env)
    }

    fn event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        Widget::event(&**self, ctx, event, env)
    }

    fn paint(&self, ctx: &mut PaintCtx) {
        Widget::paint(&**self, ctx)
    }

    fn route_event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        Widget::route_event(&**self, ctx, event, env)
    }

    fn layer_paint(&self, ctx: &mut LayerPaintCtx, layer: &Layer, scale_factor: f64) {
        Widget::layer_paint(&**self, ctx, layer, scale_factor)
    }

    fn debug_name(&self) -> &str {
        Widget::debug_name(&**self)
    }

    fn debug_node(&self) -> DebugNode {
        Widget::debug_node(&**self)
    }
}

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

#[derive(Clone)]
pub struct LayoutCacheInner<T: Clone> {
    constraints: LayoutConstraints,
    value: Option<T>,
}

#[derive(Clone)]
pub struct LayoutCache<T: Clone>(RefCell<LayoutCacheInner<T>>);

impl<T: Clone> Default for LayoutCache<T> {
    fn default() -> Self {
        LayoutCache::new()
    }
}

impl<T: Clone> LayoutCache<T> {
    pub fn new() -> LayoutCache<T> {
        LayoutCache(RefCell::new(LayoutCacheInner {
            constraints: Default::default(),
            value: None,
        }))
    }

    pub fn is_valid(&self) -> bool {
        self.0.borrow().value.is_some()
    }

    pub fn get(&self, constraints: &LayoutConstraints) -> Option<T> {
        let inner = self.0.borrow();
        if let Some(ref value) = inner.value {
            if inner.constraints == *constraints {
                return Some(value.clone());
            }
        }
        None
    }

    pub fn set(&self, constraints: &LayoutConstraints, value: T) {
        let mut inner = self.0.borrow_mut();
        inner.constraints = *constraints;
        (*inner).value = Some(value);
    }

    pub fn update(
        &self,
        ctx: &mut LayoutCtx,
        constraints: &LayoutConstraints,
        f: impl FnOnce(&mut LayoutCtx) -> T,
    ) -> T {
        let mut inner = self.0.borrow_mut();
        if inner.value.is_none() || inner.constraints != *constraints {
            let layout = f(ctx);
            // don't cache speculative layouts
            if !ctx.speculative {
                if inner.value.is_none() {
                    trace!("initial layout");
                } else {
                    trace!("layout update: constraints:{:?}->{:?}", inner.constraints, constraints);
                }
                inner.value = Some(layout.clone());
                inner.constraints = *constraints;
            }
            layout
        } else {
            inner.value.as_ref().unwrap().clone()
        }
    }

    pub fn get_cached(&self) -> Ref<T> {
        Ref::map(self.0.borrow(), |inner| {
            inner.value.as_ref().expect("layout not calculated")
        })
    }

    pub fn get_cached_constraints(&self) -> LayoutConstraints {
        self.0.borrow().constraints
    }

    pub fn invalidate(&self) {
        trace!("layout explicitly invalidated");
        self.0.borrow_mut().value = None;
    }
}

pub(crate) fn get_debug_widget_tree<W: Widget>(w: &W) -> DebugWidgetTreeNode {
    let mut nodes = Vec::new();
    send_utility_event(
        w,
        &mut Event::Internal(InternalEvent::DumpTree { nodes: &mut nodes }),
        &Environment::default(),
    );
    assert_eq!(nodes.len(), 1);
    nodes.into_iter().next().unwrap()
}

pub(crate) fn dump_widget_tree_rec(node: &DebugWidgetTreeNode, indent: usize, lines: &mut Vec<usize>, is_last: bool) {
    let mut pad = vec![' '; indent];
    for &p in lines.iter() {
        pad[p] = '│';
    }

    let mut msg: String = pad.into_iter().collect();
    msg += &format!("{}{}", if is_last { "└" } else { "├" }, node.base_type_name());
    if let Some(id) = node.id {
        msg += &format!("({:?})", id);
    }
    if let Some(ref content) = node.debug_node.content {
        msg += "  `";
        msg += content;
        msg += "`";
    }
    println!("{}", msg);

    if !is_last {
        lines.push(indent);
    }

    for (i, n) in node.children.iter().enumerate() {
        if i == node.children.len() - 1 {
            dump_widget_tree_rec(n, indent + 2, lines, true);
        } else {
            dump_widget_tree_rec(n, indent + 2, lines, false);
        }
    }

    if !is_last {
        lines.pop();
    }
}

pub(crate) fn dump_widget_tree<W: Widget>(w: &W) {
    let node = get_debug_widget_tree(w);
    dump_widget_tree_rec(&node, 0, &mut Vec::new(), true);
}
