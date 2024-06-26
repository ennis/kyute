//! winit-based application wrapper.
//!
//! Provides the `run_application` function that opens the main window and translates the incoming
//! events from winit into the events expected by kyute.
use crate::{
    asset::ASSET_LOADER,
    cache,
    cache::Cache,
    core::{dump_widget_tree, WidgetId},
    drawing::{ImageCache, IMAGE_CACHE},
    theme,
    util::fs_watch::{FileSystemWatcher, FILE_SYSTEM_WATCHER},
    AssetLoader, Environment, Event, InternalEvent, Widget,
};
use kyute_shell::{
    winit,
    winit::{
        event_loop::{ControlFlow, EventLoop, EventLoopProxy, EventLoopWindowTarget},
        window::WindowId,
    },
};
use parking_lot::Mutex;
use std::{
    collections::{hash_map::Entry, HashMap},
    fmt, mem,
    sync::Arc,
    task::{Wake, Waker},
};

pub enum ExtEvent {
    /// Triggers a recomposition
    Recompose,
}

impl fmt::Debug for ExtEvent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ExtEvent").finish()
    }
}

/// Global application context. Contains stuff passed to all widget contexts (Event,Layout,Paint...)
pub struct AppCtx {
    /// Open windows, mapped to their corresponding widget.
    pub(crate) windows: HashMap<WindowId, WidgetId>,
    pub(crate) pending_events: Vec<Event<'static>>,
    cache: Cache,
}

impl AppCtx {
    /// Creates a new AppCtx.
    fn new(waker: Waker) -> AppCtx {
        AppCtx {
            windows: HashMap::new(),
            pending_events: vec![],
            cache: Cache::new(waker),
        }
    }

    /// Registers a widget as a native window widget.
    ///
    /// The event loop will call `window_event` whenever an event targeting the window is received.
    pub(crate) fn register_window_widget(&mut self, window_id: WindowId, widget_id: WidgetId) {
        match self.windows.entry(window_id) {
            Entry::Occupied(_) => {
                warn!("window id {:?} already registered", window_id);
            }
            Entry::Vacant(entry) => {
                trace!("registered window id {:?} to widget {:?}", window_id, widget_id);
                entry.insert(widget_id);
            }
        }
    }

    /// Posts a widget event.
    pub fn post_event(&mut self, event: Event<'static>) {
        //tracing::trace!("post_event {:?}", &event);
        self.pending_events.push(event);
    }

    fn send_event(
        &mut self,
        root_widget: &dyn Widget,
        event_loop: &EventLoopWindowTarget<ExtEvent>,
        event: Event<'static>,
        root_env: &Environment,
    ) {
        self.post_event(event);
        self.flush_pending_events(root_widget, event_loop, root_env);
    }

    fn flush_pending_events(
        &mut self,
        root_widget: &dyn Widget,
        event_loop: &EventLoopWindowTarget<ExtEvent>,
        root_env: &Environment,
    ) {
        let _span = trace_span!("flush_pending_events").entered();
        while !self.pending_events.is_empty() {
            let events = mem::take(&mut self.pending_events);
            for mut event in events {
                crate::core::send_root_event(self, event_loop, root_widget, &mut event, root_env);
            }
        }
    }
}

/// Evaluates the function that produces the UI, within the application's positional cache.
/// This is also known as *recomposition*.
fn update_ui<W: Widget + 'static>(
    app_ctx: &mut AppCtx,
    event_loop: &EventLoopWindowTarget<ExtEvent>,
    root_env: &Environment,
    ui: fn() -> W,
) -> Arc<W> {
    let _span = trace_span!("UI update").entered();

    // evaluate the root widget
    let root_widget = {
        let _span = trace_span!("UI recomposition").entered();
        app_ctx
            .cache
            .recompose(root_env, || cache::memoize((), || Arc::new(ui())))
        // ensures that all widgets have received the `Initialize` event.
    };

    app_ctx.cache.dump();

    // send the initialize event
    {
        let _span = trace_span!("UI initialize event").entered();
        crate::core::send_root_event(app_ctx, event_loop, &root_widget, &mut Event::Initialize, root_env);
    }

    //dump_widget_tree(&root_widget);
    root_widget
}

