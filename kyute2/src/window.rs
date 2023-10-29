//! UI host windows
mod key;

use crate::{
    application::{AppCtx, WindowHandler},
    composable, composition,
    composition::ColorType,
    context::WidgetTree,
    debug_util::{debug_record_ui_snapshot, DebugEventData},
    drawing::ToSkia,
    event::{KeyboardEvent, PointerButton, PointerButtons, PointerEvent},
    widget::null::NullElement,
    AnyWidget, AppGlobals, ChangeFlags, Color, Element, Environment, Event, EventCtx, EventKind, HitTestResult,
    LayoutCtx, LayoutParams, PaintCtx, Point, Rect, RouteEventCtx, Size, TreeCtx, Widget, WidgetId,
};
use bitflags::Flags;
use keyboard_types::KeyState;
use kyute2::window::key::{key_code_from_winit, modifiers_from_winit};
use kyute_compose::{cache_cx, State};
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
use std::{
    any::Any,
    cell::RefCell,
    mem,
    rc::Rc,
    time::{Duration, Instant},
};
use tracing::{trace, trace_span, warn};
use winit::{
    event::{DeviceId, ElementState, MouseButton, WindowEvent},
    keyboard::KeyLocation,
    window::{WindowBuilder, WindowId},
};

/// Focus state of a window.
pub struct WindowFocusState {
    // TODO
}

/// List of paint damage regions.
#[derive(Default, Clone)]
pub struct DamageRegions {
    regions: Vec<Rect>,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

/*type WindowContentRef = Rc<RefCell<Option<Box<dyn AnyWidget>>>>;

/// A handle to update the contents (widget tree) of a window.
#[derive(Clone)]
pub struct WindowContentHandle {
    idle_handle: IdleHandle,
    content: WindowContentRef,
}

impl WindowContentHandle {
    pub(crate) fn new(idle_handle: IdleHandle, content: WindowContentRef) -> WindowContentHandle {
        WindowContentHandle { idle_handle, content }
    }

    /// Updates the widget.
    pub fn set(&self, widget: Box<dyn AnyWidget>) {
        self.content.replace(Some(widget));
        self.idle_handle.schedule_idle(CONTENT_UPDATE);
    }

    pub fn take(&self) -> Option<Box<dyn AnyWidget>> {
        self.content.replace(None)
    }
}*/

pub(crate) struct DebugOverlayData {
    pub(crate) debug_bounds: Vec<Rect>,
}

impl DebugOverlayData {
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
// Pointer events

enum PointerEventKind {
    PointerUp,
    PointerDown,
    PointerMove,
}

impl UiHostWindowHandler {
    // FIXME: smallvec?
    fn get_propagation_path(&mut self, mut target: WidgetId) -> Vec<WidgetId> {
        let mut result = vec![target];
        while let Some(parent) = self.widget_tree.get(&target) {
            result.push(*parent);
            target = *parent;
        }
        result.reverse();
        result
    }

    fn send_event(&mut self, event: &mut Event, time: Duration) -> ChangeFlags {
        let mut ctx = EventCtx {
            window: &self.window,
            focus: &mut self.focus,
            window_transform: Default::default(),
            pointer_capture: &mut self.pointer_grab,
            id: None,
            change_flags: ChangeFlags::NONE,
        };

        let root = event.next_target().expect("route should have at least one element");
        assert_eq!(root, self.root.id());

        let change_flags = self.root.event(&mut ctx, event);

        #[cfg(debug_assertions)]
        {
            // record a snapshot of the UI after delivering the event
            debug_record_ui_snapshot(
                time,
                self.window.id(),
                &self.root,
                vec![DebugEventData {
                    route: event.route.to_owned(),
                    kind: event.kind.clone(),
                    change_flags,
                }],
            );
        }

        // TODO follow-up events
        change_flags
    }

