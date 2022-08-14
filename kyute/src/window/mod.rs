mod key_code;

use crate::{
    cache, composable,
    core::{DebugNode, EventResult, FocusChange, FocusState},
    drawing::PaintCtx,
    event::{InputState, KeyboardEvent, PointerButton, PointerEvent, PointerEventKind, WheelDeltaMode, WheelEvent},
    graal,
    graal::vk::Handle,
    region::Region,
    style::WidgetState,
    widget::{Menu, WidgetPod},
    Data, Environment, Event, EventCtx, Geometry, InternalEvent, LayoutCtx, LayoutParams, Point, Size, Widget,
    WidgetId,
};
use keyboard_types::{KeyState, Modifiers};
use kyute_shell::{
    application::Application,
    winit,
    winit::{
        event::{DeviceId, MouseScrollDelta, WindowEvent},
        window::WindowBuilder,
    },
};
use skia_safe as sk;
use std::{cell::RefCell, collections::HashSet, mem, sync::Arc, time::Instant};
use tracing::trace;

////////////////////////////////////////////////////////////////////////////////////////////////////
// Skia utils
////////////////////////////////////////////////////////////////////////////////////////////////////

pub(crate) fn skia_get_proc_addr(of: sk::gpu::vk::GetProcOf) -> sk::gpu::vk::GetProcResult {
    unsafe {
        let entry = graal::get_vulkan_entry();
        let instance = graal::get_vulkan_instance();

        match of {
            sk::gpu::vk::GetProcOf::Instance(instance, name) => entry
                .get_instance_proc_addr(graal::vk::Instance::from_raw(instance as u64), name)
                .unwrap() as sk::gpu::vk::GetProcResult,
            sk::gpu::vk::GetProcOf::Device(device, name) => instance
                .get_device_proc_addr(graal::vk::Device::from_raw(device as u64), name)
                .unwrap() as sk::gpu::vk::GetProcResult,
        }
    }
}

pub(crate) unsafe fn create_skia_vulkan_backend_context(
    device: &graal::Device,
) -> sk::gpu::vk::BackendContext<'static> {
    let vk_device = device.device.handle();
    let vk_instance = graal::get_vulkan_instance().handle();
    let vk_physical_device = device.physical_device();
    let (vk_queue, vk_queue_family_index) = device.graphics_queue();
    let instance_extensions = graal::get_instance_extensions();

    let mut ctx = sk::gpu::vk::BackendContext::new_with_extensions(
        vk_instance.as_raw() as *mut _,
        vk_physical_device.as_raw() as *mut _,
        vk_device.as_raw() as *mut _,
        (vk_queue.as_raw() as *mut _, vk_queue_family_index as usize),
        &skia_get_proc_addr,
        instance_extensions,
        &[],
    );

    ctx.set_max_api_version(sk::gpu::vk::Version::new(1, 0, 0));
    ctx
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Window state & event handling
////////////////////////////////////////////////////////////////////////////////////////////////////

/// Stores information about the last click (for double-click handling)
struct LastClick {
    device_id: DeviceId,
    button: PointerButton,
    position: Point,
    time: Instant,
    repeat_count: u32,
}

/// Retained state of `Window` widgets.
///
/// This is stored in the cache and mutated in place.
pub(crate) struct WindowState {
    // It's an `Option<Window>` because we can't create the window immediately during recomp;
    // due to winit's architecture, we must have a ref to the EventLoop to create one,
    // and we only pass one during event handling.
    // TODO: at some point, replace winit with our thing and delete this horror; I hate it with a passion
    pub(crate) window: Option<kyute_shell::window::Window>,
    skia_backend_context: skia_safe::gpu::vk::BackendContext<'static>,
    skia_recording_context: skia_safe::gpu::DirectContext,
    window_builder: WindowBuilder,
    pub(crate) focus_state: FocusState,
    pub(crate) hovered: HashSet<WidgetId>,
    focus_chain: Vec<WidgetId>,
    menu: Option<Menu>,
    inputs: InputState,
    last_click: Option<LastClick>,
    scale_factor: f64,
    invalid: Region,
    recomposed: bool,
}

impl WindowState {
    /// Processes a winit `WindowEvent` sent to this window.
    ///
    /// Updates various states that are tracked across WindowEvents, such as:
    /// - the current pointer position
    /// - information about the last click, for double-click handling
    ///
    /// Returns the event that should be propagated to the content widget as a result of the window event.
    fn process_window_event(&mut self, window_event: &winit::event::WindowEvent) -> Option<Event<'static>> {
        //let _span = trace_span!("process_window_event").entered();

