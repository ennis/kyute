use crate::AppGlobals;
use kyute_compose::Cache;
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

    pub fn window_handler(&mut self, window_id: WindowId) -> &mut dyn WindowHandler {
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
    pub(crate) windows: HashMap<WindowId, Box<dyn WindowHandler>>,
}

fn run_ui(
    app_state: &mut AppState,
    cache: &mut Cache,
    event_loop: &EventLoopWindowTarget<ExtEvent>,
    logic: &mut Box<dyn FnMut(&mut AppCtx)>,
    force: bool,
) {
    if cache.is_dirty() || force {
        trace!("AppHandler: running app logic");
        // build the appctx
        let mut app_ctx = AppCtx { app_state, event_loop };
        // invoke UI closure
        cache.run(|| (logic)(&mut app_ctx));
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
    ui_fn: Box<dyn FnMut(&mut AppCtx)>,
}

impl AppLauncher {
    pub fn new(ui_fn: impl FnMut(&mut AppCtx) + 'static) -> AppLauncher {
        AppLauncher { ui_fn: Box::new(ui_fn) }
    }

    pub fn run(mut self) {
        let mut event_loop: EventLoop<ExtEvent> = EventLoopBuilder::with_user_event()
            .build()
            .expect("failed to create the event loop");

        #[cfg(feature = "debug_window")]
        let mut debug_window = crate::debug_window::DebugWindow::new(&event_loop);
        AppGlobals::new();

        let waker = Waker::from(Arc::new(AppWaker(Mutex::new(event_loop.create_proxy()))));
        let mut cache = Cache::new(waker);
        let mut app_state = AppState {
            windows: HashMap::new(),
        };
        let mut ui_fn = self.ui_fn;

        // run UI at least once to create the initial windows
        run_ui(&mut app_state, &mut cache, &event_loop, &mut ui_fn, true);

        let mut force_next_ui = false;
        // run the event loop
        event_loop.set_control_flow(winit::event_loop::ControlFlow::Wait);
        let mut event_loop_start_time = std::time::Instant::now();

        event_loop
            .run(move |event, elwt| {
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
                        run_ui(
                            &mut app_state,
                            &mut cache,
                            elwt,
                            &mut ui_fn,
                            mem::take(&mut force_next_ui),
                        );
                    }
                    _ => (),
                }
            })
            .expect("event loop run failed")
    }
}
