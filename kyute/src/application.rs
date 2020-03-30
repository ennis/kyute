use kyute_shell::platform::{Platform, PlatformWindow};
use crate::{BoxedWidget, Visual, Cache, Node, BoxConstraints, Renderer, Painter, PaintLayout, Layout, Widget, Point, Bounds};
use winit::event_loop::{EventLoop, ControlFlow, EventLoopWindowTarget};
use winit::window::{WindowBuilder, WindowId};
use crate::event::EventCtx;
use crate::layout::Size;
use winit::event::WindowEvent;
use anyhow::Result;
use crate::widget::dummy::DummyVisual;
use std::rc::{Rc, Weak};
use std::cell::RefCell;
use crate::widget::{LayoutCtx, ActionCollector};
use crate::visual::{Cursor, LayoutBox, PaintCtx};
use std::collections::HashMap;
use std::mem;
use direct2d::render_target::IRenderTarget;

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
    platform: &'a Platform,
    renderer: &'a Renderer,
    event_loop: &'a EventLoopWindowTarget<()>,
    new_windows: Vec<Rc<RefCell<Window>>>,
}

// separate platformwindow + windoweventtarget
// -> windoweventtarget event() can be called for different windows (more than one)
// -> can close the window and keep the event target object alive (is that useful?)
//
// windoweventtarget owns window:
// ->
//

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


/// A window managed by kyute with a cached visual node.
struct Window {
    window: PlatformWindow,
    node: Node<LayoutBox>,
}

impl Window {

    /// Opens a window and registers the window into the event loop.
    pub fn open(
        ctx: &mut WindowCtx,
        builder: WindowBuilder) -> Result<Rc<RefCell<Window>>>
    {
        // create the platform window
        let window = PlatformWindow::new(ctx.event_loop, builder, ctx.platform, true)?;

        // create the default visual
        let mut node = Node::new(Layout::new(Size::new(0.0,0.0)), None, LayoutBox);
        node.propagate_bounds(Point::origin());

        let window = Rc::new(RefCell::new(Window {
            window,
            node
        }));
        ctx.new_windows.push(window.clone());
        Ok(window)
    }

    /// Returns the ID of the window.
    pub fn id(&self) -> WindowId {
        self.window.id()
    }

    /// deliver window event, get actions
    fn window_event(&mut self, ctx: &mut WindowCtx, window_event: &WindowEvent)
    {
        // handle window resize
        if let WindowEvent::Resized(size) = window_event {
            self.window.resize((*size).into());
            return;
        }

        /*let mut ctx = EventCtx {
            bounds: self.node.bounds.expect("layout not done")
        };
        // TODO convert event
        let event = unimplemented!();
        // deliver the event to the node
        self.node.event(&mut ctx, event)*/
    }

    /// Updates the current visual tree for this stage.
    fn relayout<A, W: Widget<A>>(&mut self, ctx: &mut LayoutCtx<A>, widget: W)
    {
        // get window logical size
        let size : (f64,f64) = self.window.window().inner_size().to_logical::<f64>(1.0).into();
        // perform layout, update the visual node
        widget.layout(ctx, &mut self.node.cursor(), &BoxConstraints::loose(size.into()));
        // calculate the absolute bounds of the nodes within the window
        self.node.propagate_bounds(Point::origin());
        // request a redraw of this window
        self.window.window().request_redraw()
    }

    /// Called when the window needs to be repainted.
    fn paint(&mut self, renderer: &Renderer) {
        {
            let mut painter = Painter::new(renderer, &mut self.window);

            // clear the window first
            // TODO: move this into a proper visual
            painter.ctx.render_target_mut().clear(math2d::Color::from_u32(0x0047AB,1.0));

            let mut ctx = PaintCtx {
                bounds: self.node.bounds.expect("layout not done"),
                painter: &mut painter,
            };
            self.node.paint(&mut ctx);
            // drop painter
        }
        self.window.present();
    }
}

/// Runs the specified application.
pub fn run_application<A: Application + 'static>(mut app: A) -> !
{
    // winit event loop
    let event_loop = winit::event_loop::EventLoop::new();
    // platform-specific, window-independent states
    let platform = unsafe { Platform::init().expect("failed to initialize platform") };
    // target-independent renderer resources
    let renderer = Renderer::new(&platform);

    // create a window to render the main view.
    let mut win_ctx = WindowCtx {
        platform: &platform,
        renderer: &renderer,
        event_loop: &event_loop,
        new_windows: Vec::new(),
    };
    let mut main_window = Window::open(&mut win_ctx,
                                    WindowBuilder::new().with_title("Default")).expect("failed to create main window");

    // ID -> Weak<Window>
    let mut open_windows = HashMap::new();
    open_windows.insert(main_window.borrow().id(), Rc::downgrade(&main_window));

    let mut collector = Rc::new(ActionCollector::<A::Action>::new());

    event_loop.run(move |event, elwt, control_flow| {
        *control_flow = ControlFlow::Wait;
        
        let mut win_ctx = WindowCtx {
            platform: &platform,
            renderer: &renderer,
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
                        renderer: &renderer,
                        action_sink: collector.clone()
                    };

                    main_window.borrow_mut().relayout(&mut ctx, widget);
                }
            }

            winit::event::Event::RedrawRequested(window_id) => {
                // A window needs to be repainted
                if let Some(window) = open_windows.get(&window_id) {
                    if let Some(window) = window.upgrade() {
                        window.borrow_mut().paint(win_ctx.renderer);
                    }
                }
            }
            _ => (),
        }

        // remove (and close) windows that were dropped
        open_windows.retain(|_,window| window.strong_count() != 0);

        // add the newly-created windows to the list of managed windows
        open_windows.extend(
            mem::take(&mut win_ctx.new_windows).drain(..).map(|v| (v.borrow().id(), Rc::downgrade(&v)))
        );
    })
}