        let _window = self
            .window
            .as_mut()
            .expect("process_window_event received but window not initialized");

        // ---------------------------------------
        // Default window event processing: update scale factor, input states (pointer pos, keyboard mods).
        // Some input events (pointer, keyboard) are also converted to normal events delivered
        // to the widgets within the window.
        match window_event {
            // don't send Character events for control characters
            WindowEvent::ReceivedCharacter(c) if !c.is_control() => {
                Some(Event::Keyboard(KeyboardEvent {
                    state: KeyState::Down,
                    key: keyboard_types::Key::Character(c.to_string()),
                    code: keyboard_types::Code::Unidentified,
                    location: keyboard_types::Location::Standard,
                    modifiers: self.inputs.modifiers,
                    // TODO
                    repeat: false,
                    is_composing: false,
                }))
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                self.scale_factor = *scale_factor;
                None
            }
            WindowEvent::Resized(_size) => None,
            WindowEvent::Focused(true) => {
                // TODO
                None
            }
            WindowEvent::Focused(false) => {
                // TODO
                None
            }
            WindowEvent::Command(id) => {
                // send to popup menu target if any
                if let Some(target) = self.focus_state.popup_target.take() {
                    Some(Event::Internal(InternalEvent::RouteEvent {
                        target,
                        event: Box::new(Event::MenuCommand(*id)),
                    }))
                } else {
                    // command from the window menu
                    // find matching action and trigger it
                    if let Some(ref menu) = self.menu {
                        if let Some(action) = menu.find_action_by_index(*id) {
                            action.triggered.signal(());
                        }
                    }
                    None
                }
            }
            WindowEvent::KeyboardInput {
                device_id: _,
                input,
                is_synthetic: _,
            } => {
                let (key, code) = key_code::key_code_from_winit(input);
                Some(Event::Keyboard(KeyboardEvent {
                    state: match input.state {
                        winit::event::ElementState::Pressed => keyboard_types::KeyState::Down,
                        winit::event::ElementState::Released => keyboard_types::KeyState::Up,
                    },
                    key,
                    code,
                    location: keyboard_types::Location::default(),
                    modifiers: self.inputs.modifiers,
                    repeat: false,
                    is_composing: false,
                }))
            }
            WindowEvent::ModifiersChanged(mods) => {
                let mut modifiers = Modifiers::empty();
                if mods.ctrl() {
                    modifiers |= Modifiers::CONTROL;
                }
                if mods.shift() {
                    modifiers |= Modifiers::SHIFT;
                }
                if mods.alt() {
                    modifiers |= Modifiers::ALT;
                }
                if mods.logo() {
                    modifiers |= Modifiers::SUPER;
                }
                self.inputs.modifiers = modifiers;
                None
            }
            WindowEvent::CursorMoved {
                device_id, position, ..
            } => {
                let logical_position: (f64, f64) = position.to_logical::<f64>(self.scale_factor).into();
                let logical_position = Point::new(logical_position.0, logical_position.1);
                let pointer_state = self.inputs.pointers.entry(*device_id).or_default();
                pointer_state.position = logical_position;
                Some(Event::Pointer(PointerEvent {
                    kind: PointerEventKind::PointerMove,
                    target: None,
                    position: logical_position,
                    window_position: logical_position,
                    modifiers: self.inputs.modifiers,
                    buttons: pointer_state.buttons,
                    pointer_id: *device_id,
                    button: None,
                    repeat_count: 0,
                }))
            }
            WindowEvent::CursorEntered { .. } => {
                // TODO
                None
            }
            WindowEvent::CursorLeft { .. } => {
                // TODO
                None
            }
            WindowEvent::MouseWheel {
                device_id,
                delta,
                phase: _,
                ..
            } => {
                let pointer_state = self.inputs.pointers.entry(*device_id).or_default();
                let pointer = PointerEvent {
                    kind: PointerEventKind::PointerMove, // TODO don't care?
                    target: None,
                    position: pointer_state.position,
                    window_position: pointer_state.position,
                    modifiers: self.inputs.modifiers,
                    buttons: pointer_state.buttons,
                    pointer_id: *device_id,
                    button: None,
                    repeat_count: 0,
                };

                let wheel_event = match *delta {
                    MouseScrollDelta::LineDelta(x, y) => Event::Wheel(WheelEvent {
                        pointer,
                        delta_x: x as f64,
                        delta_y: y as f64,
                        delta_z: 0.0,
                        delta_mode: WheelDeltaMode::Line,
                    }),
                    MouseScrollDelta::PixelDelta(pos) => {
                        let (delta_x, delta_y): (f64, f64) = pos.to_logical::<f64>(self.scale_factor).into();
                        Event::Wheel(WheelEvent {
                            pointer,
                            delta_x,
                            delta_y,
                            delta_z: 0.0,
                            delta_mode: WheelDeltaMode::Pixel,
                        })
                    }
                };
                Some(wheel_event)
            }
            WindowEvent::MouseInput {
                device_id,
                state,
                button,
                ..
            } => {
                let pointer_state = self.inputs.pointers.entry(*device_id).or_default();
                let button = match button {
                    winit::event::MouseButton::Left => PointerButton::LEFT,
                    winit::event::MouseButton::Right => PointerButton::RIGHT,
                    winit::event::MouseButton::Middle => PointerButton::MIDDLE,
                    winit::event::MouseButton::Other(3) => PointerButton::X1,
                    winit::event::MouseButton::Other(4) => PointerButton::X2,
                    winit::event::MouseButton::Other(b) => PointerButton(*b as u16),
                };
                match state {
                    winit::event::ElementState::Pressed => pointer_state.buttons.set(button),
                    winit::event::ElementState::Released => pointer_state.buttons.reset(button),
                };

                let click_time = Instant::now();

                // determine the repeat count (double-click, triple-click, etc.) for button down event
                let repeat_count = match &mut self.last_click {
                    Some(ref mut last)
                        if last.device_id == *device_id
                            && last.button == button
                            && last.position == pointer_state.position
                            && (click_time - last.time) < Application::instance().double_click_time() =>
                    {
                        // same device, button, position, and within the platform specified double-click time
                        match state {
                            winit::event::ElementState::Pressed => {
                                last.repeat_count += 1;
                                last.repeat_count
                            }
                            winit::event::ElementState::Released => {
                                // no repeat for release events (although that could be possible?),
                                1
                            }
                        }
                    }
                    other => {
                        // no match, reset
                        match state {
                            winit::event::ElementState::Pressed => {
                                *other = Some(LastClick {
                                    device_id: *device_id,
                                    button,
                                    position: pointer_state.position,
                                    time: click_time,
                                    repeat_count: 1,
                                });
                            }
                            winit::event::ElementState::Released => {
                                *other = None;
                            }
                        };
                        1
                    }
                };

                Some(Event::Pointer(PointerEvent {
                    kind: match state {
                        winit::event::ElementState::Pressed => PointerEventKind::PointerDown,
                        winit::event::ElementState::Released => PointerEventKind::PointerUp,
                    },
                    target: None,
                    position: pointer_state.position,
                    window_position: pointer_state.position,
                    modifiers: self.inputs.modifiers,
                    buttons: pointer_state.buttons,
                    pointer_id: *device_id,
                    button: Some(button),
                    repeat_count,
                }))
            }
            winit::event::WindowEvent::TouchpadPressure { .. } => None,
            winit::event::WindowEvent::AxisMotion { .. } => None,
            winit::event::WindowEvent::Touch(_) => None,
            winit::event::WindowEvent::ThemeChanged(_) => None,
            _ => None,
        }
    }

    /// Updates the window menu if the window is created.
    fn update_menu(&mut self) {
        if let Some(ref mut window) = self.window {
            if let Some(ref menu) = self.menu {
                menu.assign_menu_item_indices();
                let m = menu.to_shell_menu(false);
                window.set_menu(Some(m));
            } else {
                window.set_menu(None);
            }
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Propagation of events to the window content
////////////////////////////////////////////////////////////////////////////////////////////////////

struct ContentEventCtx<'a, 'b> {
    state: &'a mut WindowState,
    content: &'a WidgetPod,
    event_ctx: &'a mut EventCtx<'b>,
    env: &'a Environment,
}

impl<'a, 'b> ContentEventCtx<'a, 'b> {
    /// Propagates an event to the content widget.
    ///
    /// Returns possible focus change requested emitted by widgets in this window.
    fn send_event(&mut self, event: &mut Event) -> EventResult {
        crate::core::send_event_with_parent_window(self.event_ctx, self.state, self.content, event, self.env)
    }

    fn send_routed_event(&mut self, target: WidgetId, event: Event) -> EventResult {
        let mut event = Event::Internal(InternalEvent::RouteEvent {
            target,
            event: Box::new(event),
        });
        self.send_event(&mut event)
    }

    /// Sends a "targeting pointer event" to a widget.
    ///
    /// They are pointer events that are intended for one widget only.
    /// Unlike other pointer events, they *do not* propagate to descendants on a successful hit-test.
    ///
    /// This is used for sending `Pointer{Out,Over,Enter,Exit}` events.
    fn send_targeting_pointer_event(&mut self, device_id: DeviceId, target: WidgetId, event_kind: PointerEventKind) {
        // synthesize a pointer event
        let event = self.state.inputs.pointers.get(&device_id).map(|state| PointerEvent {
            kind: event_kind,
            target: Some(target),
            position: state.position,
            window_position: state.position,
            modifiers: self.state.inputs.modifiers,
            buttons: state.buttons,
            pointer_id: device_id,
            button: None,
            repeat_count: 0,
        });
        if let Some(event) = event {
            let mut event = Event::Internal(InternalEvent::RoutePointerEvent { target, event });
            // NOTE: the result of synthetic pointer events are ignores
            self.send_event(&mut event);
        }
    }

    fn propagate_input_event(&mut self, mut event: Event) {
        let mut event_result = EventResult::default();

        let pointer_grab_auto_release = matches!(
            event,
            Event::Pointer(PointerEvent {
                kind: PointerEventKind::PointerUp,
                ..
            })
        );

        // send the event
        match event {
            Event::Pointer(_) | Event::Wheel(_) => {
                let pointer_id = match event {
                    Event::Pointer(ref pointer_event) => pointer_event.pointer_id,
                    Event::Wheel(ref wheel_event) => wheel_event.pointer.pointer_id,
                    _ => unreachable!(),
                };

                // FIXME: wheel event propagation is broken
                //pointer_device_id = Some(pointer_event.pointer_id);

                // Pointer and wheel events are delivered to the node that is currently grabbing the pointer.
                // If nothing is grabbing the pointer, the pointer event is delivered to a widget
                // that passes the hit-test
                if let Some(target) = self.state.focus_state.pointer_grab {
                    trace!("routing pointer event to pointer-capturing widget {:?}", target);
                    match event {
                        Event::Pointer(ref pointer_event) => {
                            self.send_event(&mut Event::Internal(InternalEvent::RoutePointerEvent {
                                event: pointer_event.clone(),
                                target,
                            }));
                        }
                        Event::Wheel(ref wheel_event) => {
                            self.send_event(&mut Event::Internal(InternalEvent::RouteWheelEvent {
                                event: wheel_event.clone(),
                                target,
                            }));
                        }
                        _ => unreachable!(),
                    }
                } else {
                    let old_hot = self.state.focus_state.hot;
                    let old_hovered = mem::take(&mut self.state.hovered);

                    // send event to computed target
                    event_result = self.send_event(&mut event);

                    let new_hot = self.state.focus_state.hot;
                    let new_hovered = mem::take(&mut self.state.hovered);

                    /*self.send_event(&mut Event::Internal(InternalEvent::HitTest {
                        hot: &mut hot,
                        hovered: &mut hovered,
                        position: pointer_event.position,
                    }));*/

                    // signal hot widget changes (PointerOver/PointerOut)
                    if old_hot != new_hot {
                        trace!("Old hot: {:?}, new hot: {:?}", old_hot, new_hot);
                        if let Some(old_and_busted) = old_hot {
                            self.send_targeting_pointer_event(pointer_id, old_and_busted, PointerEventKind::PointerOut);
                        }

                        if let Some(new_hotness) = new_hot {
                            self.send_targeting_pointer_event(pointer_id, new_hotness, PointerEventKind::PointerOver);
                        }
                    }

                    // Enter/exit events (PointerEnter/PointerExit)
                    if old_hovered != new_hovered {
                        trace!("Old hovered: {:?}, new hovered: {:?}", old_hovered, new_hovered);
                    }

                    for old_and_busted in old_hovered.difference(&new_hovered) {
                        self.send_targeting_pointer_event(pointer_id, *old_and_busted, PointerEventKind::PointerExit);
                    }
                    for new_hotness in new_hovered.difference(&old_hovered) {
                        self.send_targeting_pointer_event(pointer_id, *new_hotness, PointerEventKind::PointerEnter);
                    }

                    self.state.hovered = new_hovered;
                    self.state.focus_state.hot = new_hot;
                }
            }
            Event::Keyboard(_) => {
                // keyboard events are delivered to the widget that has the focus.
                // if no widget has focus, the event is dropped.
                if let Some(focus) = self.state.focus_state.focus {
                    event_result = self.send_routed_event(focus, event);
                }
            }
            _ => {
                warn!("unhandled processed window event {:?}", event)
            }
        };

        //------------------------------------------------
        // force release pointer grab on pointer up
        if pointer_grab_auto_release {
            //trace!("forcing release of pointer grab");
            self.state.focus_state.pointer_grab = None;
        }

        //------------------------------------------------
        // handle focus change requests and send FocusGained/FocusLost events to involved widgets.
        if let Some(focus_change) = event_result.focus_change {
            match focus_change {
                FocusChange::MoveTo(new_focus) => {
                    if let Some(old_focus) = self.state.focus_state.focus {
                        self.send_routed_event(old_focus, Event::FocusLost);
                    }
                    self.state.focus_state.focus = Some(new_focus);
                    self.send_routed_event(new_focus, Event::FocusGained);
                }
                FocusChange::MoveNext | FocusChange::MovePrev => {
                    if let Some(old_focus) = self.state.focus_state.focus {
                        // find position in focus chain
                        if let Some(pos) = self.state.focus_chain.iter().position(|x| old_focus == *x) {
                            let chain_len = self.state.focus_chain.len();
                            let adj_pos = match focus_change {
                                FocusChange::MoveNext if pos + 1 >= chain_len => 0,
                                FocusChange::MoveNext => pos + 1,
                                FocusChange::MovePrev if pos == 0 => chain_len - 1,
                                FocusChange::MovePrev => pos - 1,
                                _ => unreachable!(),
                            };

                            let new_focus = self.state.focus_chain[adj_pos];
                            self.send_routed_event(old_focus, Event::FocusLost);
                            self.state.focus_state.focus = Some(new_focus);
                            self.send_routed_event(new_focus, Event::FocusGained);
                        }
                        // if we can't find the widget in the focus chain, that's not a bug,
                        // it's just that the widget is not part of the focus chain, but can still be focused
                        // by clicking on it directly
                    }
                }
            }
        }
    }
}

fn propagate_input_event_to_content(
    event_ctx: &mut EventCtx,
    event: Event,
    state: &mut WindowState,
    content: &WidgetPod,
    env: &Environment,
) {
    let mut ctx = ContentEventCtx {
        state,
        content,
        event_ctx,
        env,
    };
    ctx.propagate_input_event(event);
}

fn forward_event_to_content(
    event_ctx: &mut EventCtx,
    event: &mut Event,
    state: &mut WindowState,
    content: &WidgetPod,
    env: &Environment,
) {
    let mut ctx = ContentEventCtx {
        state,
        content,
        event_ctx,
        env,
    };
    ctx.send_event(event);
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Window widget
////////////////////////////////////////////////////////////////////////////////////////////////////

/// A window managed by kyute.
#[derive(Clone)]
pub struct Window {
    id: WidgetId,
    window_state: Arc<RefCell<WindowState>>,
    content: Arc<WidgetPod>,
}

impl Window {
    /// Creates a new window.
    ///
    /// TODO: explain subtleties
    #[composable]
    pub fn new(window_builder: WindowBuilder, content: impl Widget + 'static, menu: Option<Menu>) -> Window {
        // create the initial window state
        // we don't want to recreate it every time, so it only depends on the call ID.
        let window_state = cache::once(move || {
            let application = Application::instance();
            let device = application.gpu_device().clone();
            let skia_backend_context = unsafe { create_skia_vulkan_backend_context(&device) };
            let recording_context_options = skia_safe::gpu::ContextOptions::new();
            let skia_recording_context =
                skia_safe::gpu::DirectContext::new_vulkan(&skia_backend_context, &recording_context_options)
                    .expect("failed to create skia recording context");

            // --- create the root composition layer ---
            // We don't need a ref to the event loop for it, so create it here
            Arc::new(RefCell::new(WindowState {
                window: None,
                skia_backend_context,
                skia_recording_context,
                window_builder,
                focus_state: FocusState::default(),
                hovered: Default::default(),
                focus_chain: vec![],
                menu: None,
                inputs: Default::default(),
                last_click: None,
                scale_factor: 1.0, // initialized during window creation
                invalid: Default::default(),
                recomposed: true,
            }))
        });

        // update window states:
        // menu bar ...
        {
            let mut window_state = window_state.borrow_mut();
            if !window_state.menu.same(&menu) {
                //tracing::trace!("updating window menu: {:#?}", menu);
                window_state.menu = menu;
                window_state.update_menu();
            }

            // set the `recomposed` flag to indicate that we called `Window::new` and that the contents
            // might have changed
            window_state.recomposed = true;
        }
        // TODO update title, size, position, etc.

        Window {
            id: WidgetId::here(),
            window_state,
            content: Arc::new(WidgetPod::with_native_layer(content)),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// impl Widget
////////////////////////////////////////////////////////////////////////////////////////////////////

impl Widget for Window {
    fn widget_id(&self) -> Option<WidgetId> {
        Some(self.id)
    }

    fn layout(&self, _ctx: &mut LayoutCtx, _constraints: &LayoutParams, _env: &Environment) -> Geometry {
        Geometry::default()
    }

    fn event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        let mut window_state = self.window_state.borrow_mut();
        let wstate = &mut *window_state;

        match event {
            Event::Initialize => {
                // skip if the window is already created
                if wstate.window.is_some() {
                    if wstate.recomposed {
                        // propagate initialization event
                        self.content.route_event(ctx, event, env);

                        // build focus chain
                        wstate.focus_chain.clear();
                        self.content.route_event(
                            ctx,
                            &mut Event::BuildFocusChain {
                                chain: &mut wstate.focus_chain,
                            },
                            env,
                        );
                        trace!(
                            "window {:?}: {} widget(s) in focus chain",
                            self.id,
                            wstate.focus_chain.len()
                        );
                        wstate.recomposed = false;
                    }
                } else {
                    trace!("creating window");

                    // --- actually create the window ---
                    let window = kyute_shell::window::Window::from_builder(
                        ctx.event_loop.unwrap(),
                        wstate.window_builder.clone(),
                        ctx.window_state.as_ref().and_then(|ws| ws.window.as_ref()),
                    )
                    .expect("failed to create window");

                    // register it to the AppCtx, necessary so that the event loop can route window events to this widget
                    ctx.register_window(window.id());

                    // update window state
                    wstate.scale_factor = window.scale_factor();
                    wstate.window = Some(window);

                    // create the window menu
                    wstate.update_menu();
                }
            }
            Event::WindowEvent(we) => {
                let content_event = wstate.process_window_event(we);
                if let Some(content_event) = content_event {
                    propagate_input_event_to_content(ctx, content_event, wstate, &self.content, env);
                }
            }
            //Event::WindowRedrawRequest => self.do_redraw(ctx, env),
            _ => {
                // Forward any other event
                forward_event_to_content(ctx, event, wstate, &self.content, env);
            }
        }

        // FIXME: EventCtx is a mess: sometimes we have an appctx available, sometimes not.
        // FIXME: when should we relayout and repaint?

        if let Some(ref mut window) = wstate.window {
            // --- update layout ---
            {
                //let _span = trace_span!("Window relayout").entered();
                let scale_factor = window.scale_factor();
                let size = window.logical_inner_size();
                let mut layout_ctx = LayoutCtx::new(scale_factor);
                self.content.layout(
                    &mut layout_ctx,
                    &LayoutParams {
                        widget_state: WidgetState::default(),
                        scale_factor,
                        min: Size::zero(),
                        max: size,
                    },
                    env,
                );
            }

            {
                // let _span = trace_span!("Window composition layers update").entered();
                // --- update composition layers ---
                let repainted = self.content.repaint_layer(&mut wstate.skia_recording_context);
                if repainted {
                    window.set_root_composition_layer(self.content.layer().unwrap());
                }
            }
        }
    }

    fn paint(&self, _ctx: &mut PaintCtx) {
        panic!("shouldn't be called")
    }

    fn debug_node(&self) -> DebugNode {
        let window_state = self.window_state.borrow();
        DebugNode::new(format!("title: {:?}", window_state.window_builder.window.title))
    }
}
