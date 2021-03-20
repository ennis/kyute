use crate::{
    application::AppCtx,
    event::{
        Event, InputEvent, InputState, KeyboardEvent, PointerButton, PointerButtonEvent,
        PointerEvent, PointerState, WheelDeltaMode, WheelEvent,
    },
    node::{DebugLayout, FocusState, NodeTree, PaintOptions, RepaintRequest},
    style::StyleCollection,
    visual::WindowHandler,
    widget::DummyWidget,
    BoxConstraints, BoxedWidget, Environment, EventCtx, LayoutCtx, Measurements, Offset, PaintCtx,
    Point, Rect, Size, TypedWidget, Visual, Widget, WidgetExt,
};
use bitflags::_core::any::Any;
use generational_indextree::NodeId;
use kyute_shell::{
    drawing::Color,
    window::{PlatformWindow, WindowDrawContext},
};
use std::{rc::Rc, time::Instant};
use winit::{
    dpi::LogicalSize,
    event::{VirtualKeyCode, WindowEvent},
    window::{WindowBuilder, WindowId},
};

/// Window event callbacks.
struct Callbacks {
    on_close_requested: Option<Box<dyn Fn()>>,
    on_move: Option<Box<dyn Fn(u32, u32)>>,
    on_resize: Option<Box<dyn Fn(u32, u32)>>,
    on_focus_gained: Option<Box<dyn Fn()>>,
    on_focus_lost: Option<Box<dyn Fn()>>,
}

impl Default for Callbacks {
    fn default() -> Callbacks {
        Callbacks {
            on_close_requested: None,
            on_move: None,
            on_resize: None,
            on_focus_gained: None,
            on_focus_lost: None,
        }
    }
}

/// Stores information about the last click (for double-click handling)
struct LastClick {
    device_id: winit::event::DeviceId,
    button: PointerButton,
    position: Point,
    time: Instant,
    repeat_count: u32,
}

/// A window managed by kyute.
pub struct WindowNode {
    window: PlatformWindow,
    inputs: InputState,
    focus: FocusState,
    // for double-click detection
    last_click: Option<LastClick>,
    /// Widget styles for the window.
    style_collection: Rc<StyleCollection>,
    debug_layout: DebugLayout,
    callbacks: Callbacks,
}

impl WindowHandler for WindowNode {
    fn window(&self) -> &PlatformWindow {
        &self.window
    }

    fn window_mut(&mut self) -> &mut PlatformWindow {
        &mut self.window
    }

