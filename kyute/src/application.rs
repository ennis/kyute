use crate::event::{
    Event, InputEvent, KeyboardEvent, PointerButton, PointerButtons, PointerEvent, WheelDeltaMode,
    WheelEvent,
};
use crate::layout::Size;
use crate::renderer::Theme;
use crate::visual::{
    DummyVisual, EventCtx, FocusState, InputState, LayoutBox, NodeTree, PaintCtx, PointerState,
    RepaintRequest,
};
use crate::widget::{ActionCollector, ActionSink, LayoutCtx};
use crate::{Bounds, BoxConstraints, BoxedWidget, Layout, NodeData, Point, Visual, Widget};
use anyhow::Result;
use kyute_shell::drawing::Color;
use kyute_shell::platform::Platform;
use kyute_shell::window::{PlatformWindow, WindowDrawContext};
use log::trace;
use log::warn;
use std::cell::RefCell;
use std::collections::HashMap;
use std::mem;
use std::rc::{Rc, Weak};
use winit::event::WindowEvent;
use winit::event_loop::{ControlFlow, EventLoopWindowTarget};
use winit::window::{WindowBuilder, WindowId};
use std::time::Instant;
use bitflags::_core::time::Duration;

/// Encapsulates the behavior of an application.
pub trait Application {
    /// The type of actions emitted by the view and handled by the application.
    type Action: 'static;

    /// Handles the specified action emitted by the view.
    fn update(&mut self, actions: &[Self::Action]);

    /// Returns the view (widget tree) to display to the user.
    fn view(&mut self) -> BoxedWidget<Self::Action>;

    // Called whenever an OpenGL viewport needs rendering.
    // Used with `OpenGlViewportWidget`.
    // fn render_gl(&mut self, framebuffer: GLuint, viewport_id: ViewportId, bounds: Bounds);
}

/// Context needed to open a window.
pub struct WindowCtx<'a> {
    pub(crate) platform: &'a Platform,
    event_loop: &'a EventLoopWindowTarget<()>,
    new_windows: Vec<Rc<RefCell<Window>>>,
}

// the event loop should have a ref to the windows, so that it knows where to deliver
// the event based on the window ID.
//
// a child window itself is conceptually "owned" by a parent visual.
//
// actions can be emitted by a window, but not during a traversal of the whole tree
// (only the subtree associated to the window), so action mappers can't operate during the traversal.
// Solution: action mapper has an Rc<ActionSink>
// - one root action sink, which is ActionSink<RootActionType> + one sink per mapper which forwards
//   the transformed action to the parent sink
//      - problem: potentially a lot of mappers, one Rc for each
//
// other option:
// - accumulate all generated actions in a vec alongside the window, then
//   signal the parent window that a child window has generated actions
//   then, traverse widget tree of parent window, and collect (and map) generated actions
//
// other option:
// - always propagate events starting from the root
//    for windows, it means that the event may need to traverse the whole tree before finding the child window
//
// other option:
// - nodes in the visual tree have paths, so that an event that targets a window can be delivered
//   efficiently to the node
//      - similar approach in xxgui
//      - problem: the structure of the visual tree is opaque, so need additional code in Nodes?
//
// There is actually a bigger problem, which is delivering events directly to a target node in the
// hierarchy, without having to do a traversal.
//  - can be useful for keyboard focus, delivering events to a particular window, etc.
//  -
//
// -> This means that visual nodes should be "addressable" (identifiable + an efficient way of reaching them)
// -> which is very hard right now, because
//      - A: the tree is opaque (traversal is the responsibility of each node)
//      - B: nodes don't have a common related type (there's Visual<A>, but 'A' varies between nodes).
//      - C: the layout boxes are computed on-the-fly during traversal
//
// B: The "Action" type parameter should not be in the nodes?
// A: The node hierarchy should be visible: have an explicit tree data structure?
// C: the calculated layout should be stored within the visual node
//
//
// Review of existing approaches:
// - druid: opaque tree, forced traversal to find the target
// - iced: transparent layout tree, no widget identity
// - conrod: graph, nodes accessible by ID
// - ImGui: forced traversal
// - Qt: probably pointers to widgets
// - Servo DOM: tree, garbage collected
// - Stretch (layout lib): nodes are IDs into a Vec-backed tree
// - OrbTk: IDs in an ECS
//

struct LastClick {
    device_id: winit::event::DeviceId,
    button: PointerButton,
    position: Point,
    time: Instant,
    repeat_count: u32,
}

/// A window managed by kyute with a cached visual node.
struct Window {
    window: PlatformWindow,
    tree: NodeTree,
    inputs: InputState,
    // for double-click detection
    last_click: Option<LastClick>
}

impl Window {
    /// Opens a window and registers the window into the event loop.
    pub fn open(ctx: &mut WindowCtx, builder: WindowBuilder) -> Result<Rc<RefCell<Window>>> {
        // create the platform window
        let window = PlatformWindow::new(ctx.event_loop, builder, ctx.platform, true)?;
        let size: (f64, f64) = window.window().inner_size().to_logical::<f64>(1.0).into();

        // create the default visual
        let mut tree = NodeTree::new();

        let window = Window {
            window,
            tree,
            inputs: InputState::default(),
            last_click: None,
        };
        let window = Rc::new(RefCell::new(window));

        ctx.new_windows.push(window.clone());
        Ok(window)
    }

    /// Returns the ID of the window.
    pub fn id(&self) -> WindowId {
        self.window.id()
    }

