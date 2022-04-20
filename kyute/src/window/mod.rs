mod key_code;

use crate::{
    align_boxes,
    animation::PaintCtx,
    application::AppCtx,
    cache, composable,
    core::{FocusState, WindowPaintCtx},
    event::{InputState, KeyboardEvent, PointerButton, PointerEvent, PointerEventKind, WheelDeltaMode, WheelEvent},
    graal,
    graal::{vk::Handle, MemoryLocation},
    region::Region,
    widget::{LayerWidget, Menu},
    Alignment, BoxConstraints, Data, Environment, Event, EventCtx, InternalEvent, LayoutCtx, Measurements, Point, Rect,
    RoundToPixel, Size, Widget, WidgetId, WidgetPod,
};
use keyboard_types::{KeyState, Modifiers};
use kyute_common::Transform;
use kyute_shell::{
    application::Application,
    winit,
    winit::{
        event::{DeviceId, MouseScrollDelta, WindowEvent},
        platform::windows::{WindowBuilderExtWindows, WindowExtWindows},
        window::WindowBuilder,
    },
};
use skia_safe as sk;
use std::{cell::RefCell, env, mem, sync::Arc, time::Instant};
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
    // TODO: at some point, replace winit with our thing and delete this horror
    window: Option<kyute_shell::window::Window>,
    skia_backend_context: skia_safe::gpu::vk::BackendContext<'static>,
    skia_recording_context: skia_safe::gpu::DirectContext,
    window_builder: Option<WindowBuilder>,
    focus_state: FocusState,
    menu: Option<Menu>,
    inputs: InputState,
    last_click: Option<LastClick>,
    scale_factor: f64,
    invalid: Region,
    recomposed: bool,
}

impl WindowState {
    fn do_event(
        &mut self,
        parent_ctx: &mut EventCtx,
        widget: &LayerWidget<Arc<WidgetPod>>,
        event: &mut Event,
        env: &Environment,
    ) {
        if let Some(ref mut window) = self.window {
            let mut content_ctx = parent_ctx.with_window(window, &mut self.focus_state);
            widget.route_event(&mut content_ctx, event, env);
        } else {
            widget.route_event(parent_ctx, event, env);
        }
    }