    fn window_event(
        &mut self,
        ctx: &mut AppCtx,
        window_event: &WindowEvent,
        tree: &mut NodeTree,
        root: NodeId,
    ) {
        let child_id = if let Some(child_id) = tree.arena[root].first_child() {
            child_id
        } else {
            // we don't deliver any events if we don't have a content widget
            return;
        };

        let event_result = match window_event {
            WindowEvent::Resized(size) => {
                let (w, h) = (*size).into();
                self.window.resize((w, h));
                self.callbacks.on_resize.as_ref().map(|f| (f)(w, h));
                return;
            }
            WindowEvent::Moved(pos) => {
                let (w, h): (u32, u32) = (*pos).into();
                self.callbacks.on_move.as_ref().map(|f| (f)(w, h));
                return;
            }
            WindowEvent::CloseRequested => {
                self.callbacks.on_close_requested.as_ref().map(|f| (f)());
                return;
            }
            WindowEvent::Focused(false) => {
                self.callbacks.on_focus_lost.as_ref().map(|f| (f)());
                return;
            }
            WindowEvent::Focused(true) => {
                self.callbacks.on_focus_gained.as_ref().map(|f| (f)());
                return;
            }
            WindowEvent::ModifiersChanged(m) => {
                self.inputs.mods = *m;
                return;
            }
            WindowEvent::MouseInput {
                device_id,
                state,
                button,
                ..
            } => {
                // update pointer state
                let pointer_state = self
                    .inputs
                    .pointers
                    .entry(*device_id)
                    .or_insert(PointerState::default());
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
                let position = pointer_state.position;

                // determine the repeat count (double-click, triple-click, etc.) for button down event
                let repeat_count = match &mut self.last_click {
                    Some(ref mut last)
                        if last.device_id == *device_id
                            && last.button == button
                            && last.position == position
                            && (click_time - last.time) < ctx.platform.double_click_time() =>
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
                                    position,
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

                let p = PointerButtonEvent {
                    pointer: PointerEvent {
                        position,
                        window_position: position,
                        modifiers: self.inputs.mods,
                        buttons: pointer_state.buttons,
                        pointer_id: *device_id,
                    },
                    button: Some(button),
                    repeat_count,
                };

                let e = match state {
                    winit::event::ElementState::Pressed => Event::PointerDown(p),
                    winit::event::ElementState::Released => Event::PointerUp(p),
                };

                tree.event(
                    ctx,
                    &self.window,
                    child_id,
                    &self.inputs,
                    &mut self.focus,
                    &e,
                )
            }
            WindowEvent::CursorMoved {
                device_id,
                position,
                ..
            } => {
                let logical = position.to_logical::<f64>(self.window.window().scale_factor());
                let logical = Point::new(logical.x, logical.y);

                let pointer_state = self
                    .inputs
                    .pointers
                    .entry(*device_id)
                    .or_insert(PointerState::default());
                pointer_state.position = logical;

                let p = PointerEvent {
                    position: logical,
                    window_position: logical,
                    modifiers: self.inputs.mods,
                    buttons: pointer_state.buttons,
                    pointer_id: *device_id,
                };

                let result = tree.event(
                    ctx,
                    &self.window,
                    child_id,
                    &self.inputs,
                    &mut self.focus,
                    &Event::PointerMove(p),
                );

                // force redraw if bounds debugging mode is on
                if self.debug_layout != DebugLayout::None {
                    RepaintRequest::Repaint
                } else {
                    result
                }
            }
            WindowEvent::MouseWheel {
                device_id,
                delta,
                phase,
                ..
            } => {
                let pointer = self.inputs.synthetic_pointer_event(*device_id);
                if let Some(pointer) = pointer {
                    let wheel_event = match delta {
                        winit::event::MouseScrollDelta::LineDelta(x, y) => WheelEvent {
                            pointer,
                            delta_x: *x as f64,
                            delta_y: *y as f64,
                            delta_z: 0.0,
                            delta_mode: WheelDeltaMode::Line,
                        },
                        winit::event::MouseScrollDelta::PixelDelta(pos) => WheelEvent {
                            pointer,
                            delta_x: pos.x,
                            delta_y: pos.y,
                            delta_z: 0.0,
                            delta_mode: WheelDeltaMode::Pixel,
                        },
                    };
                    tree.event(
                        ctx,
                        &self.window,
                        root,
                        &self.inputs,
                        &mut self.focus,
                        &Event::Wheel(wheel_event),
                    )
                } else {
                    log::warn!("wheel event received but pointer position is not yet known");
                    return;
                }
            }
            WindowEvent::ReceivedCharacter(char) => tree.event(
                ctx,
                &self.window,
                child_id,
                &self.inputs,
                &mut self.focus,
                &Event::Input(InputEvent { character: *char }),
            ),
            WindowEvent::KeyboardInput {
                device_id,
                input,
                is_synthetic,
            } => {
                let keyboard_event = KeyboardEvent {
                    scan_code: input.scancode,
                    key: input.virtual_keycode,
                    repeat: false, // TODO
                    modifiers: self.inputs.mods,
                };

                let event = match input.state {
                    winit::event::ElementState::Pressed => Event::KeyDown(keyboard_event),
                    winit::event::ElementState::Released => Event::KeyUp(keyboard_event),
                };

                // Ctrl+F12 cycles through bounds debugging modes
                if input.state == winit::event::ElementState::Pressed
                    && input.virtual_keycode == Some(VirtualKeyCode::F12)
                    && self.inputs.mods.ctrl()
                {
                    self.debug_layout = match self.debug_layout {
                        DebugLayout::None => DebugLayout::Hover,
                        DebugLayout::Hover => DebugLayout::All,
                        DebugLayout::All => DebugLayout::None,
                    };
                    RepaintRequest::Repaint
                } else {
                    tree.event(
                        ctx,
                        &self.window,
                        child_id,
                        &self.inputs,
                        &mut self.focus,
                        &event,
                    )
                }
            }

            _ => {
                return;
            }
        };

        // handle follow-up actions
        match event_result {
            RepaintRequest::Repaint | RepaintRequest::Relayout => {
                // TODO ask for relayout
                self.window.window().request_redraw();
            }
            _ => {}
        }
    }

    fn window_paint(&mut self, ctx: &mut AppCtx, tree: &mut NodeTree, anchor: NodeId) {
        {
            let id = self.window.id();
            let mut wdc = WindowDrawContext::new(&mut self.window);
            wdc.clear(Color::new(0.326, 0.326, 0.326, 1.0));

            let child_id = if let Some(child_id) = tree.arena[anchor].first_child() {
                child_id
            } else {
                // no content to paint
                return;
            };

            let options = PaintOptions {
                debug_draw_bounds: self.debug_layout,
            };
            tree.paint(
                &ctx.platform,
                &mut wdc,
                &self.style_collection,
                id,
                child_id,
                &self.inputs,
                &self.focus,
                &options,
            );
        }
        self.window.present();
    }
}

impl Visual for WindowNode {
    fn paint(&mut self, _ctx: &mut PaintCtx, _env: &Environment) {
        // we have nothing to paint in the parent window
    }

