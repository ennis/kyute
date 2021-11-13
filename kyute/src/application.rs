//! winit-based application wrapper.
//!
//! Provides the `run_application` function that opens the main window and translates the incoming
//! events from winit into the events expected by a kyute [`NodeTree`](crate::node::NodeTree).

use crate::{
    cache::Key, core2::WidgetId, BoxConstraints, Cache, Event, InternalEvent, LayoutItem, Point,
    WidgetPod,
};
use keyboard_types::KeyState;
use kyute_shell::{
    platform::Platform,
    winit,
    winit::{
        event::{DeviceId, ElementState, VirtualKeyCode},
        event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget},
        window::WindowId,
    },
};
use std::{
    any::Any,
    cell::RefCell,
    collections::{hash_map::Entry, HashMap},
    mem,
    time::Instant,
};
use tracing::{trace_span, warn};

/// Global application context. Contains stuff passed to all widget contexts (Event,Layout,Paint...)
pub struct AppCtx {
    /// Open windows, mapped to their corresponding widget.
    pub(crate) windows: HashMap<WindowId, WidgetId>,
    /// Main UI cache.
    ///
    /// Stores cached copies of widgets and state variables.
    pub(crate) cache: Cache,
    pub(crate) should_relayout: bool,
    pub(crate) should_redraw: bool,
    pub(crate) pending_events: Vec<Event<'static>>,
}

impl AppCtx {
    /// Creates a new AppCtx.
    fn new() -> AppCtx {
        AppCtx {
            windows: HashMap::new(),
            cache: Cache::new(),
            should_relayout: false,
            should_redraw: false,
            pending_events: vec![],
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
                entry.insert(widget_id);
            }
        }
    }

    /// Sets the value of the state variable identified by `key` in the main UI cache.
    pub(crate) fn set_state<T: 'static>(&mut self, key: Key<T>, value: T) {
        self.cache.set_state(key, value);
    }

    pub fn post_event(&mut self, event: Event<'static>) {
        //tracing::trace!("post_event {:?}", &event);
        self.pending_events.push(event);
    }

    fn send_event(
        &mut self,
        root_widget: &mut WidgetPod,
        event_loop: &EventLoopWindowTarget<()>,
        event: Event<'static>
    ) {
        self.post_event(event);
        self.flush_pending_events(root_widget, event_loop);
    }

    fn flush_pending_events(
        &mut self,
        root_widget: &mut WidgetPod,
        event_loop: &EventLoopWindowTarget<()>,
    ) {
        while !self.pending_events.is_empty() {
            let events = mem::take(&mut self.pending_events);
            for mut event in events {
                root_widget.send_root_event(self, event_loop, &mut event)
            }
        }
    }
}

pub fn run(ui: fn() -> WidgetPod) {
    let mut event_loop = EventLoop::new();
    let mut app_ctx = AppCtx::new();

    // initial evaluation of the root widget in the main UI cache.
    let mut root_widget: WidgetPod = app_ctx.cache.run(ui);
    root_widget.ensure_initialized(&mut app_ctx, &event_loop);

    // run event loop
    event_loop.run(move |event, elwt, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            winit::event::Event::WindowEvent {
                window_id,
                event: winit_event,
            } => {
                if let Some(&target) = app_ctx.windows.get(&window_id) {
                    if let Some(event) = winit_event.to_static() {
                        app_ctx.send_event(
                            &mut root_widget,
                            elwt,
                            Event::Internal(InternalEvent::RouteWindowEvent { target, event }),
                        );
                    }
                } else {
                    tracing::warn!("unregistered window id: {:?}", window_id);
                }
            }
            winit::event::Event::RedrawRequested(window_id) => {
                if let Some(&target) = app_ctx.windows.get(&window_id)  {
                    root_widget.send_root_event(
                        &mut app_ctx,
                        elwt,
                        &mut Event::Internal(InternalEvent::RouteRedrawRequest(target)),
                    )
                } else {
                    tracing::warn!("unregistered window id: {:?}", window_id);
                }
            }
            winit::event::Event::MainEventsCleared => {}
            _ => (),
        }

        // Re-evaluate the root widget.
        // If no state variable in the cache has changed (because of an event), then it will simply
        // return the same root widget.
        root_widget = app_ctx.cache.run(ui);
        root_widget.ensure_initialized(&mut app_ctx, elwt);
    })
}
