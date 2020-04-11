use crate::event::{Event, KeyboardEvent, PointerButton, PointerButtons, PointerEvent};
use crate::layout::Size;
use crate::renderer::Theme;
use crate::visual::{
    reconciliation, DummyVisual, EventCtx, EventResult, FocusState, InputState, LayoutBox,
    PaintCtx, PointerState, RepaintRequest,
};
use crate::widget::{ActionCollector, LayoutCtx};
use crate::{
    Bounds, BoxConstraints, BoxedWidget, Layout, Node, PaintLayout, Point, Visual, Widget,
};
use anyhow::Result;
use kyute_shell::drawing::Color;
use kyute_shell::platform::Platform;
use kyute_shell::window::{DrawContext, PlatformWindow};
use log::trace;
use std::cell::RefCell;
use std::collections::HashMap;
use std::mem;
use std::rc::{Rc, Weak};
use winit::event::{DeviceId, ElementState, KeyboardInput, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget};
use winit::window::{WindowBuilder, WindowId};

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

/// Converts a [`winit::WindowEvent`] into an [`Event`](crate::event::Event)
/// that can be delivered to a [`Node`] hierarchy.
fn convert_window_event(event: &WindowEvent) -> Option<Event> {
    /*match event {
        WindowEvent::KeyboardInput { device_id, input, is_synthetic } => {
            let kbd_ev = KeyboardEvent {
                scan_code: input.scancode,
                key: input.virtual_keycode,
                text: 'r',
                repeat: false,
                modifiers: ModifierState {}
            };

            match input.state {
                ElementState::Pressed => {

                },
                ElementState::Released => {

                }
            }
        }
    }*/
    unimplemented!()
}

/// A window managed by kyute with a cached visual node.
struct Window {
    window: PlatformWindow,
    node: Box<Node<dyn Visual>>,
    inputs: InputState,
    focus_state: FocusState,
}

impl Window {
    /// Opens a window and registers the window into the event loop.
    pub fn open(ctx: &mut WindowCtx, builder: WindowBuilder) -> Result<Rc<RefCell<Window>>> {
        // create the platform window
        let window = PlatformWindow::new(ctx.event_loop, builder, ctx.platform, true)?;
        let size: (f64, f64) = window.window().inner_size().to_logical::<f64>(1.0).into();

        // create the default visual
        let mut node = Node::dummy();

        let window = Window {
            window,
            node,
            inputs: InputState {
                mods: winit::event::ModifiersState::default(),
                pointers: HashMap::new(),
            },
            focus_state: FocusState {},
        };
        let window = Rc::new(RefCell::new(window));

        ctx.new_windows.push(window.clone());
        Ok(window)
    }

    /// Returns the ID of the window.
    pub fn id(&self) -> WindowId {
        self.window.id()
    }

    /// Delivers a pointer event, taking into account the visual that is grabbing the mouse, if there's one.
    fn deliver_pointer_event(&mut self, event: Event) -> EventResult {
        self.node
            .propagate_event(&event, Point::origin(), &self.inputs, &mut self.focus_state)
    }

    /// Delivers a keyboard event, taking into account the visual that has focus, if there's one.
    fn deliver_keyboard_event(&mut self, event: Event) {}

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

                let p = PointerEvent {
                    position: pointer_state.position,
                    window_position: pointer_state.position,
                    modifiers: self.inputs.mods,
                    button: Some(button),
                    buttons: pointer_state.buttons,
                    pointer_id: *device_id,
                };

                let e = match state {
                    winit::event::ElementState::Pressed => {
                        // TODO auto-grab pointer?
                        Event::PointerDown(p)
                    }
                    winit::event::ElementState::Released => {
                        // POINTER UNGRAB: If all pointer buttons are released, force ungrab
                        if p.buttons.is_empty() {
                            trace!("force ungrab");
                            // TODO
                            //self.focus_state.pointer_grab = dummy_weak_node();
                        }
                        Event::PointerUp(p)
                    }
                };

                self.deliver_pointer_event(e)
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
                };
                self.deliver_pointer_event(Event::PointerMove(p))
            }

            _ => {
                return;
            }
        };

        // handle follow-up actions
        match event_result.repaint {
            RepaintRequest::Repaint | RepaintRequest::Relayout => {
                // TODO ask for relayout
                self.window.window().request_redraw();
            }
            _ => {}
        }
    }

    /// Updates the current visual tree for this stage.
    fn relayout<A>(&mut self, ctx: &mut LayoutCtx<A>, theme: &Theme, widget: BoxedWidget<A>) {
        // get window logical size
        let size: (f64, f64) = self
            .window
            .window()
            .inner_size()
            .to_logical::<f64>(1.0)
            .into();
        dbg!(size);
        // perform layout, update the visual node
        widget.layout_single(
            ctx,
            &mut self.node,
            &BoxConstraints::loose(size.into()),
            theme,
        );
        // calculate the absolute bounds of the nodes within the window
        self.node.layout.size = size.into();
        // request a redraw of this window
        self.window.window().request_redraw()
    }

    /// Called when the window needs to be repainted.
    fn paint(&mut self, theme: &Theme) {
        {
            let mut draw_context = DrawContext::new(&mut self.window);

            draw_context.clear(Color::new(0.0, 0.5, 0.8, 1.0));

            let mut ctx = PaintCtx {
                draw_ctx: &mut draw_context,
                size: Default::default(),
            };
            self.node.paint(&mut ctx, theme);
            // drop draw_context
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
    let mut layout_ctx = LayoutCtx {
        win_ctx: &mut win_ctx,
        action_sink: collector.clone(),
    };
    main_window
        .borrow_mut()
        .relayout(&mut layout_ctx, &theme, app.view());

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
                    let mut ctx = LayoutCtx {
                        win_ctx: &mut win_ctx,
                        action_sink: collector.clone(),
                    };

                    main_window.borrow_mut().relayout(&mut ctx, &theme, widget);
                }
            }

            winit::event::Event::RedrawRequested(window_id) => {
                // A window needs to be repainted
                if let Some(window) = open_windows.get(&window_id) {
                    if let Some(window) = window.upgrade() {
                        window.borrow_mut().paint(&theme);
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