    /// deliver window event
    fn window_event(&mut self, ctx: &mut WindowCtx, window_event: &WindowEvent) {
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
                let repeat_count =
                    match &mut self.last_click {
                        Some(ref mut last) if last.device_id == *device_id && last.button == button && last.position == position &&
                            (click_time - last.time) < ctx.platform.double_click_time() =>
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
                                        repeat_count: 1
                                    });
                                }
                                winit::event::ElementState::Released => {
                                    *other = None;
                                }
                            };
                            1
                        }
                    };

                dbg!(repeat_count);

                let p = PointerEvent {
                    position,
                    window_position: position,
                    modifiers: self.inputs.mods,
                    button: Some(button),
                    buttons: pointer_state.buttons,
                    pointer_id: *device_id,
                    repeat_count
                };

                let e = match state {
                    winit::event::ElementState::Pressed => Event::PointerDown(p),
                    winit::event::ElementState::Released => Event::PointerUp(p),
                };

                self.tree.event(ctx, &self.window, &self.inputs, &e)
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
                    button: None,
                    buttons: pointer_state.buttons,
                    pointer_id: *device_id,
                    repeat_count: 0
                };

                self.tree
                    .event(ctx, &self.window, &self.inputs, &Event::PointerMove(p))
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
                    self.tree
                        .event(ctx, &self.window, &self.inputs, &Event::Wheel(wheel_event))
                } else {
                    warn!("wheel event received but pointer position is not yet known");
                    return;
                }
            }
            WindowEvent::ReceivedCharacter(char) => self.tree.event(
                ctx,
                &self.window,
                &self.inputs,
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

                self.tree.event(ctx, &self.window, &self.inputs, &event)
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

    /// Updates the current visual tree for this stage.
    fn relayout<A>(
        &mut self,
        window_ctx: &mut WindowCtx,
        action_sink: Rc<dyn ActionSink<A>>,
        theme: &Theme,
        widget: BoxedWidget<A>,
    ) {
        // get window logical size
        let size: (f64, f64) = self
            .window
            .window()
            .inner_size()
            .to_logical::<f64>(1.0)
            .into();
        let size: Size = size.into();
        dbg!(size);
        // perform layout, update the visual node
        self.tree.layout(
            widget,
            size,
            &BoxConstraints::loose(size),
            theme,
            window_ctx,
            action_sink,
        );
        // request a redraw of this window
        self.window.window().request_redraw()
    }

    /// Called when the window needs to be repainted.
    fn paint(&mut self, platform: &Platform, theme: &Theme) {
        {
            let mut wdc = WindowDrawContext::new(&mut self.window);
            wdc.clear(Color::new(0.0, 0.5, 0.8, 1.0));
            self.tree.paint(platform, &mut wdc, &self.inputs, theme);
        }
        self.window.present();
    }
}

/// Runs the specified application.
pub fn run_application<A: Application + 'static>(mut app: A) -> ! {
    // winit event loop
    let event_loop = winit::event_loop::EventLoop::new();
    // platform-specific, window-independent states
    let platform = unsafe { Platform::init() };
    // theme resources
    let theme = Theme::new(&platform);

    // create a window to render the main view.
    let mut win_ctx = WindowCtx {
        platform: &platform,
        event_loop: &event_loop,
        new_windows: Vec::new(),
    };
    let mut main_window = Window::open(&mut win_ctx, WindowBuilder::new().with_title("Default"))
        .expect("failed to create main window");

    // ID -> Weak<Window>
    let mut open_windows = HashMap::new();
    open_windows.insert(main_window.borrow().id(), Rc::downgrade(&main_window));

    let mut collector = Rc::new(ActionCollector::<A::Action>::new());

    // perform the initial layout
    main_window
        .borrow_mut()
        .relayout(&mut win_ctx, collector.clone(), &theme, app.view());

    event_loop.run(move |event, elwt, control_flow| {
        *control_flow = ControlFlow::Wait;

        let mut win_ctx = WindowCtx {
            platform: &platform,
            event_loop: elwt,
            new_windows: Vec::new(),
        };

        match event {
            winit::event::Event::WindowEvent { window_id, event } => {
                // deliver event to the window
                if let Some(window) = open_windows.get(&window_id) {
                    if let Some(window) = window.upgrade() {
                        window.borrow_mut().window_event(&mut win_ctx, &event);
                    }
                }

                if let WindowEvent::Resized(size) = event {
                    // A window has been resized.
                    // Note that, currently, we have no way of relayouting each window separately
                    // (since Application::view() returns the widget tree for all windows).
                    // For now, just relayout everything.

                    // get the widget tree
                    let widget = app.view();

                    // the root of the widget tree is the main window, update it:
                    // this will also send a redraw request for all affected windows.
                    main_window.borrow_mut().relayout(
                        &mut win_ctx,
                        collector.clone(),
                        &theme,
                        widget,
                    );
                }
            }

            winit::event::Event::RedrawRequested(window_id) => {
                // A window needs to be repainted
                if let Some(window) = open_windows.get(&window_id) {
                    if let Some(window) = window.upgrade() {
                        window.borrow_mut().paint(&platform, &theme);
                    }
                }
            }
            _ => (),
        }

        // remove (and close) windows that were dropped
        open_windows.retain(|_, window| window.strong_count() != 0);

        // add the newly-created windows to the list of managed windows
        open_windows.extend(
            mem::take(&mut win_ctx.new_windows)
                .drain(..)
                .map(|v| (v.borrow().id(), Rc::downgrade(&v))),
        );
    })
}