    /// Processes a winit `WindowEvent` sent to this window.
    ///
    /// This converts `WindowEvents` to kyute `Events` and dispatches them to target widgets.
    /// It also updates various states that are tracked across WindowEvents, such as:
    /// - the current pointer position
    /// - information about the last click, for double-click handling
    fn process_window_event(
        &mut self,
        parent_ctx: &mut EventCtx,
        content_widget: &LayerWidget<Arc<WidgetPod>>,
        window_event: &winit::event::WindowEvent,
        env: &Environment,
    ) {
        let _span = trace_span!("process_window_event").entered();

        //let _span = trace_span!("process_window_event", ?window_event).entered();

        let event = {
            let window = self
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
                WindowEvent::Resized(size) => None,
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
                    phase,
                    ..
                } => {
                    let pointer_state = self.inputs.pointers.entry(*device_id).or_default();
                    let pointer = PointerEvent {
                        kind: PointerEventKind::PointerMove, // TODO don't care?
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
        };

        if let Some(mut event) = event {
            //trace!("window event {:?}", event);

            //------------------------------------------------
            // Send event
            let old_focus = self.focus_state.focus;

            let pointer_grab_auto_release = matches!(
                event,
                Event::Pointer(PointerEvent {
                    kind: PointerEventKind::PointerUp,
                    ..
                })
            );

            match event {
                Event::Pointer(ref pointer_event)
                | Event::Wheel(WheelEvent {
                    pointer: ref pointer_event,
                    ..
                }) => {
                    // Pointer and wheel events are delivered to the node that is currently grabbing the pointer.
                    // If nothing is grabbing the pointer, the pointer event is delivered to a widget
                    // that passes the hit-test
                    if let Some(pointer_grab) = self.focus_state.pointer_grab {
                        trace!("routing pointer event to pointer-capturing widget {:?}", pointer_grab);
                        // must use RoutePointerEvent so that relative pointer positions are computed during propagation
                        self.do_event(
                            parent_ctx,
                            content_widget,
                            &mut Event::Internal(InternalEvent::RoutePointerEvent {
                                target: pointer_grab,
                                event: *pointer_event,
                            }),
                            env,
                        );
                    } else {
                        // just forward to content, will do a hit-test
                        self.do_event(parent_ctx, content_widget, &mut event, env);
                    };
                }
                Event::Keyboard(_) => {
                    // keyboard events are delivered to the widget that has the focus.
                    // if no widget has focus, the event is dropped.
                    if let Some(focus) = self.focus_state.focus {
                        self.do_event(
                            parent_ctx,
                            content_widget,
                            &mut Event::Internal(InternalEvent::RouteEvent {
                                target: focus,
                                event: Box::new(event),
                            }),
                            env,
                        );
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
                self.focus_state.pointer_grab = None;
            }

            //------------------------------------------------
            // signal focus changes
            if old_focus != self.focus_state.focus {
                let new_focus = self.focus_state.focus;
                trace!("focus changed");

                if let Some(old_focus) = old_focus {
                    self.do_event(
                        parent_ctx,
                        content_widget,
                        &mut Event::Internal(InternalEvent::RouteEvent {
                            target: old_focus,
                            event: Box::new(Event::FocusLost),
                        }),
                        env,
                    );
                }

                if let Some(new_focus) = new_focus {
                    self.do_event(
                        parent_ctx,
                        content_widget,
                        &mut Event::Internal(InternalEvent::RouteEvent {
                            target: new_focus,
                            event: Box::new(Event::FocusGained),
                        }),
                        env,
                    );
                }
            }
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
// Window widget
////////////////////////////////////////////////////////////////////////////////////////////////////

/// A window managed by kyute.
#[derive(Clone)]
pub struct Window {
    id: WidgetId,
    window_state: Arc<RefCell<WindowState>>,
    contents: LayerWidget<Arc<WidgetPod>>,
}

impl Window {
    /// Creates a new window.
    ///
    /// TODO: explain subtleties
    #[composable]
    pub fn new(window_builder: WindowBuilder, contents: impl Widget + 'static, menu: Option<Menu>) -> Window {
        // create the initial window state
        // we don't want to recreate it every time, so it only depends on the call ID.
        let window_state = cache::once(move || {
            let application = kyute_shell::application::Application::instance();
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
                window_builder: Some(window_builder),
                focus_state: FocusState::default(),
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
            contents: LayerWidget::new(Arc::new(WidgetPod::new(contents))),
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

    fn layout(&self, ctx: &mut LayoutCtx, _constraints: BoxConstraints, env: &Environment) -> Measurements {
        //self.layout_contents(ctx.app_ctx, env);
        Measurements::default()
    }

    fn event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        match event {
            Event::Initialize => {
                let mut window_state = self.window_state.borrow_mut();

                // skip if the window is already created
                if window_state.window.is_some() {
                    // if the window is created, and we haven't recomposed, don't send initialize
                    if window_state.recomposed {
                        self.contents.event(ctx, event, env);
                        window_state.recomposed = false;
                    }
                } else {
                    tracing::trace!("creating window");

                    // --- actually create the window ---
                    let mut window_builder = window_state.window_builder.take().unwrap();
                    let window = kyute_shell::window::Window::from_builder(
                        ctx.event_loop.unwrap(),
                        window_builder,
                        ctx.parent_window.as_deref(),
                    )
                    .expect("failed to create window");

                    // register it to the AppCtx, necessary so that the event loop can route window events to this widget
                    ctx.register_window(window.id());

                    // update window state
                    window_state.scale_factor = window.scale_factor();
                    window_state.window = Some(window);

                    // create the window menu
                    window_state.update_menu();
                }

                // `Initialize` is special, and we don't need to do anything else
                return;
            }
            Event::WindowEvent(window_event) => {
                let mut window_state = self.window_state.borrow_mut();
                window_state.process_window_event(ctx, &self.contents, window_event, env);
            }
            //Event::WindowRedrawRequest => self.do_redraw(ctx, env),
            _ => {
                let mut window_state = self.window_state.borrow_mut();
                let window_state = &mut *window_state; // hrmpf...

                if window_state.window.is_some() {
                    window_state.do_event(ctx, &self.contents, event, env);
                } else {
                    //tracing::warn!("received window event before initialization: {:?}", event);
                    self.contents.route_event(ctx, event, env);
                }
            }
        }

        // --- update layout ---
        let mut window_state = self.window_state.borrow_mut();
        if let Some(ref mut window) = window_state.window {
            let _span = trace_span!("window_relayout").entered();
            let scale_factor = window.scale_factor();
            let size = window.logical_inner_size();
            {
                let mut layout_ctx = LayoutCtx::new(ctx.app_ctx.as_deref_mut().unwrap(), scale_factor);
                self.contents.layout(&mut layout_ctx, BoxConstraints::loose(size), env);
            }
        }

        let window_state = &mut *window_state;
        if let Some(ref mut window) = window_state.window {
            // --- update composition layers ---
            self.contents.repaint(window_state.skia_recording_context.clone());
            window.set_root_composition_layer(self.contents.layer());
        }
    }

    fn paint(&self, ctx: &mut PaintCtx) {
        panic!("shouldn't be called")
    }
}