    fn hit_test(&mut self, _point: Point, _bounds: Rect) -> bool {
        unimplemented!()
    }

    fn event(&mut self, _event_ctx: &mut EventCtx, _event: &Event) {
        // we don't care about events from the parent window
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn window_handler_mut(&mut self) -> Option<&mut dyn WindowHandler> {
        Some(self)
    }
}

pub struct Window<'a> {
    builder: WindowBuilder,
    contents: BoxedWidget<'a>,
    callbacks: Callbacks,
    parent_window: Option<&'a PlatformWindow>,
}

impl<'a> Window<'a> {
    pub fn new(builder: WindowBuilder) -> Window<'a> {
        Window {
            builder,
            contents: DummyWidget.boxed(),
            callbacks: Callbacks::default(),
            parent_window: None,
        }
    }

    pub fn parent_window(mut self, parent_window: &'a PlatformWindow) -> Self {
        self.parent_window = Some(parent_window);
        self
    }

    pub fn contents(mut self, contents: impl Widget + 'a) -> Self {
        self.contents = contents.boxed();
        self
    }

    pub fn on_close_requested(mut self, on_close_requested: impl Fn() + 'static) -> Self {
        self.callbacks.on_close_requested = Some(Box::new(on_close_requested));
        self
    }

    pub fn on_focus_gained(mut self, on_focus_gained: impl Fn() + 'static) -> Self {
        self.callbacks.on_focus_gained = Some(Box::new(on_focus_gained));
        self
    }

    pub fn on_focus_lost(mut self, on_focus_lost: impl Fn() + 'static) -> Self {
        self.callbacks.on_focus_lost = Some(Box::new(on_focus_lost));
        self
    }

    pub fn on_move(mut self, on_move: impl Fn(u32, u32) + 'static) -> Self {
        self.callbacks.on_move = Some(Box::new(on_move));
        self
    }

    pub fn on_resize(mut self, on_resize: impl Fn(u32, u32) + 'static) -> Self {
        self.callbacks.on_resize = Some(Box::new(on_resize));
        self
    }
}

impl<'a> TypedWidget for Window<'a> {
    type Visual = WindowNode;

    fn layout(
        self,
        context: &mut LayoutCtx,
        previous_visual: Option<Box<WindowNode>>,
        constraints: &BoxConstraints,
        env: Environment,
    ) -> (Box<WindowNode>, Measurements) {
        let is_resizable = self.builder.window.resizable;

        let mut visual = if let Some(visual) = previous_visual {
            // the window has already been created, update its properties
            // TODO update its properties
            visual
        } else {
            log::info!("creating window of size {:?}", self.builder);
            // create the window for the first time
            let window = PlatformWindow::new(
                context.event_loop,
                self.builder,
                &context.app_ctx.platform,
                self.parent_window,
                false,
            )
            .expect("failed to create window");
            // register the window so that we receive raw window events
            context.register_window(window.id());
            Box::new(WindowNode {
                window,
                inputs: Default::default(),
                focus: FocusState::default(),
                last_click: None,
                style_collection: context.app_ctx.style.clone(),
                debug_layout: DebugLayout::None,
                callbacks: self.callbacks,
            })
        };

        let (child_id, child_measurements) = if is_resizable {
            // the window is resizeable by the user:
            // we use the current window size (possibly constrained by the parent constraints)
            // as the available layout space
            let current_size: (f64, f64) = visual
                .window
                .window()
                .inner_size()
                .to_logical::<f64>(1.0)
                .into();
            let current_size: Size = dbg!(current_size.into());
            let constraints = dbg!(constraints).enforce(&BoxConstraints::loose(current_size));
            let size = constraints.constrain(current_size);
            if current_size != size {
                // we need to resize the window to match the parent constraints
                visual
                    .window
                    .window()
                    .set_inner_size(LogicalSize::new(size.width, size.height));
            }
            context.emit_child(self.contents, &constraints, env, Some(&visual.window))
        } else {
            // the window is not resizeable: we use the parent constraints for content layout,
            // and then set the size of the window to fit the contents
            let (child_id, child_measurements) =
                context.emit_child(self.contents, &constraints, env, Some(&visual.window));
            let size = constraints.constrain(child_measurements.size);
            log::warn!(
                "fitting window to contents of size {} (constraints: {:?})",
                size,
                constraints
            );
            visual
                .window
                .window()
                .set_inner_size(LogicalSize::new(size.width, size.height));
            (child_id, child_measurements)
        };

        // request a redraw, because the contents might have changed
        visual.window.window().request_redraw();

        // a window does not take any space in its parent
        // FIXME that can be surprising, maybe add something extra to Measurements to communicate
        // that the space is overlapping (does not participate in layout)?
        (visual, Measurements::new(Size::zero()))
    }
}
