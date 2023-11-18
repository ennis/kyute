//! UI host windows
use std::{
    any::Any,
    cell::RefCell,
    mem,
    rc::{Rc, Weak},
    time::{Duration, Instant},
};

use keyboard_types::KeyState;
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
use tracing::trace;
use tracy_client::span;
use winit::{
    event::{DeviceId, ElementState, KeyEvent, MouseButton, WindowEvent},
    keyboard::KeyLocation,
    platform::windows::WindowBuilderExtWindows,
    window::{Window, WindowBuilder},
};

use crate::{
    composition::{ColorType, LayerID},
    context::ElementIdTree,
    debug_util,
    debug_util::{DebugWriter, EventDebugInfo, LayoutDebugInfo, PaintDebugInfo, WindowSnapshot},
    drawing::ToSkia,
    event::{KeyboardEvent, PointerButton, PointerButtons, PointerEvent},
    theme,
    widget::{null::NullElement, WidgetExt},
    window::key::{key_code_from_winit, modifiers_from_winit},
    AppGlobals, BoxConstraints, ChangeFlags, Color, Element, ElementId, Event, EventCtx, EventKind, Geometry,
    HitTestResult, LayoutCtx, PaintCtx, Point, Rect, Size, TreeCtx, Widget,
};

mod key;

////////////////////////////////////////////////////////////////////////////////////////////////////

/// Options for painting a window.
#[derive(Debug, Clone, Default)]
pub struct WindowPaintOptions {
    /// Debug overlays to be painted on top of the UI.
    pub debug_overlay: Option<DebugOverlay>,
    /// Force a relayout before painting the window.
    pub force_relayout: bool,
}

