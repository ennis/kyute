//! UI host windows
use std::{
    cell::{Cell, RefCell},
    mem,
    time::{Duration, Instant},
};

use keyboard_types::KeyState;
use raw_window_handle::HasWindowHandle;
use tracing::{info, warn};
use tracy_client::span;
use winit::{
    event::{DeviceId, ElementState, KeyEvent, MouseButton, WindowEvent},
    event_loop::EventLoopWindowTarget,
    keyboard::{KeyLocation, NamedKey},
    window::{Window, WindowBuilder},
};

use crate::{
    application::ExtEvent,
    composition::{ColorType, LayerID},
    core::HitTestEntry,
    drawing::ToSkia,
    event::{PointerButton, PointerButtons, PointerEvent},
    window::key::{key_code_from_winit, modifiers_from_winit},
    AppGlobals, BoxConstraints, ChangeFlags, Color, Ctx, Event, Geometry, HitTestResult, LayoutCtx, PaintCtx, Point,
    Rect, Size, Widget, WidgetCtx, WidgetPod, WidgetPtr, WidgetPtrAny,
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

////////////////////////////////////////////////////////////////////////////////////////////////////

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

#[derive(Clone)]
struct InputState {
    // TODO do tracking in winit and remove this
    cursor_pos: Point,
    /// Modifier state. Tracked here because winit doesn't want to give it to us in events.
    modifiers: keyboard_types::Modifiers,
    /// Pointer button state.
    pointer_buttons: PointerButtons,
    last_click: Option<LastClick>,
    /// The widget currently grabbing the pointer.
    pointer_grab: Vec<HitTestEntry>,
    /// The widget that has the focus for keyboard events.
    focus: Option<WidgetPtrAny>,
    /// Result of the previous hit-test
    prev_hit_test_result: Vec<HitTestEntry>,
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

    // The owner window for popups.
    //pub owner: Option<Rc<UiHostWindowHandler>>,
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
            //owner: None,
            inner_size: None,
            position: None,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// UiHostWindowHandler

//const WINDOW_FOCUS: AmbientKey<Vec<WidgetId>> = AmbientKey::new("kyute.window.focus");

pub struct UiHostWindowState {
    window: Window,
    input_state: InputState,
    close_requested: Cell<bool>,
    dismissed: Cell<bool>,
    layer: LayerID,
    hidden_before_first_draw: Cell<bool>,
    scale_factor: Cell<f64>,
    change_flags: ChangeFlags,
}

impl UiHostWindowState {
    pub fn new(options: &UiHostWindowOptions, event_loop: &EventLoopWindowTarget<ExtEvent>) -> UiHostWindowState {
        //
        // Create window
        //
        let mut window_builder = WindowBuilder::new()
            .with_visible(false) // Initially invisible
            .with_decorations(options.decorations) // No decorations
            .with_resizable(options.resizable);

        if let Some(size) = options.inner_size {
            window_builder = window_builder.with_inner_size(winit::dpi::LogicalSize::new(size.width, size.height));
        }
        if let Some(position) = options.position {
            window_builder = window_builder.with_position(winit::dpi::LogicalPosition::new(position.x, position.y));
        }

        let window = window_builder.build(event_loop).expect("failed to create popup window");

        //
        // Create the compositor layer for the window
        //
        let size = window.inner_size();
        let app = AppGlobals::get();
        let layer = app
            .compositor
            .create_surface_layer(Size::new(size.width as f64, size.height as f64), ColorType::RGBAF16);

        let raw_window_handle = window
            .window_handle()
            .expect("failed to get raw window handle")
            .as_raw();
        unsafe {
            // Bind the layer to the window
            // SAFETY: idk? the window handle is valid?
            app.compositor.bind_layer(layer, raw_window_handle);
        }

        // On windows, the initial wait is important:
        // see https://learn.microsoft.com/en-us/windows/uwp/gaming/reduce-latency-with-dxgi-1-3-swap-chains#step-4-wait-before-rendering-each-frame
        app.compositor.wait_for_surface(layer);

        UiHostWindowState {
            window,
            input_state: InputState {
                cursor_pos: Default::default(),
                modifiers: Default::default(),
                pointer_buttons: Default::default(),
                last_click: None,
                pointer_grab: vec![],
                focus: None,
                prev_hit_test_result: vec![],
            },
            close_requested: Cell::new(false),
            dismissed: Cell::new(false),
            layer,
            hidden_before_first_draw: Cell::new(true),
            scale_factor: Cell::new(1.0),
            change_flags: ChangeFlags::empty(),
        }
    }

    /// Handles `WindowEvent`s sent to this window.
    ///
    /// It updates the last known input state (`input_state`), and resizes the compositor layer
    /// if needed.
    fn handle_window_event(&mut self, cx: &mut Ctx, content: WidgetPtrAny, event: &WindowEvent, time: Duration) {
        //eprintln!("handle_window_event {:?}", event);
        match event {
            WindowEvent::Resized(new_size) => {
                //self.dismiss_popups();
                if new_size.width != 0 && new_size.height != 0 {
                    // resize the compositor layer
                    let size = Size::new(new_size.width as f64, new_size.height as f64);
                    let app = AppGlobals::get();
                    app.compositor.set_surface_layer_size(self.layer, size);
                    // mark the geometry and the visuals dirty
                    self.change_flags |= ChangeFlags::GEOMETRY | ChangeFlags::PAINT;
                }
                self.update_layout(content.clone());
                self.window.request_redraw();
            }
            WindowEvent::Moved(_) => { /*self.dismiss_popups()*/ }
            WindowEvent::KeyboardInput {
                device_id: _,
                event,
                is_synthetic: _,
            } => {
                self.handle_keyboard_input(cx, content.clone(), event, time);
            }
            WindowEvent::ModifiersChanged(modifiers) => {
                self.input_state.modifiers = modifiers_from_winit(*modifiers);
            }
            WindowEvent::CursorMoved { position, device_id: _ } => {
                let pointer_event = {
                    let logical_position = position.to_logical(self.scale_factor.get());
                    self.input_state.cursor_pos.x = logical_position.x;
                    self.input_state.cursor_pos.y = logical_position.y;
                    PointerEvent {
                        position: self.input_state.cursor_pos,
                        modifiers: self.input_state.modifiers,
                        buttons: self.input_state.pointer_buttons,
                        button: None, // Dummy
                        repeat_count: 0,
                        transform: Default::default(),
                        request_capture: false,
                    }
                };
                self.dispatch_pointer_event(
                    cx,
                    content.clone(),
                    Event::PointerMove(pointer_event),
                    self.input_state.cursor_pos,
                    time,
                );
            }
            WindowEvent::MouseInput {
                button,
                state,
                device_id,
            } => {
                //self.dismiss_popups();
                self.handle_mouse_input(cx, content.clone(), *device_id, *button, *state, time);
            }
            WindowEvent::RedrawRequested => {
                self.paint(time, &WindowPaintOptions::default(), content.clone());
            }
            WindowEvent::CloseRequested => {
                self.close_requested.set(true);
                //self.merge_change_flags(ChangeFlags::APP_LOGIC);
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                self.scale_factor.set(*scale_factor);
                //self.merge_change_flags(ChangeFlags::GEOMETRY);
            }
            WindowEvent::Focused(_focused) => {
                /*if !*focused {
                    // dismiss all popups when the parent window loses focus
                    self.dismiss_popups();
                }*/
            }

            _ => {}
        };

        // When we reach that point, all events have been dispatched to widgets, which may have
        // requested updates and/or relayouts. We need to process these now.
        // First process pending updates.
        cx.dispatch_queued_callbacks();

        // Then if something requested a relayout, do it now.
        if cx.needs_layout() {
            info!("layout requested");
            self.update_layout(content.clone());
            // also request a repaint, even though it may be pessimistic
            self.window.request_redraw();
        }
    }

    /// Handles mouse input.
    fn handle_mouse_input(
        &mut self,
        cx: &mut Ctx,
        content: WidgetPtrAny,
        device_id: DeviceId,
        button: MouseButton,
        state: ElementState,
        time: Duration,
    ) {
        let button = match button {
            MouseButton::Left => PointerButton::LEFT,
            MouseButton::Right => PointerButton::RIGHT,
            MouseButton::Middle => PointerButton::MIDDLE,
            MouseButton::Back => PointerButton::X1,
            MouseButton::Forward => PointerButton::X2,
            MouseButton::Other(_) => {
                // FIXME ignore extended buttons for now, but they should really be propagated as well
                return;
            }
        };
        if state.is_pressed() {
            self.input_state.pointer_buttons.set(button);
        } else {
            self.input_state.pointer_buttons.reset(button);
        }
        let click_time = Instant::now();

        /*// implicit pointer ungrab
        if !state.is_pressed() {
            self.input_state.pointer_grab = None;
        }*/

        // determine the repeat count (double-click, triple-click, etc.) for button down event
        let repeat_count = match &mut self.input_state.last_click {
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
            position: self.input_state.cursor_pos,
            modifiers: self.input_state.modifiers,
            buttons: self.input_state.pointer_buttons,
            button: Some(button),
            repeat_count: repeat_count as u8,
            transform: Default::default(),
            request_capture: false,
        };

        let event = if state.is_pressed() {
            Event::PointerDown(pe)
        } else {
            Event::PointerUp(pe)
        };

        self.dispatch_pointer_event(cx, content, event, self.input_state.cursor_pos, time);
    }

    /// Handles keyboard input.
    ///
    /// Returns whether the keyboard input was handled
    fn handle_keyboard_input(&self, _cx: &mut Ctx, _content: WidgetPtrAny, event: &KeyEvent, _time: Duration) {
        /*let mut popups = self.popups.borrow();
        // If there are active popups, keyboard events are delivered to the popups.
        // TODO there should be only one popup active at a time.
        // TODO the terminology is misleading. What we call "popups" are specifically
        // non-activable popup windows (popups that don't deactivate the parent window), like
        // contextual menus. We should probably call them "menus" instead.
        if !popups.is_empty() {
            for popup in popups.iter() {
                if let Some(popup) = popup.upgrade() {
                    popup.handle_keyboard_input(input_state, event, time);
                    return;
                }
            }
        }*/

        // keyboard events are delivered to the widget that has the focus.
        // if no widget has focus, the event is dropped.
        let mut handled = false;
        if let Some(ref focus) = self.input_state.focus {
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

            /*// determine route to focused widget and send the event to it
            let route = self.get_propagation_path(focus);
            let mut event = Event::new(
                &route,
                EventKind::Keyboard(KeyboardEvent {
                    state,
                    key,
                    location,
                    modifiers: input_state.modifiers,
                    repeat: event.repeat,
                    is_composing: false, //TODO
                    code,
                }),
            );
            self.send_event(input_state, &mut event, time);
            handled = event.handled;*/
        }

        if !handled {
            // if nothing handled the event, do default handling at the window level
            match event.logical_key {
                winit::keyboard::Key::Named(NamedKey::Escape) => {
                    /*// if we're a popup, dismiss ourselves
                    if self.popup_owner.is_some() {
                        self.dismissed.set(true);
                        // re-run app logic because it detains the popup
                        self.merge_change_flags(ChangeFlags::APP_LOGIC);
                    }*/
                }
                _ => {}
            }
        }
    }

    /// Propagates a pointer event in the UI tree.
    ///
    /// It first determines the target of the event (i.e. either the pointer-capturing element or
    /// the deepest element that passes the hit-test), then propagates the event to the target with `send_event`.
    ///
    /// TODO It should also handle focus and hover update events (FocusGained/Lost, PointerOver/Out).
    ///
    /// # Return value
    ///
    /// Returns true if the app logic should re-run in response of the event.
    fn dispatch_pointer_event(
        &mut self,
        cx: &mut Ctx,
        content: WidgetPtrAny,
        mut event: Event,
        position: Point,
        time: Duration,
    ) {
        // Pointer events are delivered in priority to the node that is currently grabbing the pointer.
        // If nothing is grabbing the pointer, then the event is delivered to a widget that passes the hit-test.
        let path = if !self.input_state.pointer_grab.is_empty() {
            mem::take(&mut self.input_state.pointer_grab)
        } else {
            let mut hit_test_result = HitTestResult::new();
            content.dyn_hit_test(&mut hit_test_result, position);
            hit_test_result.hits
        };

        self.dispatch_pointer_event_inner(cx, &path, event);

        // Send pointerover/pointerout events
        // Compare the result of the hit-test with the previous hit-test result
        let over: Vec<_> = path
            .iter()
            .filter(|item| {
                !self
                    .input_state
                    .prev_hit_test_result
                    .iter()
                    .find(|x| x.same(item))
                    .is_some()
            })
            .cloned()
            .collect();
        let out: Vec<_> = self
            .input_state
            .prev_hit_test_result
            .iter()
            .filter(|item| !path.iter().find(|x| x.same(item)).is_some())
            .cloned()
            .collect();
        dbg!(&over);
        dbg!(&out);
        self.input_state.prev_hit_test_result = path;
        if !over.is_empty() {
            self.dispatch_pointer_event_inner(
                cx,
                &over,
                Event::PointerOver(PointerEvent {
                    position,
                    modifiers: self.input_state.modifiers,
                    buttons: self.input_state.pointer_buttons,
                    button: None,
                    repeat_count: 0,
                    transform: Default::default(),
                    request_capture: false,
                }),
            );
        }
        if !out.is_empty() {
            self.dispatch_pointer_event_inner(
                cx,
                &out,
                Event::PointerOut(PointerEvent {
                    position,
                    modifiers: self.input_state.modifiers,
                    buttons: self.input_state.pointer_buttons,
                    button: None,
                    repeat_count: 0,
                    transform: Default::default(),
                    request_capture: false,
                }),
            );
        }
    }

    fn dispatch_pointer_event_inner(&mut self, cx: &mut Ctx, path: &[HitTestEntry], mut event: Event) {
        for entry in path.iter() {
            event.set_transform(&entry.transform);
            entry.widget.dyn_event(cx, &mut event);
        }

        if event.capture_requested() {
            // someone in the path requested capture
            self.input_state.pointer_grab = path.into();
        }
    }

    fn update_layout(&self, content: WidgetPtrAny) {
        let _span = span!("update_layout");
        //span.emit_text(&format!("Window ID: {:016X}", u64::from(self.window.id())));
        //span.emit_text(&format!("Window title: {:?}", self.window.title()));

        let scale_factor = self.scale_factor.get();
        let size = self.window.inner_size().to_logical(scale_factor);

        let mut ctx = LayoutCtx::new(scale_factor);
        content.layout(&mut ctx, &BoxConstraints::loose(Size::new(size.width, size.height)));

        /*let geometry = ctx.layout(&mut *self.root_element.borrow_mut(), &layout_params);
        trace!(
            "update_layout window_size:{:?}, result geometry:{:?}",
            window_size,
            geometry
        );*/
        // Layout is clean now.
        //self.clear_change_flags(ChangeFlags::GEOMETRY);
        /*#[cfg(debug_assertions)]
        {
            self.layout_debug_info.replace(ctx.debug_info);
        }*/
    }

    fn set_title(&self, title: &str) {
        self.window.set_title(title);
    }

    /// Returns the position of the window on the desktop.
    ///
    /// The returned value is in logical pixels.
    fn outer_position(&self) -> Point {
        // Querying the window position should succeed on all platforms that we support.
        let scale_factor = self.window.scale_factor();
        let pos = self
            .window
            .outer_position()
            .expect("failed to get window position (unsupported platform?)")
            .to_logical(scale_factor);
        Point::new(pos.x, pos.y)
    }

    fn set_outer_position(&self, position: Point) {
        self.window
            .set_outer_position(winit::dpi::LogicalPosition::new(position.x, position.y));
    }

    fn outer_size(&self) -> Size {
        let scale_factor = self.window.scale_factor();
        let size = self.window.outer_size().to_logical(scale_factor);
        Size::new(size.width, size.height)
    }

    fn close_requested(&self) -> bool {
        self.close_requested.get()
    }

    /// Paints the window.
    ///
    /// This doesn't call `layout` on the content, the caller must to that themselves before calling
    /// this method.
    ///
    /// # Arguments
    ///
    /// * `time` - The current time.
    /// * `options` - Options for painting the window.
    /// * `widget` - The root widget to paint into the window.
    ///
    fn paint(&mut self, _time: Duration, options: &WindowPaintOptions, content: WidgetPtrAny) {
        let _span = span!("paint");
        eprintln!("paint");

        //span.emit_text(&format!("Window ID: {:016X}", u64::from(self.window.id())));
        //span.emit_text(&format!("Window title: {:?}", self.window.title()));

        /*// Recalculate layout if asked to.
        //
        // For now it's only used for debugging.
        if self.change_flags.get().intersects(ChangeFlags::GEOMETRY) || options.force_relayout {
            self.update_layout();
        }*/

        // Acquire a drawing surface and clear it.
        let app = AppGlobals::get();
        let surface = app.compositor.acquire_drawing_surface(self.layer);
        // FIXME: only clear and flip invalid regions
        {
            let mut skia_surface = surface.surface();
            skia_surface.canvas().clear(Color::from_hex("#111111").to_skia());
        }

        // Now paint the UI tree.
        {
            let mut paint_ctx = PaintCtx {
                scale_factor: self.scale_factor.get(),
                window_transform: Default::default(),
                surface: &surface,
                //debug_info: Default::default(),
            };
            content.paint(&mut paint_ctx);

            /*// Paint the debug overlay if there's one.
            if let Some(ref debug_overlay) = options.debug_overlay {
                debug_overlay.paint(&mut paint_ctx);
            }*/

            // Save debug information after painting.
            //self.paint_debug_info.replace(paint_ctx.debug_info);
        }

        // Nothing more to paint, release the surface.
        //
        // This flushes the skia command buffers, and presents the surface to the compositor.
        app.compositor.release_drawing_surface(self.layer, surface);

        // Windows are initially created hidden, and are only shown after the first frame is painted.
        // Now that we've rendered the first frame, we can reveal it.
        if self.hidden_before_first_draw.get() {
            self.hidden_before_first_draw.set(false);
            self.window.set_visible(true);
        }

        //self.clear_change_flags(ChangeFlags::PAINT);

        // Wait for the compositor to be ready to render another frame (this is to reduce latency)
        // FIXME: this assumes that there aren't any other windows waiting to be painted!
        app.compositor.wait_for_surface(self.layer);
    }
}

/// A window handler that hosts a UI tree.
pub struct UiHostWindowHandler {
    /// Gui widgets
    content: WidgetPtrAny,
    options: UiHostWindowOptions,
    window: RefCell<Option<UiHostWindowState>>,
    /// Damage regions to be repainted.
    damage_regions: DamageRegions,
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// PUBLIC API

impl UiHostWindowHandler {
    /// Creates a new window and registers it with the event loop.
    pub fn new(content: impl Widget + 'static, options: UiHostWindowOptions) -> UiHostWindowHandler {
        //------------------------------------------------
        // build the window handler
        UiHostWindowHandler {
            content: WidgetPod::new(content),
            options,
            window: RefCell::new(None),
            damage_regions: DamageRegions::default(),
        }
    }

    fn open_window(&self, cx: &mut WidgetCtx<Self>) {
        eprintln!("open_window");
        let window = UiHostWindowState::new(&self.options, &cx.event_loop);
        // associate this widget to the window so that window events are sent to this widget
        cx.register_window(window.window.id());
        window.window.set_visible(true);
        window.window.request_redraw();
        self.window.replace(Some(window));
    }
}

impl Widget for UiHostWindowHandler {
    fn mount(&mut self, cx: &mut WidgetCtx<Self>) {
        self.open_window(cx);
        self.content.dyn_mount(cx);
    }

    fn update(&mut self, cx: &mut WidgetCtx<Self>) {
        //self.content.dyn_update(cx);
    }

    fn event(&mut self, _cx: &mut WidgetCtx<Self>, _event: &mut Event) {
        // we don't receive or handle events
    }

    fn hit_test(&mut self, _result: &mut HitTestResult, _position: Point) -> bool {
        false
    }

    fn layout(&mut self, _cx: &mut LayoutCtx, _bc: &BoxConstraints) -> Geometry {
        // We return a null geometry here because window widgets typically don't take any space in the parent window.
        // If you're looking for where the contents of the window are laid out, that's done in the `window_event` handler.
        Geometry::ZERO
    }

    fn window_event(&mut self, cx: &mut WidgetCtx<Self>, event: &WindowEvent, time: Duration) {
        if let Some(ref mut window) = &mut *self.window.borrow_mut() {
            window.handle_window_event(cx, self.content.clone(), event, time);
        } else {
            warn!("window_event called before window was created");
        }
    }

    fn paint(&mut self, cx: &mut PaintCtx) {
        // We don't paint anything in the parent window.
        // If you're looking for where the contents of the window are painted, see `UiHostWindowState::window_event`.
    }

    /* fn before_app_logic(&self) {
        let _span = span!("events_cleared");
        //span.emit_text(&format!("Window ID: {:016X}", u64::from(self.window.borrow().id())));
        //span.emit_text(&format!("Window title: {:?}", self.window.title()));

        // Once all events in the queue are processed, recalculate the layout if necessary.
        // We do that now so that the application logic can see a layout that is up-to-date with
        // the latest output events.
        //
        // This is important for stuff like virtual lists with widgets created on-demand:
        // the application logic needs to see the latest layout so that it can create the widgets
        // that are visible on the screen.
        if self.change_flags.get().contains(ChangeFlags::GEOMETRY) {
            self.update_layout();
        }
    }

    fn after_app_logic(&self) {
        self.clear_change_flags(ChangeFlags::APP_LOGIC);

        // Queue a repaint if necessary after the app logic is run.
        if self
            .change_flags
            .get()
            .intersects(ChangeFlags::GEOMETRY | ChangeFlags::PAINT)
        {
            self.window.borrow().request_redraw()
        }
    }*/

    /*fn request_redraw(&self) {
        self.window.borrow().request_redraw()
    }*/

    /*fn snapshot(&self) -> Option<WindowSnapshot> {
        let root = debug_util::dump_ui_tree(&*self.root_element.borrow());
        let window = self.window.borrow();
        let input_state = self.input_state.borrow();
        Some(WindowSnapshot {
            window: window.id(),
            window_title: window.title(),
            layout_info: self.layout_debug_info.borrow().clone(),
            paint_info: self.paint_debug_info.borrow().clone(),
            event_info: self.event_debug_info.take(),
            root,
            focused: input_state.focus,
            pointer_grab: input_state.pointer_grab,
            element_id_tree: self.element_id_tree.borrow().clone(),
        })
    }*/
}

////////////////////////////////////////////////////////////////////////////////////////////////////

/*
/// Handle to a top-level window.
///
/// To close it, just drop the handle.
pub struct AppWindowHandle {
    handler: Rc<UiHostWindowHandler>,
    close_requested: bool,
}

impl AppWindowHandle {
    /// Whether the used clicked the close button.
    ///
    /// FIXME this is never reset
    pub fn close_requested(&self) -> bool {
        self.close_requested
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
        // Ignore the change flags on the initial update
        //handler.update(ctx, content);
        handler.paint(Duration::ZERO, &WindowPaintOptions::default());

        AppWindowHandle {
            handler,
            close_requested: false,
        }
    }
}*/

////////////////////////////////////////////////////////////////////////////////////////////////////

/*
/// TODO update docs
///
/// The UI element for a child window.
///
/// It's only an empty placeholder to represent the child window in the UI tree of the parent window.
/// As such, it doesn't draw anything, takes no space, and doesn't receive events.
/// The actual logic for the child window is stored in the corresponding `UiHostWindowHandler`.
pub struct PopupTargetElement<T> {
    /// Popup window handler.
    popup_handler: Option<Rc<UiHostWindowHandler>>,
    /// Positioning
    position: Option<PopupPosition>,
    /// The content widget to which this popup is attached to.
    content: T,
    content_geometry: Geometry,
}

impl<T: Element> Element for PopupTargetElement<T> {
    fn id(&self) -> ElementId {
        // The popup target doesn't receive events (the popup window itself does, though).
        self.content.id()
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, params: &BoxConstraints) -> Geometry {
        self.content_geometry = ctx.layout(&mut self.content, params);
        self.content_geometry
    }

    fn event(&mut self, ctx: &mut EventCtx, event: &mut Event) -> ChangeFlags {
        self.content.event(ctx, event)
    }

    fn natural_width(&mut self, height: f64) -> f64 {
        self.content.natural_width(height)
    }

    fn natural_height(&mut self, width: f64) -> f64 {
        self.content.natural_height(width)
    }

    fn natural_baseline(&mut self, params: &BoxConstraints) -> f64 {
        self.content.natural_baseline(params)
    }

    fn hit_test(&self, ctx: &mut HitTestResult, position: Point) -> bool {
        self.content.hit_test(ctx, position)
    }

    fn paint(&mut self, ctx: &mut PaintCtx) {
        // TODO only update position if necessary
        if let Some(ref popup_handler) = self.popup_handler {
            let owner_window_pos = popup_handler
                .popup_owner()
                .expect("popup window must have an owner window")
                .outer_position();

            if let Some(ref positioning) = self.position {
                let target_rect = ctx
                    .window_transform
                    .transform_rect_bbox(self.content_geometry.bounding_rect);
                let target_anchor = positioning.parent_anchor.eval(&target_rect);
                let popup_size = popup_handler.outer_size();
                let popup_anchor = positioning.popup_anchor.eval(&popup_size.to_rect());
                let popup_window_pos = owner_window_pos.to_vec2() + target_anchor.to_vec2() - popup_anchor.to_vec2();
                popup_handler.set_outer_position(popup_window_pos.to_point());
            } else {
                todo!("automatic popup positioning")
            }

            // Also repaint the contents of the popup here. Before the first paint the popup is invisible,
            // but we can't paint it earlier because we don't know it's position until the target
            // has been laid out.

            // FIXME: if it's not visible the redraw request goes nowhere!
            popup_handler.paint(Duration::ZERO, &WindowPaintOptions::default());
            eprintln!("PopupTargetElement paint");
        }

        self.content.paint(ctx);
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn debug(&self, w: &mut DebugWriter) {
        w.type_name("PopupTargetElement");
        w.child("", &self.content);
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Anchor {
    Absolute(Point),
    Relative(Point),
}

impl Anchor {
    pub fn eval(&self, rect: &Rect) -> Point {
        match self {
            Anchor::Absolute(point) => *point,
            Anchor::Relative(point) => {
                let Rect { x0, x1, y0, y1 } = *rect;
                Point::new(x0 + point.x * (x1 - x0), y0 + point.y * (y1 - y0))
            }
        }
    }
}

/// Popup anchor.
#[derive(Copy, Clone, Debug)]
pub struct PopupPosition {
    /// Anchor point on the parent.
    pub parent_anchor: Anchor,
    /// Anchor point on the popup.
    pub popup_anchor: Anchor,
}

/// Options for building a popup window.
#[derive(Clone, Debug)]
pub struct PopupOptions {
    /// Whether the window is opened.
    pub opened: bool,
    /// Requested inner size of the window (in logical pixels).
    pub size: Option<Size>,
    /// Position of the window, **relative to the target**.
    pub position: Option<PopupPosition>,
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

/// TODO update docs
///
/// A widget that opens a popup window that shows the specified content.
///
/// It must be inserted into the UI tree of a parent window in order for the child window to show up,
/// otherwise, nothing will happen. This can be done by adding the `PopupWindow` widget to a
/// container widget, or as an invisible overlay over another widget
/// (e.g. `widget.overlay(PopupWindow::new(...))`.
/// The corresponding element is a dummy element that doesn't take any space in the parent window
/// (see `PopupWindowElement`).
pub struct PopupTarget<W, F, FDismiss> {
    /// Inner widget.
    pub content: W,
    /// A closure `FnOnce(&mut TreeCtx) -> W where W: Widget` that creates the content of the child window.
    pub popup_content: F,
    /// A closure `FnOnce(&mut TreeCtx)` called if the popup has been dismissed.
    pub on_dismiss: FDismiss,
    /// Popup options.
    pub options: PopupOptions,
}

impl<W, F, FDismiss, P> PopupTarget<W, F, FDismiss>
where
    F: FnOnce(&mut TreeCtx) -> P,
    FDismiss: FnOnce(&mut TreeCtx),
    P: Widget + 'static,
    W: Widget + 'static,
{
}

fn open_popup_window<T: Widget + 'static>(
    cx: &mut TreeCtx,
    options: &PopupOptions,
    content: T,
) -> Rc<UiHostWindowHandler> {
    let _span = span!("open_popup_window");
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
            inner_size: options.size,
            position: None, // positioned later
        },
    );

    // initial update
    handler.update(cx, content);
    // handler.paint(Duration::ZERO, &WindowPaintOptions::default());
    handler
}

impl<F, FDismiss, W, P> Widget for PopupTarget<W, F, FDismiss>
where
    F: FnOnce(&mut TreeCtx) -> P,
    FDismiss: FnOnce(&mut TreeCtx),
    P: Widget + 'static,
    W: Widget + 'static,
{
    type Element = PopupTargetElement<W::Element>;

    fn build(self, cx: &mut TreeCtx, _id: ElementId) -> Self::Element {
        let content = cx.build(self.content);
        if self.options.opened {
            // Open the window.
            let popup_content = (self.popup_content)(cx);
            let handler = open_popup_window(cx, &self.options, popup_content);
            PopupTargetElement {
                content,
                popup_handler: Some(handler),
                position: self.options.position,
                content_geometry: Default::default(),
            }
        } else {
            PopupTargetElement {
                content,
                popup_handler: None,
                position: self.options.position,
                content_geometry: Default::default(),
            }
        }
    }

    fn update(self, cx: &mut TreeCtx, element: &mut Self::Element) -> ChangeFlags {
        // update popup target
        let mut f = cx.update(self.content, &mut element.content);
        element.position = self.options.position;

        // update popup window contents
        if self.options.opened {
            if let Some(ref mut handler) = element.popup_handler {
                if handler.dismissed() {
                    eprintln!("dismissing popup");
                    (self.on_dismiss)(cx);
                    f |= ChangeFlags::APP_LOGIC;
                } else {
                    let content = (self.popup_content)(cx);
                    handler.update(cx, content);
                    // FIXME APP_LOGIC should probably be propagated
                }
            } else {
                // window not yet opened, open it now
                let popup_content = (self.popup_content)(cx);
                element.popup_handler = Some(open_popup_window(cx, &self.options, popup_content));
                // signal geometry change: this will force a layout
                f |= ChangeFlags::GEOMETRY;
            }
        } else {
            // Close the window.
            // This should be the only reference to the handler, so it should be dropped right now,
            // along with the window object inside it.
            eprintln!("closing popup");
            element.popup_handler = None;
        }

        f
    }
}
*/
