use crate::{
    context::root_tree_dispatch,
    widget::{WidgetPaths, WidgetPathsRef},
    window::WindowPaintOptions,
    AppGlobals, ChangeFlags, TreeCtx, Widget, WidgetId,
};
use std::{
    collections::HashMap,
    fmt,
    rc::{Rc, Weak},
    sync::{Arc, Mutex},
    task::Wake,
    time::Instant,
};
use tracing::{error, trace, warn};
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

/// Holds the windows and the application logic.
pub(crate) struct AppState {
    /// Widget paths to open windows.
    pub(crate) windows: HashMap<WindowId, Vec<WidgetId>>,
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

/// Holds the UI root widget + the application state.
struct App {
    root: Box<dyn Widget>,
    app_state: AppState,
}

impl App {
    fn update(&mut self, event_loop: &EventLoopWindowTarget<ExtEvent>) {
        let mut tree_ctx = TreeCtx::new(&mut self.app_state, event_loop);
        tree_ctx.update(&mut *self.root);
    }

    /// Dispatches the specified closure
    fn dispatch<'a>(
        &mut self,
        event_loop: &EventLoopWindowTarget<ExtEvent>,
        paths: WidgetPathsRef,
        mut f: impl FnMut(&mut TreeCtx, &mut dyn Widget) + 'a,
    ) {
        root_tree_dispatch(&mut self.app_state, event_loop, &mut *self.root, paths, &mut f);
    }

    fn dispatch_one<R>(
        &mut self,
        event_loop: &EventLoopWindowTarget<ExtEvent>,
        widget_path: &[WidgetId],
        f: impl FnOnce(&mut TreeCtx, &mut dyn Widget) -> R,
    ) -> Option<R> {
        let mut result = None;
        {
            let mut result = &mut None;
            let mut f = Some(f);
            let mut f_mut = move |tree_ctx: &mut TreeCtx, widget: &mut dyn Widget| {
                if let Some(f) = f.take() {
                    *result = Some(f(tree_ctx, widget));
                }
            };
            self.dispatch(event_loop, WidgetPaths::from_path(widget_path).as_slice(), &mut f_mut);
        }
        result
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

    /// Manually runs a UI function.
    ///
    /// Typically this is used to create the initial window before entering the event loop.
    pub fn with_ctx<R>(&mut self, f: impl FnOnce(&mut TreeCtx) -> R) -> R {
        let mut tree_ctx = TreeCtx::new(&mut self.app_state, &self.event_loop);
        f(&mut tree_ctx)
    }

    pub fn run(self, root_widget: impl Widget + 'static) {
        self.run_inner(Box::new(root_widget))
    }

    fn run_inner(self, mut root: Box<dyn Widget>) {
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
                        if let Some(window_widget_path) = app.app_state.windows.get(&window_id).cloned() {
                            app.dispatch_one(elwt, &window_widget_path, move |cx, widget| {
                                widget.window_event(cx, &event, event_time);
                            });

                            /*if let Some(win_handler) = win_handler.upgrade() {
                                match event {
                                    WindowEvent::RedrawRequested => {
                                        #[allow(unused_assignments)]
                                        let mut options = WindowPaintOptions::default();
                                        /*#[cfg(feature = "debug_window")]
                                        {
                                            // get special paint options (debug overlays) from the
                                            // debug state
                                            options = debug_window.window_paint_options(window_id);
                                        }*/
                                        win_handler.paint(event_time, &options);
                                    }
                                    _ => {
                                        app_logic_dirty |= win_handler.event(&event, event_time);
                                    }
                                }
                            } else {
                                warn!("received event for expired window {:?}", window_id);
                            }*/
                        } else {
                            warn!("received event for unknown window {:?}", window_id);
                        }
                    }
                    winit::event::Event::AboutToWait => {
                        // FIXME: if all we did was paint, we don't need to run the app logic again

                        eprintln!("AboutToWait");

                        /*// Call "before_app_logic" on all windows.
                        for handler in app_state.windows.values() {
                            if let Some(handler) = handler.upgrade() {
                                handler.before_app_logic();
                            }
                        }*/

                        /*// Once we've processed all incoming window events and propagated them to
                        // the elements, run the application logic.
                        // It can call `elwt.exit()` to exit the event loop, and request window repaints.
                        let mut change_flags = ChangeFlags::APP_LOGIC;
                        let mut iterations = 0;
                        let max_iterations = 10;
                        // Run it until it doesn't request re-evaluations, or we reach a
                        // maximum number of iterations, or the application requests to exit.
                        while change_flags.contains(ChangeFlags::APP_LOGIC) && !elwt.exiting() {
                            let _span = span!("app logic");
                            let mut cx = TreeCtx::new(elwt);
                            change_flags = logic(&mut cx);
                            iterations += 1;
                            if iterations > max_iterations {
                                error!(
                                    "app logic ran for {} iterations, there might be an infinite update loop",
                                    iterations
                                );
                                break;
                            }
                        }*/

                        /*// Call "after_app_logic" on all windows.
                        for handler in app_state.windows.values() {
                            if let Some(handler) = handler.upgrade() {
                                handler.after_app_logic();
                            }
                        }*/

                        /*// Debug window maintenance
                        #[cfg(feature = "debug_window")]
                        {
                            // The debug window redraws continuously. Don't bother with trying to
                            // optimize it.
                            debug_window.request_redraw();

                            // collect debug snapshots
                            if debug_util::is_collection_enabled() {
                                let mut snapshots = vec![];
                                for handler in app_state.windows.values() {
                                    if let Some(handler) = handler.upgrade() {
                                        if let Some(snapshot) = handler.snapshot() {
                                            snapshots.push(snapshot);
                                        }
                                    }
                                }
                                if !snapshots.is_empty() {
                                    debug_util::record_app_snapshot(DebugSnapshot {
                                        cause: SnapshotCause::AfterPaint,
                                        time: event_time,
                                        window_snapshots: snapshots,
                                    });
                                }
                            }

                            // If "Force continuous redraw" has been enabled, request a redraw for
                            // all windows.
                            if debug_window.force_continuous_redraw() {
                                for handler in app_state.windows.values() {
                                    if let Some(handler) = handler.upgrade() {
                                        handler.request_redraw();
                                    }
                                }
                            }
                        }*/

                        eprintln!("------ end event cycle ------");
                    }
                    _ => (),
                }
            })
            .expect("event loop run failed")
    }
}
