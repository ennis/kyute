use crate::application::WindowCtx;
use crate::event::{
    Event, InputEvent, InputState, KeyboardEvent, PointerButton, PointerButtonEvent, PointerEvent,
    PointerState, WheelDeltaMode, WheelEvent,
};
use crate::node::{DebugLayout, FocusState, NodeTree, PaintOptions, RepaintRequest};
use crate::style::StyleCollection;
use crate::widget::DummyWidget;
use crate::{BoxConstraints, BoxedWidget, Environment, EventCtx, LayoutCtx, Measurements, PaintCtx, Point, Rect, Size, TypedWidget, Visual, Widget, WidgetExt, Offset};
use bitflags::_core::any::Any;
use generational_indextree::NodeId;
use kyute_shell::drawing::Color;
use kyute_shell::window::{PlatformWindow, WindowDrawContext};
use std::rc::Rc;
use std::time::Instant;
use winit::event::{VirtualKeyCode, WindowEvent};
use winit::window::{WindowBuilder, WindowId};

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
}

pub trait WindowHandler {}

impl Visual for WindowNode {
    fn paint(&mut self, _ctx: &mut PaintCtx, _env: &Environment) {
        // should never be called
    }

    fn hit_test(&mut self, _point: Point, _bounds: Rect) -> bool {
        unimplemented!()
    }

    fn event(&mut self, _event_ctx: &mut EventCtx, _event: &Event) {
        // should never be called
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn window_id(&self) -> Option<WindowId> {
        Some(self.window.id())
    }

    fn window_event(
        &mut self,
        ctx: &mut WindowCtx,
        window_event: &WindowEvent,
        tree: &mut NodeTree,
        root: NodeId,
    ) {
        let event_result = match window_event {
            WindowEvent::Resized(size) => {
                self.window.resize((*size).into());
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

                tree.event(ctx, &self.window, root, &self.inputs, &mut self.focus, &e)
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

                let result =
                    tree.event(ctx, &self.window, root, &self.inputs, &mut self.focus, &Event::PointerMove(p));

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
                    tree
                        .event(ctx, &self.window, root, &self.inputs, &mut self.focus, &Event::Wheel(wheel_event))
                } else {
                    log::warn!("wheel event received but pointer position is not yet known");
                    return;
                }
            }
            WindowEvent::ReceivedCharacter(char) => tree.event(
                ctx, &self.window, root, &self.inputs, &mut self.focus,
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
                    tree.event(ctx, &self.window, root, &self.inputs, &mut self.focus, &event)
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

    fn window_paint(&mut self,
                    ctx: &mut WindowCtx,
                    tree: &mut NodeTree,
                    anchor: NodeId) {
        {
            let id = self.window.id();
            let mut wdc = WindowDrawContext::new(&mut self.window);
            wdc.clear(Color::new(0.326, 0.326, 0.326, 1.0));
            let options = PaintOptions {
                debug_draw_bounds: self.debug_layout,
            };
            tree.paint(
                ctx.platform,
                &mut wdc,
                &self.style_collection,
                id,
                anchor,
                &self.inputs,
                &self.focus,
                &options,
            );
        }
        self.window.present();
    }
}

pub struct Window<'a> {
    builder: WindowBuilder,
    contents: BoxedWidget<'a>,
}

impl<'a> Window<'a> {
    pub fn new(builder: WindowBuilder) -> Window<'a> {
        Window {
            builder,
            contents: DummyWidget.boxed(),
        }
    }

    pub fn contents(mut self, contents: impl Widget + 'a) -> Self {
        self.contents = contents.boxed();
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
        let w = if let Some(w) = previous_visual {
            // the window has already been created, update its properties
            // TODO update its properties
            w
        } else {
            // create the window for the first time
            let window = PlatformWindow::new(
                context.win_ctx.event_loop,
                self.builder,
                context.win_ctx.platform,
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
                style_collection: context.win_ctx.style.clone(),
                debug_layout: DebugLayout::None,
            })
        };

        // get the window logical size
        let size: (f64, f64) = w
            .window
            .window()
            .inner_size()
            .to_logical::<f64>(1.0)
            .into();
        let size: Size = size.into();

        // perform layout, update the visual node
        // ignore the parent constraints, since we're in another window
        context.emit_child(self.contents, &BoxConstraints::loose(size), env);
        // request a redraw, because the contents might have changed
        w.window.window().request_redraw();

        // a window does not take any space in its parent
        (w, Measurements::new(Size::zero()))
    }
}