    fn propagate_pointer_event(&mut self, event_kind: EventKind, position: Point, time: Duration) -> ChangeFlags {
        //let pe = PointerEvent::from_glazier(event);
        /*let event_kind = match kind {
            PointerEventKind::PointerUp => EventKind::PointerUp(pe),
            PointerEventKind::PointerDown => EventKind::PointerDown(pe),
            PointerEventKind::PointerMove => EventKind::PointerMove(pe),
        };*/

        let change_flags = if let Some(pointer_grab) = self.pointer_grab {
            // Pointer events are delivered to the node that is currently grabbing the pointer
            // if there's one.
            let route = self.get_propagation_path(pointer_grab);
            let mut event = Event::new(&route[..], event_kind);
            self.send_event(&mut event, time)
        } else {
            // If nothing is grabbing the pointer, the pointer event is delivered to a widget
            // that passes the hit-test.
            let mut htr = HitTestResult::new();
            let hit = self.root.hit_test(&mut htr, position);
            if hit {
                // send to "most specific" (deepest) hit in the stack
                let target = *htr
                    .hits
                    .first()
                    .expect("successful hit test result should contain at least one element");
                let route = self.get_propagation_path(target);
                trace!("hit-test @ {:?} target={:?} route={:?}", position, target, &route[..]);
                let mut event = Event::new(&route[..], event_kind);
                self.send_event(&mut event, time)
            } else {
                // no grab, no hit, drop the pointer event
                //trace!("hit-test @ {:?} failed", event.pos);
                ChangeFlags::NONE
            }
        };

        change_flags

        // TODO: follow-up events (focus update, hover update)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

/// Stores information about the last click (for double-click handling)
struct LastClick {
    device_id: DeviceId,
    button: PointerButton,
    position: Point,
    time: Instant,
    repeat_count: u32,
}

struct InputState {
    // TODO do tracking in winit and remove this
    cursor_pos: Point,
    /// Modifier state. Tracked here because winit doesn't want to give it to us in events.
    modifiers: keyboard_types::Modifiers,
    /// Pointer button state.
    pointer_buttons: PointerButtons,
}

/// UI host window handler
pub(crate) struct UiHostWindowHandler {
    /// Winit window
    pub(crate) window: winit::window::Window,
    /// Root composition layer for the window.
    layer: composition::LayerID,
    /// Damage regions to be repainted.
    damage_regions: DamageRegions,
    widget_tree: WidgetTree,
    /// Root of the UI element tree.
    pub(crate) root: Box<dyn Element>,
    //idle_handle: Option<IdleHandle>,
    /*/// Window contents (widget tree).
    ///
    /// This is updated from the application logic via `WindowContentHandle`s.
    /// When a content update is signalled to the window, the widget is consumed and applied to
    /// the retained element tree (`root` + `elem_tree`).
    content: WindowContentRef,*/
    // Run loop handle
    //app_handle: AppHandle,
    pub(crate) focus: Option<WidgetId>,
    pub(crate) pointer_grab: Option<WidgetId>,
    input_state: InputState,
    last_click: Option<LastClick>,
    close_requested: bool,
    debug_overlay: Option<DebugOverlayData>,
}

impl UiHostWindowHandler {
    fn new(window: winit::window::Window, layer: composition::LayerID) -> UiHostWindowHandler {
        // TODO remove layer on drop?
        UiHostWindowHandler {
            window,
            layer,
            damage_regions: DamageRegions::default(),
            widget_tree: Default::default(),
            root: Box::new(NullElement),
            focus: None,
            pointer_grab: None,
            // FIXME: initial value? pray that winit sends a cursor move event immediately after creation
            input_state: InputState {
                cursor_pos: Default::default(),
                modifiers: Default::default(),
                pointer_buttons: Default::default(),
            },
            last_click: None,
            close_requested: false,
            debug_overlay: None,
        }
    }

    fn update_content(&mut self, content: Box<dyn AnyWidget>) {
        let mut tree_ctx = TreeCtx::new(&mut self.widget_tree);

        // FIXME: this should come alongside the content
        let env = Environment::new();
        let mut change_flags = content.update(&mut tree_ctx, &mut self.root, &env);
        trace!("update_content: {:?}", change_flags);
        if change_flags.intersects(ChangeFlags::STRUCTURE | ChangeFlags::GEOMETRY) {
            self.update_layout();
        }
        if change_flags.intersects(ChangeFlags::PAINT) {
            self.window.request_redraw();
        }
    }

    fn update_layout(&mut self) {
        let _span = trace_span!("update_layout").entered();
        let scale_factor = self.window.scale_factor();
        let window_size = self.window.inner_size().to_logical(scale_factor);
        let mut ctx = LayoutCtx::new(&self.window, self.focus);
        let layout_params = LayoutParams {
            scale_factor, // assume x == y
            min: Size::ZERO,
            max: Size::new(window_size.width, window_size.height),
        };
        let geometry = self.root.layout(&mut ctx, &layout_params);
        trace!(
            "update_layout window_size:{:?}, result geometry:{:?}",
            window_size,
            geometry
        );
        self.window.request_redraw();
    }

    /*fn schedule_render(&mut self) {
        if !self.synced_with_presentation {
            let _span = trace_span!("PRESENT_SYNC").entered();
            let layer = self.main_layer.unwrap();
            let app = AppGlobals::get();
            app.compositor.wait_for_surface(layer);
            self.synced_with_presentation = true;
        }
        self.handle.invalidate();
    }*/
}

impl UiHostWindowHandler {
    fn paint(&mut self) {
        trace!("UiHostWindowHandler::paint");
        let app = AppGlobals::get();

        // acquire a drawing surface, then repaint, and swap invalid regions
        let surface = app.compositor.acquire_drawing_surface(self.layer);

        // FIXME: only clear and flip invalid regions
        {
            let mut skia_surface = surface.surface();
            skia_surface.canvas().clear(Color::from_hex("#111111").to_skia());
        }

        let mut paint_ctx = PaintCtx {
            window: &self.window,
            focus: self.focus,
            window_transform: Default::default(),
            id: None,
            surface,
        };

        self.root.paint(&mut paint_ctx);

        // paint debug overlay
        self.debug_overlay.as_ref().map(|overlay| overlay.paint(&mut paint_ctx));

        app.compositor.release_drawing_surface(self.layer, paint_ctx.surface);
        // wait for the compositor to be ready to render another frame (this is to reduce latency)
        app.compositor.wait_for_surface(self.layer);
    }

    pub(crate) fn set_debug_overlay(&mut self, debug_overlay: Option<DebugOverlayData>) {
        self.debug_overlay = debug_overlay;
    }
}

impl WindowHandler for UiHostWindowHandler {
    /*fn connect(&mut self, handle: &WindowHandle) {
        let idle_handle = handle.get_idle_handle().unwrap();
        self.handle = Some(handle.clone());
        self.idle_handle = Some(handle.get_idle_handle().unwrap());
        // create composition layer
        let app = AppGlobals::get();
        let size = handle.get_size();
        let layer = app.compositor.create_surface_layer(size, ColorType::RGBAF16);
        self.layer = Some(layer);
        // SAFETY: the raw window handle is valid
        unsafe {
            app.compositor.bind_layer(layer, handle.raw_window_handle());
            // this initial wait is important
            app.compositor.wait_for_surface(layer);
        }
    }*/

    /*fn size(&mut self, size: Size) {
        trace!("UiHostWindowHandler::size({size})");
        // resize the layer
        let app = AppGlobals::get();
        app.compositor.set_surface_layer_size(self.layer.unwrap(), size);
        // relayout
        self.update_layout();
    }*/

    /*fn prepare_paint(&mut self) {
        // submit damage regions
        let handle = self.handle.clone().unwrap();
        for r in mem::take(&mut self.damage_regions.regions) {
            handle.invalidate_rect(r);
        }
    }*/

    /*fn pointer_move(&mut self, event: &glazier::PointerEvent) {
        //trace!("UiHostWindowHandler::pointer_move");
        self.propagate_pointer_event(PointerEventKind::PointerMove, event);
        // somehow schedule a recomp
        self.app_handle.schedule_update();
    }

    fn pointer_down(&mut self, event: &glazier::PointerEvent) {
        trace!("UiHostWindowHandler::pointer_down");
        self.propagate_pointer_event(PointerEventKind::PointerDown, event);
        // FIXME: this might be called multiple times
        self.app_handle.schedule_update();
    }

    fn pointer_up(&mut self, event: &glazier::PointerEvent) {
        trace!("UiHostWindowHandler::pointer_up");
        self.propagate_pointer_event(PointerEventKind::PointerUp, event);
        self.app_handle.schedule_update();
    }*/

    /*fn request_close(&mut self) {
        self.handle.as_ref().unwrap().close();
    }*/

    /*fn idle(&mut self, token: IdleToken) {
        trace!("UiHostWindowHandler::idle({token:?})");
        match token {
            CONTENT_UPDATE => {
                self.update_element_tree();
            }
            _ => {
                warn!("unknown idle token {token:?}")
            }
        }
    }*/

    fn event(&mut self, event: &WindowEvent, time: Duration) -> bool {
        let mut should_update_ui = false;
        match event {
            WindowEvent::Resized(new_size) => {
                if new_size.width != 0 && new_size.height != 0 {
                    let size = Size::new(new_size.width as f64, new_size.height as f64);
                    let app = AppGlobals::get();
                    app.compositor.set_surface_layer_size(self.layer, size);
                    self.update_layout();
                }
            }
            WindowEvent::KeyboardInput {
                device_id,
                event,
                is_synthetic,
            } => {
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
                    self.send_event(
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
            WindowEvent::ModifiersChanged(modifiers) => {
                self.input_state.modifiers = modifiers_from_winit(*modifiers);
            }
            WindowEvent::CursorMoved { position, device_id } => {
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
                        if last.device_id == *device_id
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
                                device_id: *device_id,
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
            }
            WindowEvent::RedrawRequested => self.paint(),
            WindowEvent::CloseRequested => {
                self.close_requested = true;
                should_update_ui = true;
            }
            /*
            WindowEvent::ActivationTokenDone { .. } => {}
            WindowEvent::Moved(_) => {}
            WindowEvent::Destroyed => {}
            WindowEvent::DroppedFile(_) => {}
            WindowEvent::HoveredFile(_) => {}
            WindowEvent::HoveredFileCancelled => {}
            WindowEvent::Focused(_) => {}
            WindowEvent::Ime(_) => {}
            WindowEvent::CursorEntered { .. } => {}
            WindowEvent::CursorLeft { .. } => {}
            WindowEvent::MouseWheel { .. } => {}
            WindowEvent::TouchpadMagnify { .. } => {}
            WindowEvent::SmartMagnify { .. } => {}
            WindowEvent::TouchpadRotate { .. } => {}
            WindowEvent::TouchpadPressure { .. } => {}
            WindowEvent::AxisMotion { .. } => {}
            WindowEvent::Touch(_) => {}
            WindowEvent::ScaleFactorChanged { .. } => {}
            WindowEvent::ThemeChanged(_) => {}
            WindowEvent::Occluded(_) => {}*/
            _ => return false,
        };

        should_update_ui
    }

    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

//
// The application logic and the cache is stored in `AppState`, accessed though shared `AppHandle`s.
//
// AppState receives messages via the glazier::AppHandler trait. One important message is
// `UI_UPDATE`: in response, AppState re-runs the application logic.
//
// The application logic is a function (or closure), that, when invoked, forms a call trace.
// The cache can be used to retrieve data at particular locations in the call trace that were stored
// from a previous run.
//
// The application logic is in charge of creating and updating the windows, with `AppWindowBuilder`.
// `AppWindowBuilder` takes an `AppHandle`.
// Within the app logic,
//

pub struct AppWindowBuilder<T> {
    title: String,
    content: T,
    id: State<Option<WindowId>>,
}

impl<T: Widget> AppWindowBuilder<T> {
    #[composable]
    pub fn new(content: T) -> AppWindowBuilder<T>
    where
        T: Widget,
    {
        let id = State::default();
        AppWindowBuilder {
            title: "".to_string(),
            content,
            id,
        }
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }
}

pub struct AppWindowHandle {
    window_id: WindowId,
    close_requested: bool,
}

impl AppWindowHandle {
    pub fn close_requested(&self) -> bool {
        self.close_requested
    }

    pub fn close(&self, app_ctx: &mut AppCtx) {
        app_ctx.close_window(self.window_id);
    }
}

impl<T: Widget + 'static> AppWindowBuilder<T> {
    pub fn build(self, cx: &mut AppCtx, env: &Environment) -> AppWindowHandle {
        let id = self.id.get();
        match id {
            None => {
                // create the window
                let window = WindowBuilder::new()
                    .build(cx.event_loop)
                    .expect("failed to create window");
                let size = window.inner_size();
                let window_id = window.id();
                let raw_window_handle = window.raw_window_handle()/*.expect("failed to get raw window handle")*/;

                // create a compositor layer for the window
                let app = AppGlobals::get();
                let layer = app
                    .compositor
                    .create_surface_layer(Size::new(size.width as f64, size.height as f64), ColorType::RGBAF16);

                let mut handler = UiHostWindowHandler::new(window, layer);

                unsafe {
                    // Bind the layer to the window
                    // SAFETY: idk? the window handle is valid?
                    app.compositor.bind_layer(layer, raw_window_handle);
                }
                // the initial wait is important: see https://learn.microsoft.com/en-us/windows/uwp/gaming/reduce-latency-with-dxgi-1-3-swap-chains#step-4-wait-before-rendering-each-frame
                app.compositor.wait_for_surface(layer);

                // update the content
                // FIXME: avoid boxing here?
                handler.update_content(Box::new(self.content));
                cx.register_window(window_id, Box::new(handler));
                self.id.set_without_invalidation(Some(window_id));
                AppWindowHandle {
                    window_id,
                    close_requested: false,
                }
            }
            Some(id) => {
                let handler = cx
                    .window_handler(id)
                    .as_any()
                    .downcast_mut::<UiHostWindowHandler>()
                    .expect("unexpected window type");
                handler.window.set_title(&self.title);
                handler.update_content(Box::new(self.content));
                let close_requested = mem::take(&mut handler.close_requested);
                AppWindowHandle {
                    window_id: id,
                    close_requested,
                }
            }
        }
    }
}

/*/// Child window widget
pub struct Window<T> {
    title: Option<String>,
    content: T,
}

impl<T: Widget> Widget for Window<T> {
    type Element = WindowElement<T>;

    fn build(self, cx: &mut TreeCtx, env: &Environment) -> Self::Element {
        let app = glazier::Application::global();
        let handle = glazier::WindowBuilder::new(app)
            .title(self.title.unwrap_or_default())
            .handler(UiHostWindowHandler::new())
            .build()
            .unwrap();
        let mut tree_handle : WidgetTreeHandle = todo!();
        tree_handle.set(Box::new(self.content));
        let content = self.content.build(cx, env);
        WindowElement {
            handle,
            content: Box::new(content),
        }
    }

    fn update(self, cx: &mut TreeCtx, element: &mut Self::Element, env: &Environment) {
        if let Some(title) = self.title {
            element.handle.set_title(&title);
        }
        element.tree_handle.set(Box::new())
        self.content.update(cx, &mut element.content, env);
        // TODO if update flags != unchanged, request window repaint with repaint damage region

    }
}

pub struct WindowElement{
    handle: WindowHandle,
    tree_handle: WidgetTreeHandle,
}

impl<T> Element for WindowElement<T> {
    fn id(&self) -> Option<WidgetId> {
        todo!()
    }

    fn layout(&self, ctx: &mut LayoutCtx, params: &LayoutParams) -> Geometry {
        todo!()
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}*/
