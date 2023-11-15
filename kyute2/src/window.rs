//! UI host windows
mod key;

use crate::{
    application::{AppCtx, AppState, ExtEvent, WindowHandler},
    composition,
    composition::{ColorType, LayerID},
    context::ElementTree,
    debug_util,
    debug_util::{
        DebugEventInfo, DebugLayoutInfo, DebugPaintInfo, DebugSnapshot, DebugSnapshotCause, DebugWriter, ElementPtrId,
    },
    drawing::ToSkia,
    event::{KeyboardEvent, PointerButton, PointerButtons, PointerEvent},
    theme,
    widget::{null::NullElement, WidgetExt},
    AnyWidget, AppGlobals, BoxConstraints, ChangeFlags, Color, Element, ElementId, Event, EventCtx, EventKind,
    Geometry, HitTestResult, LayoutCtx, PaintCtx, Point, Rect, Size, TreeCtx, Widget,
};
use bitflags::Flags;
use keyboard_types::KeyState;
use kyute2::{
    debug_util::DebugPaintElementInfo,
    window::key::{key_code_from_winit, modifiers_from_winit},
};
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
use std::{
    any::Any,
    cell::RefCell,
    collections::HashMap,
    mem,
    rc::Rc,
    time::{Duration, Instant},
};
use tracing::{info, trace, trace_span, warn};
use winit::{
    event::{DeviceId, ElementState, MouseButton, WindowEvent},
    event_loop::EventLoopWindowTarget,
    keyboard::KeyLocation,
    platform::windows::WindowBuilderExtWindows,
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

/// Utility function to create a compositing layer for a window.
fn create_composition_layer(window: &winit::window::Window) -> LayerID {}

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
    fn get_propagation_path(&mut self, mut target: ElementId) -> Vec<ElementId> {
        let mut result = vec![target];
        while let Some(parent) = self.element_tree.get(&target) {
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
            debug_info: Default::default(),
        };

        let root = event.next_target().expect("route should have at least one element");
        assert_eq!(root, self.root.id());
        let change_flags = ctx.event(&mut self.root, event);

        self.debug_event_info = ctx.debug_info;

        #[cfg(debug_assertions)]
        {
            dbg!(self.focus);
            dbg!(self.pointer_grab);
            // record a snapshot of the UI after delivering the event
            self.record_debug_snapshot(DebugSnapshotCause::Event, time);
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
                // send to "most specific" (deepest) hit in the stack that is not anonymous
                let target = *htr
                    .hits
                    .iter()
                    .find(|id| !id.is_anonymous())
                    .expect("successful hit test result should contain at least one element");
                let route = self.get_propagation_path(target);
                eprintln!("hit-test @ {:?} target={:?} route={:?}", position, target, &route[..]);
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

/// A window handler that hosts a UI tree.
pub(crate) struct UiHostWindowHandler {
    /// Winit window
    pub(crate) window: winit::window::Window,
    /// Root composition layer for the window.
    layer: composition::LayerID,
    /// Damage regions to be repainted.
    damage_regions: DamageRegions,
    pub(crate) element_tree: ElementTree,
    /// Root of the UI element tree.
    pub(crate) root: Box<dyn Element>,
    // Run loop handle
    //app_handle: AppHandle,
    pub(crate) focus: Option<ElementId>,
    pub(crate) pointer_grab: Option<ElementId>,
    input_state: InputState,
    last_click: Option<LastClick>,
    close_requested: bool,

    pub(crate) debug_overlay: Option<DebugOverlayData>,

    /// Debug info collected during the last call to update_layout.
    debug_layout_info: DebugLayoutInfo,
    /// Debug info collected during the last event propagation.
    debug_event_info: DebugEventInfo,
    /// Debug info collected during the last repaint.
    debug_paint_info: DebugPaintInfo,
}

impl UiHostWindowHandler {
    /// Creates a new `UiHostWindowHandler`.
    pub fn new(window: winit::window::Window) -> UiHostWindowHandler {
        // create a compositor layer for the window
        let size = window.inner_size();
        let app = AppGlobals::get();
        let raw_window_handle = window.raw_window_handle()/*.expect("failed to get raw window handle")*/;
        let layer = app
            .compositor
            .create_surface_layer(Size::new(size.width as f64, size.height as f64), ColorType::RGBAF16);
        unsafe {
            // Bind the layer to the window
            // SAFETY: idk? the window handle is valid?
            app.compositor.bind_layer(layer, raw_window_handle);
        }

        // On windows, the initial wait is important:
        // see https://learn.microsoft.com/en-us/windows/uwp/gaming/reduce-latency-with-dxgi-1-3-swap-chains#step-4-wait-before-rendering-each-frame
        app.compositor.wait_for_surface(layer);

        // TODO remove layer on drop?
        UiHostWindowHandler {
            window,
            layer,
            damage_regions: DamageRegions::default(),
            element_tree: Default::default(),
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
            debug_layout_info: Default::default(),
            debug_event_info: Default::default(),
            debug_paint_info: Default::default(),
        }
    }

    /*fn get<'a>(app_ctx: &'a mut AppCtx, window_id: WindowId) -> &'a mut UiHostWindowHandler {
        let handler = app_ctx.window_handler(window_id);
        handler
            .as_any()
            .downcast_mut::<UiHostWindowHandler>()
            .expect("unexpected window type")
    }*/

    /// Helper function to update the contents of a window from a widget.
    ///
    /// This is a static function instead of a method because a `&mut` method would
    /// borrow-lock `app_ctx`, but we need `app_ctx` to build or update widgets.
    ///
    /// # Preconditions
    ///
    /// - the handler for the window identified by `window_id` must be a `UiHostWindowHandler`.
    fn update<T>(&mut self, cx: &mut TreeCtx, content: T)
    where
        T: Widget + Any,
    {
        // provide default theme values
        let content = content.provide(theme::DARK_THEME);
        let change_flags = cx.update_with_element_tree(content, &mut self.root, &mut self.element_tree);

        // handle changes reported during the update of the element tree
        eprintln!("update_content: {:?}", change_flags);
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
        let layout_params = BoxConstraints {
            min: Size::ZERO,
            max: Size::new(window_size.width, window_size.height),
        };
        let geometry = ctx.layout(&mut self.root, &layout_params);
        trace!(
            "update_layout window_size:{:?}, result geometry:{:?}",
            window_size,
            geometry
        );

        #[cfg(debug_assertions)]
        {
            self.debug_layout_info = ctx.debug_info;
        }

        //self.window.request_redraw();
    }

    fn record_debug_snapshot(&mut self, cause: DebugSnapshotCause, time: Duration) {
        if debug_util::is_collection_enabled() {
            let ui_tree = debug_util::dump_ui_tree(&self.root);
            debug_util::record_ui_snapshot(DebugSnapshot {
                cause,
                time,
                window: self.window.id(),
                root: ui_tree,
                element_tree: self.element_tree.clone(),
                layout_info: self.debug_layout_info.clone(),
                paint_info: self.debug_paint_info.clone(),
                event_info: mem::take(&mut self.debug_event_info),
                focused: self.focus,
                pointer_grab: self.pointer_grab,
            });
        }
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
    fn paint(&mut self, time: Duration) {
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
            debug_info: Default::default(),
        };

        paint_ctx.paint(&mut self.root);

        // paint debug overlay
        self.debug_overlay.as_ref().map(|overlay| overlay.paint(&mut paint_ctx));

        app.compositor.release_drawing_surface(self.layer, paint_ctx.surface);
        // wait for the compositor to be ready to render another frame (this is to reduce latency)
        app.compositor.wait_for_surface(self.layer);

        #[cfg(debug_assertions)]
        {
            self.debug_paint_info = paint_ctx.debug_info;
            self.record_debug_snapshot(DebugSnapshotCause::AfterPaint, time);
        }
    }

    pub(crate) fn set_debug_overlay(&mut self, debug_overlay: Option<DebugOverlayData>) {
        self.debug_overlay = debug_overlay;
    }
}

impl WindowHandler for UiHostWindowHandler {
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
            WindowEvent::RedrawRequested => self.paint(time),
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

    fn window_id(&self) -> WindowId {
        self.window.id()
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

/// Handle to an top-level window.
pub struct AppWindowHandle {
    window_id: WindowId,
    close_requested: bool,
}

impl AppWindowHandle {
    pub fn close_requested(&self) -> bool {
        self.close_requested
    }
    pub fn close(&self, ctx: &mut TreeCtx) {
        ctx.close_window(self.window_id);
    }

    pub fn update<T>(&self, cx: &mut TreeCtx, content: T)
    where
        T: Widget + Any,
    {
        cx.with_window_handler(self.window_id, |cx, handler| {
            handler
                .as_any()
                .downcast_mut::<UiHostWindowHandler>()
                .expect("unexpected window type")
                .update(cx, content)
        });
    }

    /// Creates a new top-level window and registers it to the event loop.
    pub fn new<T>(ctx: &mut TreeCtx, window_builder: WindowBuilder, content: T) -> AppWindowHandle
    where
        T: Widget + Any,
    {
        // create the window
        let window = window_builder
            .with_no_redirection_bitmap(true)
            .build(ctx.event_loop)
            .expect("failed to create window");
        let window_id = window.id();

        // register handler
        ctx.register_window(window_id, Box::new(UiHostWindowHandler::new(window)));

        // Initial update
        // TODO we could build the initial element before registering the window handler,
        // but that's more code to write, and this works just as well at the price of some
        // additional hash map lookups.
        ctx.with_window_handler(window_id, |cx, handler| {
            handler
                .as_any()
                .downcast_mut::<UiHostWindowHandler>()
                .expect("unexpected window type")
                .update(cx, content)
        });

        AppWindowHandle {
            window_id,
            close_requested: false,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct ChildWindowElement {
    window_id: WindowId,
}

impl Element for ChildWindowElement {
    fn id(&self) -> ElementId {
        todo!()
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, params: &BoxConstraints) -> Geometry {
        Geometry::ZERO
    }

    fn event(&mut self, ctx: &mut EventCtx, event: &mut Event) -> ChangeFlags {
        ChangeFlags::NONE
    }

    fn natural_width(&mut self, height: f64) -> f64 {
        0.0
    }

    fn natural_height(&mut self, width: f64) -> f64 {
        0.0
    }

    fn natural_baseline(&mut self, params: &BoxConstraints) -> f64 {
        0.0
    }

    fn hit_test(&self, ctx: &mut HitTestResult, position: Point) -> bool {
        false
    }

    fn paint(&mut self, ctx: &mut PaintCtx) {}

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn debug(&self, w: &mut DebugWriter) {
        w.type_name("ChildWindowElement");
        w.property("window_id", &self.window_id);
    }
}

/// Child window widget
pub struct ChildWindow<T> {
    window_builder: WindowBuilder,
    content: T,
}

impl<T: Widget> Widget for ChildWindow<T> {
    type Element = ChildWindowElement;

    fn build(self, cx: &mut TreeCtx, id: ElementId) -> Self::Element {
        // build the window
        // Pretty much the same code as top-level windows, except we're creating a child window.
        let parent_window_handle = cx
            .parent_window()
            .expect("a child window must be created within a parent window");

        // SAFETY: we're not safe, really, we have no way to check that the handle reported by
        // the parent window is handle. As long as there's only our windows we're fine,
        // but if clients start creating their own windows and WindowHandler, it's their responsibility to
        // ensure that the handle is valid.
        let window = unsafe {
            self.window_builder
                .with_no_redirection_bitmap(true)
                .with_parent_window(Some(parent_window_handle))
                .build(cx.event_loop)
                .expect("failed to create window")
        };
        let window_id = window.id();

        // register handler
        cx.register_window(window_id, Box::new(UiHostWindowHandler::new(window)));
    }

    fn update(self, cx: &mut TreeCtx, element: &mut Self::Element) -> ChangeFlags {
        cx.with_window_handler(element.window_id, |cx, handler| {
            // TODO update window title
            handler
                .as_any()
                .downcast_mut::<UiHostWindowHandler>()
                .expect("unexpected window type")
                .update(cx, self.content)
        });
        ChangeFlags::NONE
    }
}
