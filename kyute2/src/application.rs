use crate::{
    window::{WindowHandler, WindowPaintOptions},
    AppGlobals, TreeCtx,
};
use std::{
    collections::HashMap,
    fmt,
    rc::{Rc, Weak},
    sync::{Arc, Mutex},
    task::Wake,
};
use tracing::{trace, warn};
use tracy_client::{set_thread_name, span};
use winit::{
    event::WindowEvent,
    event_loop::{ControlFlow, EventLoop, EventLoopBuilder, EventLoopProxy, EventLoopWindowTarget},
    window::WindowId,
};

////////////////////////////////////////////////////////////////////////////////////////////////////

/// Event loop user event.
// TODO make this public?
pub enum ExtEvent {
    /// Triggers an UI update
    UpdateUi,
}

impl fmt::Debug for ExtEvent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ExtEvent").finish()
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

/// Application context passed to the main UI function.
///
/// Manages the event loop and open windows.
/// TODO remove and fuse with TreeCtx
pub struct AppCtx<'a> {
    pub(crate) app_state: &'a mut AppState,
    pub(crate) event_loop: &'a EventLoopWindowTarget<ExtEvent>,
}

impl<'a> AppCtx<'a> {
    pub fn quit(&mut self) {
        self.event_loop.exit();
    }

    pub fn register_window(&mut self, window_id: WindowId, handler: &Rc<dyn WindowHandler>) {
        trace!("registering window {:016X}", u64::from(window_id));
        if self
            .app_state
            .windows
            .insert(window_id, Rc::downgrade(handler))
            .is_some()
        {
            panic!("window already registered");
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

/// Holds the windows and the application logic.
pub(crate) struct AppState {
    /// All open windows by ID.
    ///
    /// TODO this could be replaced by a WeakMap
    pub(crate) windows: HashMap<WindowId, Weak<dyn WindowHandler>>,
}

impl AppState {
    pub(crate) fn window_handler(&self, id: WindowId) -> Option<Rc<dyn WindowHandler>> {
        self.windows.get(&id).and_then(|handler| handler.upgrade())
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

/// Waker for the application event loop.
///
/// TODO document
struct AppWaker(Mutex<EventLoopProxy<ExtEvent>>);

impl AppWaker {
    fn new(event_loop: &EventLoop<ExtEvent>) -> AppWaker {
        AppWaker(Mutex::new(event_loop.create_proxy()))
    }
}

impl Wake for AppWaker {
    fn wake(self: Arc<Self>) {
        self.0
            .lock()
            .unwrap()
            .send_event(ExtEvent::UpdateUi)
            .expect("failed to wake event loop");
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.0
            .lock()
            .unwrap()
            .send_event(ExtEvent::UpdateUi)
            .expect("failed to wake event loop");
    }
}

/// Application launcher.
///
/// # Example
///
/// TODO
pub struct AppLauncher {
    tracy_client: tracy_client::Client,
    app_state: AppState,
    event_loop: EventLoop<ExtEvent>,
    #[cfg(feature = "debug_window")]
    debug_window: crate::debug_window::DebugWindow,
}

impl AppLauncher {
    pub fn new() -> AppLauncher {
        let event_loop: EventLoop<ExtEvent> = EventLoopBuilder::with_user_event()
            .build()
            .expect("failed to create the event loop");

        let app_state = AppState {
            windows: Default::default(),
        };

        // Create the debug window before the AppGlobals:
        // on windows the internal WGPU instance will create a debug DX12 device,
        // which will remove any existing device (notably the one used by the compositor in AppGlobals).
        //
        // This is OK if we create the compositor device after.
        #[cfg(feature = "debug_window")]
        let debug_window = crate::debug_window::DebugWindow::new(&event_loop);

        AppGlobals::new();

        let tracy_client = tracy_client::Client::start();

        AppLauncher {
            tracy_client,
            app_state,
            event_loop,
            #[cfg(feature = "debug_window")]
            debug_window,
        }
    }

    /// Manually runs a UI function.
    ///
    /// Typically this is used to create the initial window before entering the event loop.
    pub fn with_ctx<R>(&mut self, f: impl FnOnce(&mut TreeCtx) -> R) -> R {
        let mut tree_ctx = TreeCtx::new(&mut self.app_state, &self.event_loop);
        f(&mut tree_ctx)
    }

    pub fn run<F>(self, mut logic: F)
    where
        F: FnMut(&mut TreeCtx) + 'static,
    {
        let event_loop = self.event_loop;
        let mut app_state = self.app_state;
        let mut debug_window = self.debug_window;
        let _tracy_client = self.tracy_client;
        set_thread_name!("UI thread");

        // run UI at least once to create the initial windows
        //app_state.run_ui(&event_loop, &mut ui_fn);

        // run the event loop
        event_loop.set_control_flow(ControlFlow::Wait);
        let event_loop_start_time = std::time::Instant::now();

        event_loop
            .run(move |event, elwt| {
                let event_time = std::time::Instant::now().duration_since(event_loop_start_time);

                #[cfg(feature = "debug_window")]
                if debug_window.event(elwt, &event, &mut app_state) {
                    return;
                }

                match event {
                    winit::event::Event::WindowEvent { window_id, event } => {
                        eprintln!("Window {:016X} -> {:?}", u64::from(window_id), event);
                        // dispatch to the appropriate window handler
                        if let Some(win_handler) = app_state.windows.get_mut(&window_id) {
                            if let Some(win_handler) = win_handler.upgrade() {
                                match event {
                                    WindowEvent::RedrawRequested => {
                                        #[allow(unused_assignments)]
                                        let mut options = WindowPaintOptions::default();
                                        #[cfg(feature = "debug_window")]
                                        {
                                            // get special paint options (debug overlays) from the
                                            // debug state
                                            options = debug_window.window_paint_options(window_id);
                                        }
                                        win_handler.paint(event_time, &options);
                                    }
                                    _ => {
                                        win_handler.event(&event, event_time);
                                    }
                                }
                            } else {
                                warn!("received event for expired window {:?}", window_id);
                            }
                        } else {
                            warn!("received event for unknown window {:?}", window_id);
                        }
                    }
                    winit::event::Event::AboutToWait => {
                        // Signal to window handlers that there are no more events.
                        for handler in app_state.windows.values() {
                            if let Some(handler) = handler.upgrade() {
                                handler.events_cleared();
                            }
                        }

                        // Once we've processed all incoming window events and propagated them to
                        // the elements, run the application logic.
                        // It can call `elwt.exit()` to exit the event loop, and request window repaints.
                        let _span = span!("app logic");
                        let mut cx = TreeCtx::new(&mut app_state, elwt);
                        logic(&mut cx);

                        // Debug window maintenance
                        #[cfg(feature = "debug_window")]
                        {
                            // The debug window redraws continuously. Don't bother with trying to
                            // optimize it.
                            debug_window.request_redraw();

                            // If "Force continuous redraw" has been enabled, request a redraw for
                            // all windows.
                            if debug_window.force_continuous_redraw() {
                                for handler in app_state.windows.values() {
                                    if let Some(handler) = handler.upgrade() {
                                        handler.request_redraw();
                                    }
                                }
                            }
                        }
                    }
                    _ => (),
                }
            })
            .expect("event loop run failed")
    }
}
