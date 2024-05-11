use std::{
    collections::HashMap,
    fmt,
    sync::{Arc, Mutex},
    task::Wake,
    time::Instant,
};

use crate::{AppGlobals, Widget, WidgetCtx, WidgetPod, WidgetPtr};
use tracing::warn;
use tracy_client::set_thread_name;
use winit::{
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

/// Holds the windows and the application logic.
pub(crate) struct AppState {
    /// Widget paths to open windows.
    pub(crate) windows: HashMap<WindowId, WidgetPtr>,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

/*
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
}*/

/// Holds the UI root widget + the application state.
struct App {
    root: WidgetPtr,
    app_state: AppState,
}

impl App {
    fn update(&mut self, event_loop: &EventLoopWindowTarget<ExtEvent>) {
        let mut tree_ctx = WidgetCtx::new(&mut self.app_state, event_loop, self.root.clone());
        self.root.widget.borrow_mut().update(&mut tree_ctx);
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
    //#[cfg(feature = "debug_window")]
    //debug_window: crate::debug_window::DebugWindow,
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
        //#[cfg(feature = "debug_window")]
        //let debug_window = crate::debug_window::DebugWindow::new(&event_loop);

        AppGlobals::new();

        let tracy_client = tracy_client::Client::start();

        AppLauncher {
            tracy_client,
            app_state,
            event_loop,
            //#[cfg(feature = "debug_window")]
            //debug_window,
        }
    }

    pub fn run(self, root_widget: impl Widget + 'static) {
        self.run_inner(WidgetPod::new(root_widget))
    }

    fn run_inner(self, root: WidgetPtr) {
        let event_loop = self.event_loop;
        //let mut debug_window = self.debug_window;
        let _tracy_client = self.tracy_client;
        set_thread_name!("UI thread");

        // initial UI update
        let mut app = App {
            root,
            app_state: self.app_state,
        };
        app.update(&event_loop);

        // run the event loop
        event_loop.set_control_flow(ControlFlow::Wait);
        let event_loop_start_time = Instant::now();

        event_loop
            .run(move |event, elwt| {
                let event_time = Instant::now().duration_since(event_loop_start_time);

                /*#[cfg(feature = "debug_window")]
                if debug_window.event(elwt, &event, &mut app_state) {
                    return;
                }*/

                match event {
                    winit::event::Event::WindowEvent { window_id, event } => {
                        eprintln!("Window {:08X} -> {:?}", u64::from(window_id), event);

                        // dispatch to the appropriate window handler
                        if let Some(window_widget) = app.app_state.windows.get(&window_id).cloned() {
                            let mut tree_ctx = WidgetCtx::new(&mut app.app_state, elwt, window_widget.clone());
                            window_widget.window_event(&mut tree_ctx, &event, event_time);
                        } else {
                            warn!("received event for unknown window {:?}", window_id);
                        }
                    }
                    winit::event::Event::AboutToWait => {
                        // FIXME: if all we did was paint, we don't need to run the app logic again
                        eprintln!("AboutToWait");
                        eprintln!("------ end event cycle ------");
                    }
                    _ => (),
                }
            })
            .expect("event loop run failed")
    }
}
