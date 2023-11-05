use crate::AppGlobals;
use std::{
    any::Any,
    collections::HashMap,
    fmt, mem,
    sync::{Arc, Mutex},
    task::{Wake, Waker},
    time::Duration,
};
use tracing::{event, trace, warn};
use winit::{
    event_loop::{EventLoop, EventLoopBuilder, EventLoopProxy, EventLoopWindowTarget},
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

pub trait WindowHandler {
    /// Returns true to force the UI to rebuild.
    fn event(&mut self, event: &winit::event::WindowEvent, time: Duration) -> bool;
    fn as_any(&mut self) -> &mut dyn Any;
    fn window_id(&self) -> WindowId;
}

/// Application context passed to the main UI function.
///
/// Manages the event loop and open windows.
pub struct AppCtx<'a> {
    pub(crate) app_state: &'a mut AppState,
    pub(crate) event_loop: &'a EventLoopWindowTarget<ExtEvent>,
}

impl<'a> AppCtx<'a> {
    pub fn quit(&mut self) {
        self.event_loop.exit();
    }

    pub fn register_window(&mut self, window_id: WindowId, handler: Box<dyn WindowHandler>) {
        if self.app_state.windows.insert(window_id, handler).is_some() {
            panic!("window already registered");
        }
    }

    pub(crate) fn window_handler(&mut self, window_id: WindowId) -> &mut dyn WindowHandler {
        &mut **self
            .app_state
            .windows
            .get_mut(&window_id)
            .expect("window not registered")
    }

    pub fn close_window(&mut self, window_id: WindowId) {
        self.app_state.windows.remove(&window_id);
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

/// Holds the windows and the application logic.
pub(crate) struct AppState {
    //pub(crate) windows_by_id: HashMap<u64, WindowId>,
    pub(crate) windows: HashMap<WindowId, Box<dyn WindowHandler>>,
}

impl AppState {
    /*pub fn register_window(&mut self, window_id: WindowId, handler: Box<dyn WindowHandler>) {
        if self.windows.insert(window_id, handler).is_some() {
            panic!("window already registered");
        }
    }

    pub fn window_handler(&mut self, window_id: WindowId) -> &mut dyn WindowHandler {
        &mut **self.windows.get_mut(&window_id).expect("window not registered")
    }

    pub fn close_window(&mut self, window_id: WindowId) {
        self.windows.remove(&window_id);
    }*/

    fn run_ui<F>(&mut self, event_loop: &EventLoopWindowTarget<ExtEvent>, logic: &mut F)
    where
        F: FnMut(&mut AppCtx) + 'static,
    {
        let mut app_ctx = AppCtx {
            app_state: self,
            event_loop,
        };
        (logic)(&mut app_ctx);
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
    app_state: AppState,
    event_loop: EventLoop<ExtEvent>,
    #[cfg(feature = "debug_window")]
    debug_window: crate::debug_window::DebugWindow,
}

impl AppLauncher {
    pub fn new() -> AppLauncher {
        let mut event_loop: EventLoop<ExtEvent> = EventLoopBuilder::with_user_event()
            .build()
            .expect("failed to create the event loop");

        let mut app_state = AppState {
            windows: Default::default(),
        };

        // Create the debug window before the AppGlobals:
        // on windows the internal WGPU instance will create a debug DX12 device,
        // which will remove any existing device (notably the one used by the compositor in AppGlobals).
        //
        // This is OK if we create the compositor device after.
        #[cfg(feature = "debug_window")]
        let mut debug_window = crate::debug_window::DebugWindow::new(&event_loop);

        AppGlobals::new();

        AppLauncher {
            app_state,
            event_loop,
            #[cfg(feature = "debug_window")]
            debug_window,
        }
    }

    // FIXME: annoyingly we need to use a closure because of `event_loop`
    pub fn with_app_ctx<R>(&mut self, f: impl FnOnce(&mut AppCtx) -> R) -> R {
        let mut app_ctx = AppCtx {
            app_state: &mut self.app_state,
            event_loop: &self.event_loop,
        };
        f(&mut app_ctx)
    }

    pub fn run<F>(self, mut ui_fn: F)
    where
        F: FnMut(&mut AppCtx) + 'static,
    {
        let event_loop = self.event_loop;
        let mut app_state = self.app_state;
        let mut debug_window = self.debug_window;

        // run UI at least once to create the initial windows
        //app_state.run_ui(&event_loop, &mut ui_fn);

        let mut force_next_ui = false;
        // run the event loop
        event_loop.set_control_flow(winit::event_loop::ControlFlow::Wait);
        let mut event_loop_start_time = std::time::Instant::now();

        event_loop
            .run(move |event, elwt| {
                //eprintln!("{:?}", event);
                let event_time = std::time::Instant::now().duration_since(event_loop_start_time);

                #[cfg(feature = "debug_window")]
                if debug_window.event(elwt, &event, &mut app_state) {
                    return;
                }

                match event {
                    winit::event::Event::WindowEvent { window_id, event } => {
                        // dispatch to the appropriate window widget
                        if let Some(window) = app_state.windows.get_mut(&window_id) {
                            force_next_ui |= window.event(&event, event_time);
                        } else {
                            warn!("event for unknown window {:?}", window_id);
                        }
                    }
                    winit::event::Event::AboutToWait => {
                        #[cfg(feature = "debug_window")]
                        debug_window.request_redraw();

                        // Application update code.
                        // It can call `elwt.exit()` to exit the event loop, and request window repaints.
                        app_state.run_ui(elwt, &mut ui_fn);
                    }
                    _ => (),
                }
            })
            .expect("event loop run failed")
    }
}