struct EventLoopWaker(Mutex<EventLoopProxy<ExtEvent>>);

impl EventLoopWaker {
    fn new(event_loop: &EventLoop<ExtEvent>) -> EventLoopWaker {
        EventLoopWaker(Mutex::new(event_loop.create_proxy()))
    }
}

impl Wake for EventLoopWaker {
    fn wake(self: Arc<Self>) {
        self.0
            .lock()
            .send_event(ExtEvent::Recompose)
            .expect("failed to wake event loop");
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.0
            .lock()
            .send_event(ExtEvent::Recompose)
            .expect("failed to wake event loop");
    }
}

pub fn run<W: Widget + 'static>(ui: fn() -> W) {
    run_inner(ui, Environment::new())
}

pub fn run_with_env<W: Widget + 'static>(ui: fn() -> W, env_overrides: Environment) {
    run_inner(ui, env_overrides)
}

fn run_inner<W: Widget + 'static>(ui: fn() -> W, env_overrides: Environment) {
    let event_loop = EventLoop::<ExtEvent>::with_user_event();
    let event_loop_waker = Waker::from(Arc::new(EventLoopWaker::new(&event_loop)));
    let mut app_ctx = AppCtx::new(event_loop_waker);

    // setup env
    let mut env = Environment::new();

    // TODO: move those to lazy_statics
    let asset_loader = AssetLoader::new();
    env.set(&ASSET_LOADER, asset_loader.clone());
    let image_cache = ImageCache::new(asset_loader);
    env.set(&IMAGE_CACHE, image_cache);
    let fs_watcher = FileSystemWatcher::new();
    env.set(&FILE_SYSTEM_WATCHER, fs_watcher);
    theme::setup_default_style(&mut env);

    env = env.merged(env_overrides);

    // setup and enter the tokio runtime
    let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");
    let _rt_guard = rt.enter();

    // initial evaluation of the root widget in the main UI cache.
    let mut root_widget = update_ui(&mut app_ctx, &event_loop, &env, ui);

    // run event loop
    event_loop.run(move |event, elwt, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            // --- WINDOW EVENT PROCESSING ---------------------------------------------------------
            winit::event::Event::WindowEvent {
                window_id,
                event: winit_event,
            } => {
                if let Some(&target) = app_ctx.windows.get(&window_id) {
                    if let Some(event) = winit_event.to_static() {
                        app_ctx.send_event(
                            &root_widget,
                            elwt,
                            Event::Internal(InternalEvent::RouteWindowEvent { target, event }),
                            &env,
                        );
                    }
                } else {
                    warn!("unregistered window id: {:?}", window_id);
                }
            }
            // --- RECOMPOSITION -------------------------------------------------------------------
            // happens after window event processing
            winit::event::Event::MainEventsCleared => {
                // Re-evaluate the root widget.
                // If no state variable in the cache has changed (because of an event), then it will simply
                // return the same root widget.
                root_widget = update_ui(&mut app_ctx, elwt, &env, ui);
            }
            // --- EXT EVENTS ----------------------------------------------------------------------
            winit::event::Event::UserEvent(ext_event) => match ext_event {
                ExtEvent::Recompose => {
                    // will recomp in maineventscleared
                    //root_widget = eval_root_widget(&mut app_ctx, elwt, &env, ui);
                }
            },
            // --- REPAINT -------------------------------------------------------------------------
            // happens after recomposition
            winit::event::Event::RedrawRequested(window_id) => {
                if let Some(&target) = app_ctx.windows.get(&window_id) {
                    app_ctx.send_event(
                        &root_widget,
                        elwt,
                        Event::Internal(InternalEvent::RouteRedrawRequest(target)),
                        &env,
                    )
                } else {
                    warn!("unregistered window id: {:?}", window_id);
                }
            }
            _ => (),
        }
    })
}
