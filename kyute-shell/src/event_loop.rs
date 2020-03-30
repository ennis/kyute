use log::trace;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoopWindowTarget};
use winit::window::WindowId;
use crate::platform::Platform;

/// The result of delivering an event to a window.
///
/// See [`WindowEventTarget::event`].
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum EventResult {
    /// Do nothing.
    None,
    /// The window should be closed as a result of this event.
    Close,
}

impl Default for EventResult {
    fn default() -> Self {
        EventResult::None
    }
}

/// A wrapper around a window that receives events.
pub trait WindowEventTarget {
    /// Returns the [`WindowId`] of this window.
    ///
    /// [`WindowId`]: winit::WindowId
    fn window_id(&self) -> WindowId;

    /// Called whenever the window receives an event.
    ///
    /// If the return value is `EventResult::Close`, the window is closed immediately.
    fn event(&mut self, ctx: &mut WindowCtx, event: WindowEvent) -> EventResult;

    /// Called when the window should be painted.
    ///
    /// TODO describe how to create a painting context.
    fn paint(&mut self, ctx: &mut WindowCtx);

    /// Returns whether this window is modal: i.e., whether it should capture all user inputs.
    ///
    /// TODO modal stack?
    fn is_modal(&self) -> bool {
        false
    }
}

/// Context passed to functions that create new windows
/// so that they can be registered with the event loop.
///
/// Also the interface for emitting actions, requesting stuff from the run loop, etc.
pub struct WindowCtx<'a> {
    elwt: &'a EventLoopWindowTarget<()>,
    /// Newly-created windows
    new_windows: Vec<Box<dyn WindowEventTarget>>,
    /// platform-specific application state
    platform: &'a Platform,
}

impl<'a> WindowCtx<'a> {
    /// Creates a new context.
    fn new(elwt: &'a EventLoopWindowTarget<()>, platform: &'a Platform) -> WindowCtx<'a> {
        WindowCtx {
            elwt,
            new_windows: Vec::new(),
            platform,
        }
    }

    /// Returns an event loop proxy object that can be used to create other windows via winit while
    /// the event loop is running.
    pub fn event_loop(&self) -> &EventLoopWindowTarget<()> {
        self.elwt
    }

    /// Registers a window to the application event loop.
    ///
    /// The run loop will send events to the window.
    /// See [`WindowEventTarget`].
    pub fn add_window(&mut self, window: impl WindowEventTarget + 'static) {
        self.new_windows.push(Box::new(window));
    }

    /// Returns a reference to the application platform object.
    pub fn platform(&self) -> &'a Platform {
        self.platform
    }
}

/// Represents the main application event loop.
///
/// The event loop also manages the application open windows and dispatches messages to them.
/// It's a wrapper around winit's [`EventLoop`]. It also handle platform-specific
/// application-related services.
///
/// [`EventLoop`]: winit::EventLoop
pub struct MainEventLoop {
    /// Inner winit event loop.
    inner: winit::event_loop::EventLoop<()>,
    /// List of windows created before the event loop is entered.
    early_windows: Vec<Box<dyn WindowEventTarget>>,
    /// Platform-specific global state.
    platform: Platform,
}

impl MainEventLoop {
    /// Creates the application event loop.
    pub fn new() -> MainEventLoop {
        let inner = winit::event_loop::EventLoop::new();
        let platform =
            unsafe { Platform::init().expect("failed to initialize platform state") };
        let ui = MainEventLoop {
            inner,
            early_windows: Vec::new(),
            platform,
        };
        ui
    }

    /// Runs the specified closure with a window context. This can be used to create windows before
    /// the event loop is started.
    pub fn with_window_ctx(&mut self, f: impl FnOnce(&mut WindowCtx)) {
        trace!("creating a new document");
        let mut nw = {
            let mut ctx = WindowCtx::new(&self.inner, &self.platform);
            f(&mut ctx);
            ctx.new_windows
        };
        self.early_windows.append(&mut nw);
    }

    /// Enters the main application event loop.
    pub fn run(self) -> ! {
        let mut windows = self.early_windows;
        // index of the currently modal window
        let platform = self.platform;

        self.inner.run(move |event, elwt, control_flow| {
            *control_flow = ControlFlow::Wait;

            let mut ctx = WindowCtx::new(elwt, &platform);

            match event {
                Event::WindowEvent { window_id, event } => {
                    // find the window with a matching ID and deliver event
                    windows
                        .iter_mut()
                        .find(|w| w.window_id() == window_id)
                        .map(|w| w.event(&mut ctx, event));
                }
                Event::RedrawRequested(window_id) => {
                    // find the window with a matching ID and deliver event
                    windows
                        .iter_mut()
                        .find(|w| w.window_id() == window_id)
                        .map(|w| w.paint(&mut ctx));
                }
                _ => (),
            }

            // add the newly-created windows to the list of managed windows
            windows.append(&mut ctx.new_windows);
        })
    }
}