pub trait WindowHandler {
    /// Returns true to force the UI to rebuild.
    fn event(&self, event: &winit::event::WindowEvent, time: Duration) -> bool;
    fn events_cleared(&self) {}
    fn as_any(&self) -> &dyn Any;
    fn request_redraw(&self);
    fn paint(&self, time: Duration, options: &WindowPaintOptions);
    /// Requests a debug snapshot of the window state.
    fn snapshot(&self) -> Option<debug_util::WindowSnapshot> {
        None
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

/// Focus state of a window.
pub struct WindowFocusState {
    // TODO
}

/// List of paint damage regions.
#[derive(Default, Clone, Debug)]
pub struct DamageRegions {
    regions: Vec<Rect>,
}

/// Debug overlay.
#[derive(Debug, Clone)]
pub struct DebugOverlay {
    /// Bounding rectangles to draw on top of the UI.
    pub debug_bounds: Vec<Rect>,
}

impl DebugOverlay {
    /// Paints the components of the debug overlay.
    fn paint(&self, ctx: &mut PaintCtx) {
        let mut surface = ctx.surface.surface();
        let canvas = surface.canvas();

        for bounds in &self.debug_bounds {
            let mut paint = skia_safe::Paint::new(Color::from_hex("#FF000080").to_skia(), None);
            paint.set_stroke_width(1.0);
            canvas.draw_rect(bounds.to_skia(), &paint);
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

/// Stores information about the last click (for double-click handling)
#[derive(Clone, Debug)]
struct LastClick {
    device_id: DeviceId,
    button: PointerButton,
    position: Point,
    time: Instant,
    repeat_count: u32,
}

#[derive(Clone, Debug)]
struct InputState {
    // TODO do tracking in winit and remove this
    cursor_pos: Point,
    /// Modifier state. Tracked here because winit doesn't want to give it to us in events.
    modifiers: keyboard_types::Modifiers,
    /// Pointer button state.
    pointer_buttons: PointerButtons,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

/*
#[derive(Clone)]
struct WindowAndElementId {
    handler: Option<Weak<UiHostWindowHandler>>,
    element: ElementId,
}*/

struct UiHostWindowHandlerInner {
    // this is split from inner to avoid "already borrowed" errors.
    /// The root of the UI tree on this window.
    root_element: Box<dyn Element>,
    /// A tree mapping each element ID to its parent and owner window.
    element_id_tree: ElementIdTree,
    window: Window,
    /// Damage regions to be repainted.
    damage_regions: DamageRegions,
    input_state: InputState,
    last_click: Option<LastClick>,
    close_requested: bool,
    /// Debug info collected during the last call to update_layout.
    layout_debug_info: LayoutDebugInfo,
    /// Debug info collected during the last event propagation.
    event_debug_info: EventDebugInfo,
    /// Debug info collected during the last repaint.
    paint_debug_info: PaintDebugInfo,
    debug_overlay: Option<DebugOverlay>,
    /// Root composition layer for the window.
    layer: LayerID,
    hidden_before_first_draw: bool,
    /// If the window needs to be repainted.
    repaint: bool,
    /// If the contents need to be laid out.
    relayout: bool,
    /// Element currently grabbing the pointer.
    pointer_grab: Option<ElementId>,
    /// Element that has the focus for keyboard events.
    focus: Option<ElementId>,
    /// List of active popups owned by this window.
    popups: Vec<Weak<UiHostWindowHandler>>,
    /// If this is a popup, the owner window & the ID of the popup.
    popup_owner: Option<Weak<UiHostWindowHandler>>,
}

impl UiHostWindowHandlerInner {
    /// Cleans the popup list of all expired popups.
    fn clean_popups(&mut self) {
        self.popups.retain(|popup| popup.upgrade().is_some());
    }

    /// Returns the event propagation path (as a sequence of Element IDs) to the specified target.
    // FIXME: return a smallvec?
    fn get_propagation_path(&self, target: ElementId) -> Vec<ElementId> {
        let mut path = self.element_id_tree.id_path(target);
        path.reverse();
        assert_eq!(
            path[0],
            self.root_element.id(),
            "expected root element ID in propagation path"
        );
        path
    }

    /// Updates internal dirty flags from ChangeFlags reported by the UI tree.
    fn merge_change_flags(&mut self, change_flags: ChangeFlags) {
        if change_flags.intersects(ChangeFlags::STRUCTURE | ChangeFlags::GEOMETRY) {
            self.relayout = true;
        }
        if change_flags.intersects(ChangeFlags::PAINT) {
            self.repaint = true;
        }
    }

    /// Propagates an event in the UI tree.
    fn send_event(&mut self, event: &mut Event, _time: Duration) {
        let mut ctx = EventCtx {
            focus: &mut self.focus,
            pointer_capture: &mut self.pointer_grab,
            window_transform: Default::default(),
            id: None,
            change_flags: ChangeFlags::NONE,
            debug_info: Default::default(),
        };

        let root = event.next_target().expect("route should have at least one element");
        assert_eq!(root, self.root_element.id());

        // if we're a popup window, the

        let change_flags = ctx.event(&mut self.root_element, event);
        // save debug information
        self.event_debug_info = ctx.debug_info;
        // TODO follow-up events
        self.merge_change_flags(change_flags);
    }

    /// Propagates a pointer event in the UI tree.
    ///
    /// It first determines the target of the event (i.e. either the pointer-capturing element or
    /// the deepest element that passes the hit-test), then propagates the event to the target with `send_event`.
    ///
    /// TODO It should also handle focus and hover update events (FocusGained/Lost, PointerOver/Out).
    fn propagate_pointer_event(&mut self, event_kind: EventKind, position: Point, time: Duration) {
        if let Some(pointer_grab) = self.pointer_grab {
            // Pointer events are delivered to the node that is currently grabbing the pointer
            // if there's one.
            let route = self.get_propagation_path(pointer_grab);
            let mut event = Event::new(&route[..], event_kind);
            self.send_event(&mut event, time)
        } else {
            // If nothing is grabbing the pointer, the pointer event is delivered to a widget
            // that passes the hit-test.
            let mut htr = HitTestResult::new();
            let hit = self.root_element.hit_test(&mut htr, position);
            if hit {
                // send to "most specific" (deepest) hit in the stack that is not anonymous
                let target = htr.hits.iter().find(|id| !id.is_anonymous()).cloned();
                let Some(target) = target else {
                    // There were hits, but only on anonymous elements, which don't receive events.
                    return;
                };
                let route = self.get_propagation_path(target);
                //eprintln!("hit-test @ {:?} target={:?} route={:?}", position, target, &route[..]);
                let mut event = Event::new(&route[..], event_kind);
                self.send_event(&mut event, time)
            } else {
                // no grab, no hit, drop the pointer event
                //trace!("hit-test @ {:?} failed", event.pos);
            }
        };

        // TODO: follow-up events (focus update, hover update)
    }

    /// Handles mouse input.
    fn handle_mouse_input(
        &mut self,
        device_id: DeviceId,
        button: MouseButton,
        state: ElementState,
        time: Duration,
    ) -> bool {
        let button = match button {
            MouseButton::Left => PointerButton::LEFT,
            MouseButton::Right => PointerButton::RIGHT,
            MouseButton::Middle => PointerButton::MIDDLE,
            MouseButton::Back => PointerButton::X1,
            MouseButton::Forward => PointerButton::X2,
            MouseButton::Other(_) => return false, // FIXME
        };
        if state.is_pressed() {
            self.input_state.pointer_buttons.set(button);
        } else {
            self.input_state.pointer_buttons.reset(button);
        }
        let click_time = Instant::now();

        // implicit pointer ungrab
        if !state.is_pressed() {
            self.pointer_grab = None;
        }

        // determine the repeat count (double-click, triple-click, etc.) for button down event
        let repeat_count = match &mut self.last_click {
            Some(ref mut last)
                if last.device_id == device_id
                    && last.button == button
                    && last.position == self.input_state.cursor_pos
                    && (click_time - last.time) < AppGlobals::get().double_click_time() =>
            {
                // same device, button, position, and within the platform specified double-click time
                if state.is_pressed() {
                    last.repeat_count += 1;
                    last.repeat_count
                } else {
                    // no repeat for release events (although that could be possible?)
                    1
                }
            }
            other => {
                // no match, reset
                if state.is_pressed() {
                    *other = Some(LastClick {
                        device_id,
                        button,
                        position: self.input_state.cursor_pos,
                        time: click_time,
                        repeat_count: 1,
                    });
                } else {
                    *other = None;
                };
                1
            }
        };
        let pe = PointerEvent {
            target: None,
            position: self.input_state.cursor_pos,
            modifiers: self.input_state.modifiers,
            buttons: self.input_state.pointer_buttons,
            button: Some(button),
            repeat_count: repeat_count as u8,
            transform: Default::default(),
        };

        if state.is_pressed() {
            self.propagate_pointer_event(EventKind::PointerDown(pe), self.input_state.cursor_pos, time);
        } else {
            self.propagate_pointer_event(EventKind::PointerUp(pe), self.input_state.cursor_pos, time);
        }
        false
    }

    /// Handles keyboard input.
    fn handle_keyboard_input(&mut self, event: &KeyEvent, time: Duration) {
        // If there are active popups, keyboard events are delivered to the popups.
        // TODO there should be only one popup active at a time.
        // TODO the terminology is misleading. What we call "popups" are specifically
        // non-activable popup windows (popups that don't deactivate the parent window), like
        // contextual menus. We should probably call them "menus" instead.
        if !self.popups.is_empty() {
            for popup in self.popups.iter() {
                if let Some(popup) = popup.upgrade() {
                    popup.inner.borrow_mut().handle_keyboard_input(event, time);
                    return;
                }
            }
        }

        // keyboard events are delivered to the widget that has the focus.
        // if no widget has focus, the event is dropped.
        if let Some(focus) = self.focus {
            let (key, code) = key_code_from_winit(event);
            let state = match event.state {
                ElementState::Pressed => KeyState::Down,
                ElementState::Released => KeyState::Up,
            };
            let location = match event.location {
                KeyLocation::Standard => keyboard_types::Location::Standard,
                KeyLocation::Left => keyboard_types::Location::Left,
                KeyLocation::Right => keyboard_types::Location::Right,
                KeyLocation::Numpad => keyboard_types::Location::Numpad,
            };

            // determine route to focused widget and send the event to it
            let route = self.get_propagation_path(focus);
            let _change_flags = self.send_event(
                &mut Event::new(
                    &route,
                    EventKind::Keyboard(KeyboardEvent {
                        state,
                        key,
                        location,
                        modifiers: self.input_state.modifiers,
                        repeat: event.repeat,
                        is_composing: false, //TODO
                        code,
                    }),
                ),
                time,
            );
        }
    }

    /// Handles `WindowEvent`s sent to this window.
    fn handle_window_event(&mut self, event: &WindowEvent, time: Duration) -> bool {
        let mut should_update_ui = false;
        match event {
            WindowEvent::Resized(new_size) => {
                if new_size.width != 0 && new_size.height != 0 {
                    let size = Size::new(new_size.width as f64, new_size.height as f64);
                    let app = AppGlobals::get();
                    app.compositor.set_surface_layer_size(self.layer, size);
                    self.relayout = true;
                }
            }
            WindowEvent::KeyboardInput {
                device_id: _,
                event,
                is_synthetic: _,
            } => {
                self.handle_keyboard_input(event, time);
            }
            WindowEvent::ModifiersChanged(modifiers) => {
                self.input_state.modifiers = modifiers_from_winit(*modifiers);
            }
            WindowEvent::CursorMoved { position, device_id: _ } => {
                let logical_position = position.to_logical(self.window.scale_factor());
                self.input_state.cursor_pos.x = logical_position.x;
                self.input_state.cursor_pos.y = logical_position.y;
                let pe = PointerEvent {
                    target: None,
                    position: self.input_state.cursor_pos,
                    modifiers: self.input_state.modifiers,
                    buttons: self.input_state.pointer_buttons,
                    button: None, // Dummy
                    repeat_count: 0,
                    transform: Default::default(),
                };
                self.propagate_pointer_event(EventKind::PointerMove(pe), self.input_state.cursor_pos, time);
            }
            WindowEvent::MouseInput {
                button,
                state,
                device_id,
            } => {
                self.handle_mouse_input(*device_id, *button, *state, time);
            }
            WindowEvent::RedrawRequested => {
                // this is handled in paint
            }
            WindowEvent::CloseRequested => {
                self.close_requested = true;
                should_update_ui = true;
            }
            _ => return false,
        };

        should_update_ui
    }

    fn events_cleared(&mut self) {
        // Once all events in the queue are processed, recalculate the layout if necessary.
        // We do that now so that the application logic can see a layout that is up-to-date with
        // the latest output events.
        //
        // This is important for stuff like virtual lists with widgets created on-demand:
        // the application logic needs to see the latest layout so that it can create the widgets
        // that are visible on the screen.

        let span = span!("events_cleared");
        //span.emit_text(&format!("Window ID: {:016X}", u64::from(self.window.borrow().id())));
        //span.emit_text(&format!("Window title: {:?}", self.window.title()));

        if self.relayout {
            self.relayout = false;
            self.update_layout();
        }
    }

    fn update_layout(&mut self) {
        let span = span!("update_layout");
        //span.emit_text(&format!("Window ID: {:016X}", u64::from(self.window.id())));
        //span.emit_text(&format!("Window title: {:?}", self.window.title()));

        let scale_factor = self.window.scale_factor();
        let window_size = self.window.inner_size().to_logical(scale_factor);
        let mut ctx = LayoutCtx::new(scale_factor);
        let layout_params = BoxConstraints {
            min: Size::ZERO,
            max: Size::new(window_size.width, window_size.height),
        };
        let geometry = ctx.layout(&mut self.root_element, &layout_params);
        trace!(
            "update_layout window_size:{:?}, result geometry:{:?}",
            window_size,
            geometry
        );

        #[cfg(debug_assertions)]
        {
            self.layout_debug_info = ctx.debug_info;
        }
    }

    fn paint(&mut self, _time: Duration, options: &WindowPaintOptions) {
        let span = span!("paint");
        //span.emit_text(&format!("Window ID: {:016X}", u64::from(self.window.id())));
        //span.emit_text(&format!("Window title: {:?}", self.window.title()));

        // Recalculate layout if asked to.
        //
        // For now it's only used for debugging.
        if options.force_relayout {
            self.update_layout();
        }

        let app = AppGlobals::get();

        // Acquire a drawing surface and clear it.
        let surface = app.compositor.acquire_drawing_surface(self.layer);
        // FIXME: only clear and flip invalid regions
        {
            let mut skia_surface = surface.surface();
            skia_surface.canvas().clear(Color::from_hex("#111111").to_skia());
        }

        // Now paint the UI tree.
        {
            let mut paint_ctx = PaintCtx {
                scale_factor: self.window.scale_factor(),
                window_transform: Default::default(),
                id: None,
                surface: &surface,
                debug_info: Default::default(),
            };
            paint_ctx.paint(&mut self.root_element);

            // Paint the debug overlay if there's one.
            if let Some(ref debug_overlay) = options.debug_overlay {
                debug_overlay.paint(&mut paint_ctx);
            }

            // Save debug information after painting.
            self.paint_debug_info = paint_ctx.debug_info;
        }

        // Nothing more to paint, release the surface.
        //
        // This flushes the skia command buffers, and presents the surface to the compositor.
        app.compositor.release_drawing_surface(self.layer, surface);

        // Windows are initially created hidden, and are only shown after the first frame is painted.
        // Now that we've rendered the first frame, we can reveal it.
        if self.hidden_before_first_draw {
            self.hidden_before_first_draw = false;
            self.window.set_visible(true);
        }

        // Wait for the compositor to be ready to render another frame (this is to reduce latency)
        // FIXME: this assumes that there aren't any other windows waiting to be painted!
        app.compositor.wait_for_surface(self.layer);
    }
}

/// Type of UI host window.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WindowType {
    /// Top-level windows.
    ///
    /// They maintain their own focus state, can gain or lose focus, have decorations by default.
    TopLevel,
}

/// Options for UI host windows.
#[derive(Clone)]
pub struct UiHostWindowOptions {
    /// Repaint continuously
    pub continuous_repaint: bool,

    /// Initial title.
    pub title: String,

    /// Is the window resizable?
    pub resizable: bool,

    /// Whether to create the window with decorations.
    pub decorations: bool,

    /// Create a popup window.
    pub popup: bool,

    /// The owner window for popups.
    pub owner: Option<Rc<UiHostWindowHandler>>,

    /// Initial inner size
    pub inner_size: Option<Size>,

    /// Initial position
    pub position: Option<Point>,
}

impl Default for UiHostWindowOptions {
    fn default() -> Self {
        UiHostWindowOptions {
            continuous_repaint: false,
            title: "".to_string(),
            resizable: true,
            decorations: true,
            popup: false,
            owner: None,
            inner_size: None,
            position: None,
        }
    }
}

/// A window handler that hosts a UI tree.
pub struct UiHostWindowHandler {
    inner: RefCell<UiHostWindowHandlerInner>,
    // Newly registered popups. They are added here temporarily to avoid borrowing issues.
    registered_popups: RefCell<Vec<Weak<UiHostWindowHandler>>>,
    raw_window_handle: RawWindowHandle,
}

impl UiHostWindowHandler {
    /// Creates a new window and registers it with the event loop.
    pub fn new(cx: &mut TreeCtx, options: &UiHostWindowOptions) -> Rc<UiHostWindowHandler> {
        //------------------------------------------------
        // Setup window options
        let mut window_builder = WindowBuilder::new()
            .with_visible(false) // Initially invisible
            .with_decorations(options.decorations) // No decorations
            .with_resizable(options.resizable);

        if options.popup {
            // set owner window
            let parent_window_handle = options
                .owner
                .as_ref()
                .expect("a popup window must have an owner window")
                .raw_window_handle();
            match parent_window_handle {
                RawWindowHandle::Win32(parent_hwnd) => {
                    window_builder = window_builder.with_owner_window(parent_hwnd.hwnd.into())
                }
                _ => panic!("Come back later for non-windows support"),
            };
            window_builder = window_builder.with_active(false);
        }
        if let Some(size) = options.inner_size {
            window_builder = window_builder.with_inner_size(winit::dpi::LogicalSize::new(size.width, size.height));
        }
        if let Some(position) = options.position {
            window_builder = window_builder.with_position(winit::dpi::LogicalPosition::new(position.x, position.y));
        }

        //------------------------------------------------
        // build the window
        let window = window_builder
            .build(cx.event_loop)
            .expect("failed to create popup window");
        let window_id = window.id();

        //------------------------------------------------
        // create a compositor layer for the window
        let size = window.inner_size();
        let app = AppGlobals::get();
        let layer = app
            .compositor
            .create_surface_layer(Size::new(size.width as f64, size.height as f64), ColorType::RGBAF16);

        let raw_window_handle = window.raw_window_handle().expect("failed to get raw window handle");
        unsafe {
            // Bind the layer to the window
            // SAFETY: idk? the window handle is valid?
            app.compositor.bind_layer(layer, raw_window_handle);
        }

        // On windows, the initial wait is important:
        // see https://learn.microsoft.com/en-us/windows/uwp/gaming/reduce-latency-with-dxgi-1-3-swap-chains#step-4-wait-before-rendering-each-frame
        app.compositor.wait_for_surface(layer);

        //------------------------------------------------
        // build the window handler
        let handler = Rc::new(UiHostWindowHandler {
            inner: RefCell::new(UiHostWindowHandlerInner {
                root_element: Box::new(NullElement),
                element_id_tree: Default::default(),
                window,
                layer,
                damage_regions: DamageRegions::default(),
                // FIXME: initial value? pray that winit sends a cursor move event immediately after creation
                input_state: InputState {
                    cursor_pos: Default::default(),
                    modifiers: Default::default(),
                    pointer_buttons: Default::default(),
                },
                last_click: None,
                close_requested: false,
                debug_overlay: None,
                layout_debug_info: Default::default(),
                event_debug_info: Default::default(),
                paint_debug_info: Default::default(),
                hidden_before_first_draw: true,
                repaint: false,
                relayout: false,
                pointer_grab: None,
                focus: None,
                popups: vec![],
                popup_owner: options.owner.as_ref().map(|owner| Rc::downgrade(owner)),
            }),
            registered_popups: RefCell::new(Vec::new()),
            raw_window_handle,
        });

        //------------------------------------------------
        // register ourselves to the application and owner window

        // register ourselves to the application so that we may receive WindowEvents
        let handler2: Rc<dyn WindowHandler> = handler.clone(); // dyn coercion
        cx.register_window(window_id, &handler2);

        // If this is a popup, register ourselves with the owner window.
        if options.popup {
            options
                .owner
                .as_ref()
                .expect("a popup window must have an owner window")
                .register_popup(&handler);
        }

        handler
    }

    /// Gets the raw window handle of the window.
    pub fn raw_window_handle(&self) -> RawWindowHandle {
        self.raw_window_handle
    }

    /// Registers a popup window.
    pub fn register_popup(&self, popup: &Rc<Self>) {
        self.registered_popups.borrow_mut().push(Rc::downgrade(popup))
        /*let mut inner = self.inner.borrow_mut();
        trace!(
            "window {:016x}: registering popup window {:016X}",
            u64::from(inner.window.id()),
            u64::from(popup.inner.borrow().window.id())
        );
        inner.popups.push();*/
    }

    /// Updates the contents of the window.
    pub fn update<T>(self: &Rc<Self>, cx: &mut TreeCtx, content: T)
    where
        T: Widget + 'static,
    {
        let mut inner = self.inner.borrow_mut();
        let inner = &mut *inner;

        let _span = span!("update");
        //span.emit_text(&format!("Window ID: {:016X}", u64::from(inner.window.id())));
        //span.emit_text(&format!("Window title: {:?}", inner.window.title()));

        // provide default theme values
        let content = content.provide(theme::DARK_THEME);
        // update the UI tree
        let change_flags = cx.with_parent_window(self.clone(), |cx| {
            cx.update_with_id_tree(content, &mut inner.root_element, &mut inner.element_id_tree)
        });

        // new popups may have popped up
        inner.clean_popups();
        inner.popups.extend(self.registered_popups.take());

        // handle changes reported during UI tree update
        if change_flags.intersects(ChangeFlags::STRUCTURE | ChangeFlags::GEOMETRY) {
            // new elements have been added, or the geometry of existing elements has changed,
            // so we need to recalculate the layout
            inner.update_layout();
        }
        if change_flags.intersects(ChangeFlags::PAINT) {
            // the appearance of some elements has changed, so we need to repaint
            inner.window.request_redraw();
        }
    }

    pub fn set_title(&self, title: &str) {
        let inner = self.inner.borrow_mut();
        inner.window.set_title(title);
    }

    pub fn close_requested(&self) -> bool {
        self.inner.borrow().close_requested
    }
}

impl WindowHandler for UiHostWindowHandler {
    fn event(&self, event: &WindowEvent, time: Duration) -> bool {
        self.inner.borrow_mut().handle_window_event(event, time)
    }

    fn events_cleared(&self) {
        self.inner.borrow_mut().events_cleared();
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn request_redraw(&self) {
        self.inner.borrow().window.request_redraw()
    }

    fn paint(&self, time: Duration, options: &WindowPaintOptions) {
        let mut inner = self.inner.borrow_mut();
        inner.paint(time, options);
    }

    fn snapshot(&self) -> Option<WindowSnapshot> {
        let mut this = self.inner.borrow_mut();
        let root = debug_util::dump_ui_tree(&this.root_element);
        Some(WindowSnapshot {
            window: this.window.id(),
            window_title: this.window.title(),
            layout_info: this.layout_debug_info.clone(),
            paint_info: this.paint_debug_info.clone(),
            event_info: mem::take(&mut this.event_debug_info),
            root,
            focused: this.focus,
            pointer_grab: this.pointer_grab,
            element_id_tree: this.element_id_tree.clone(),
        })
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

/// Handle to an top-level window.
///
/// To close it, just drop the handle.
pub struct AppWindowHandle {
    handler: Rc<UiHostWindowHandler>,
    close_requested: bool,
}

impl AppWindowHandle {
    /// Whether the used clicked the close button.
    pub fn close_requested(&self) -> bool {
        self.close_requested
    }

    pub fn update<T>(&mut self, cx: &mut TreeCtx, content: T)
    where
        T: Widget + Any,
    {
        self.handler.update(cx, content);
        self.close_requested = self.handler.close_requested();
    }

    /// Creates a new top-level window and registers it to the event loop.
    pub fn new<T>(ctx: &mut TreeCtx, title: &str, content: T) -> AppWindowHandle
    where
        T: Widget + Any,
    {
        let handler = UiHostWindowHandler::new(
            ctx,
            &UiHostWindowOptions {
                title: title.to_string(),
                ..Default::default()
            },
        );

        // Initial update & paint
        handler.update(ctx, content);
        handler.paint(Duration::ZERO, &WindowPaintOptions::default());

        AppWindowHandle {
            handler,
            close_requested: false,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

/// The UI element for a child window.
///
/// It's only an empty placeholder to represent the child window in the UI tree of the parent window.
/// As such, it doesn't draw anything, takes no space, and doesn't receive events.
/// The actual logic for the child window is stored in the corresponding `UiHostWindowHandler`.
pub struct PopupWindowElement {
    handler: Option<Rc<UiHostWindowHandler>>,
}

impl Element for PopupWindowElement {
    fn id(&self) -> ElementId {
        // Child window elements don't receive events _that propagate from the parent window_.
        // They do receive their own window events of course, but that's handled by `WindowHandler`.
        ElementId::ANONYMOUS
    }

    fn layout(&mut self, _ctx: &mut LayoutCtx, _params: &BoxConstraints) -> Geometry {
        Geometry::ZERO
    }

    fn event(&mut self, _ctx: &mut EventCtx, _event: &mut Event) -> ChangeFlags {
        ChangeFlags::NONE
    }

    fn natural_width(&mut self, _height: f64) -> f64 {
        0.0
    }

    fn natural_height(&mut self, _width: f64) -> f64 {
        0.0
    }

    fn natural_baseline(&mut self, _params: &BoxConstraints) -> f64 {
        0.0
    }

    fn hit_test(&self, _ctx: &mut HitTestResult, _position: Point) -> bool {
        false
    }

    fn paint(&mut self, _ctx: &mut PaintCtx) {}

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn debug(&self, w: &mut DebugWriter) {
        w.type_name("PopupWindowElement");
    }
}

/// Options for building a popup window.
#[derive(Clone, Debug)]
pub struct PopupOptions {
    /// Whether the window is opened.
    pub opened: bool,
    /// Requested inner size of the window (in logical pixels).
    pub size: Option<Size>,
    /// Position of the window, **relative to the parent**.
    pub position: Option<Point>,
}

impl Default for PopupOptions {
    fn default() -> Self {
        PopupOptions {
            opened: true,
            size: None,
            position: None,
        }
    }
}

/// A widget that opens a popup window that shows the specified content.
///
/// It must be inserted into the UI tree of a parent window in order for the child window to show up,
/// otherwise, nothing will happen. This can be done by adding the `PopupWindow` widget to a
/// container widget, or as an invisible overlay over another widget
/// (e.g. `widget.overlay(PopupWindow::new(...))`.
/// The corresponding element is a dummy element that doesn't take any space in the parent window
/// (see `PopupWindowElement`).
pub struct PopupWindow<F> {
    /// A closure `FnOnce(&mut TreeCtx) -> W where W: Widget` that creates the content of the child window.
    pub content: F,
    pub options: PopupOptions,
}

impl<F, W> PopupWindow<F>
where
    F: FnOnce(&mut TreeCtx) -> W,
    W: Widget + 'static,
{
    fn open_window(self, cx: &mut TreeCtx) -> Rc<UiHostWindowHandler> {
        let _span = span!("open_window");

        // register handler
        let handler = UiHostWindowHandler::new(
            cx,
            &UiHostWindowOptions {
                continuous_repaint: false,
                title: "".to_string(),
                resizable: false,
                decorations: false,
                popup: true,
                owner: Some(cx.parent_window().expect("popup window must have an owner window")),
                inner_size: self.options.size,
                position: self.options.position,
            },
        );

        // initial update
        let content = (self.content)(cx);
        handler.update(cx, content);
        handler.paint(Duration::ZERO, &WindowPaintOptions::default());

        handler
    }
}

impl<F, W> Widget for PopupWindow<F>
where
    F: FnOnce(&mut TreeCtx) -> W,
    W: Widget + 'static,
{
    type Element = PopupWindowElement;

    fn build(self, cx: &mut TreeCtx, _id: ElementId) -> Self::Element {
        if self.options.opened {
            // Open the window.
            let handler = self.open_window(cx);
            PopupWindowElement { handler: Some(handler) }
        } else {
            PopupWindowElement { handler: None }
        }
    }

    fn update(self, cx: &mut TreeCtx, element: &mut Self::Element) -> ChangeFlags {
        if self.options.opened {
            if let Some(ref mut handler) = element.handler {
                let content = (self.content)(cx);
                handler.update(cx, content);
            } else {
                // window not yet opened
                element.handler = Some(self.open_window(cx));
            }
        } else {
            // Close the window.
            // This should be the only reference to the handler, so it should be dropped right now,
            // along with the window object inside it.
            element.handler = None;
        }

        // Changes to a child window incur no changes to the parent window.
        ChangeFlags::NONE
    }
}
