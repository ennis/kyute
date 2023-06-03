//! UI host windows

use crate::{
    app_state::AppHandle, composable, composition, composition::ColorType, widget::NullElement, AnyWidget, Application,
    ChangeFlags, Element, Environment, Event, Geometry, LayoutCtx, LayoutParams, Rect, Size, TreeCtx, Widget, WidgetId,
};
use bitflags::Flags;
use glazier::{raw_window_handle::HasRawWindowHandle, IdleHandle, IdleToken, PointerEvent, Region, WindowHandle};
use kyute_compose::{cache_cx, Cache};
use std::{any::Any, cell::RefCell, mem, rc::Rc};
use tracing::{trace, warn};

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

/// Sent to a window when its content has been updated (via WindowContentHandle).
pub(crate) const CONTENT_UPDATE: IdleToken = IdleToken::new(0);
pub(crate) const RECOMPOSE: IdleToken = IdleToken::new(0);

type WindowContentRef = Rc<RefCell<Option<Box<dyn AnyWidget>>>>;

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
}

////////////////////////////////////////////////////////////////////////////////////////////////////

/// UI host window handler
struct UiHostWindowHandler {
    /// Window handle
    handle: Option<WindowHandle>,
    /// Root composition layer for the window.
    layer: Option<composition::LayerID>,
    /// Damage regions to be repainted.
    damage_regions: DamageRegions,
    // The retained element tree
    //elem_tree: Tree,
    /// Root of the UI element tree.
    root: Box<dyn Element>,
    idle_handle: Option<IdleHandle>,
    /// Window contents (widget tree).
    ///
    /// This is updated from the application logic via `WindowContentHandle`s.
    /// When a content update is signalled to the window, the widget is consumed and applied to
    /// the retained element tree (`root` + `elem_tree`).
    content: WindowContentRef,
    /// Run loop handle
    app_handle: AppHandle,
    focus: WindowFocusState,
}

impl UiHostWindowHandler {
    fn new(app_handle: AppHandle) -> UiHostWindowHandler {
        UiHostWindowHandler {
            handle: None,
            layer: None,
            damage_regions: DamageRegions::default(),
            root: Box::new(NullElement),
            idle_handle: None,
            content: WindowContentRef::default(),
            app_handle,
            focus: WindowFocusState {},
        }
    }

    fn update_element_tree(&mut self) {
        let Some(content) = self.content.take() else { return };

        let mut tree_ctx = TreeCtx::new(self.app_handle.clone());
        // FIXME: this should come alongside the content
        let env = Environment::new();
        let mut change_flags = content.update(&mut tree_ctx, &mut self.root, &env);
        if change_flags.contains(ChangeFlags::STRUCTURE | ChangeFlags::LAYOUT) {
            self.update_layout();
        }
    }

    fn update_layout(&mut self) {
        let handle = self.handle.clone().unwrap();
        let window_size = handle.get_size();
        let window_scale = handle.get_scale();
        let mut ctx = LayoutCtx::new(handle.clone(), &mut self.focus);
        let layout_params = LayoutParams {
            widget_state: (),
            scale_factor: 1.0,
            min: Size::ZERO,
            max: window_size,
        };
        let geometry = self.root.layout(&mut ctx, &layout_params);
    }
}

impl glazier::WinHandler for UiHostWindowHandler {
    fn connect(&mut self, handle: &WindowHandle) {
        let idle_handle = handle.get_idle_handle().unwrap();
        self.handle = Some(handle.clone());
        self.idle_handle = Some(handle.get_idle_handle().unwrap());
        // create composition layer
        let app = Application::global();
        let mut compositor = app.compositor();
        let size = handle.get_size();
        let layer = compositor.create_surface_layer(size, ColorType::RGBAF16);
        self.layer = Some(layer);
        // SAFETY: the raw window handle is valid
        unsafe {
            compositor.bind_layer(layer, handle.raw_window_handle());
        }
    }

    fn size(&mut self, size: Size) {
        trace!("UiHostWindowHandler::size({size})");
        // resize the layer
        let app = Application::global();
        let mut compositor = app.compositor();
        compositor.set_surface_layer_size(self.layer.unwrap(), size);
    }

    fn prepare_paint(&mut self) {
        // submit damage regions
        let handle = self.handle.clone().unwrap();
        for r in mem::take(&mut self.damage_regions.regions) {
            handle.invalidate_rect(r);
        }
    }

    fn paint(&mut self, invalid: &Region) {
        trace!("UiHostWindowHandler::paint");
        let app = Application::global();
        let mut compositor = app.compositor();
        let image = compositor.acquire_drawing_surface(self.layer.unwrap());
        compositor.release_drawing_surface(self.layer.unwrap(), image);
        // acquire a drawing surface, then repaint, and swap invalid regions
    }

    fn pointer_move(&mut self, event: &PointerEvent) {
        trace!("UiHostWindowHandler::pointer_move");
        let event = Event::PointerMove(event.clone());
        // somehow schedule a recomp
        self.app_handle.schedule_update();
    }

    fn pointer_down(&mut self, event: &PointerEvent) {
        trace!("UiHostWindowHandler::pointer_down");
        let event = Event::PointerDown(event.clone());
        // FIXME: this might be called multiple times
        self.app_handle.schedule_update();
    }

    fn pointer_up(&mut self, event: &PointerEvent) {
        trace!("UiHostWindowHandler::pointer_up");
        let event = Event::PointerUp(event.clone());
        self.app_handle.schedule_update();
    }

    fn idle(&mut self, token: IdleToken) {
        trace!("UiHostWindowHandler::idle({token:?})");
        match token {
            CONTENT_UPDATE => {
                self.update_element_tree();
            }
            _ => {
                warn!("unknown idle token {token:?}")
            }
        }
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
}

impl<T: Widget> AppWindowBuilder<T> {
    pub fn new(content: T) -> AppWindowBuilder<T>
    where
        T: Widget,
    {
        AppWindowBuilder {
            title: "".to_string(),
            content,
        }
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }
}

impl<T: Widget + 'static> AppWindowBuilder<T> {
    #[composable]
    pub fn build(self, app_handle: AppHandle) {
        let (var, new) = cache_cx::variable(|| {
            let app = glazier::Application::global();
            trace!("AppWindowBuilder::build: new window (\"{}\")", self.title);
            let handler = UiHostWindowHandler::new(app_handle.clone());
            let content_handle = handler.content.clone();
            let window_handle = glazier::WindowBuilder::new(app)
                .title(self.title.clone())
                .transparent(true)
                .handler(Box::new(handler))
                .build()
                .expect("failed to create window");
            window_handle.show();
            let idle_handle = window_handle.get_idle_handle().expect("failed to get idle handle");
            (
                window_handle,
                WindowContentHandle {
                    content: content_handle,
                    idle_handle,
                },
            )
        });
        let (window, content_handle) = var.get();

        if !new {
            // update title
            window.set_title(&self.title)
        }

        // update content
        content_handle.set(Box::new(self.content))
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
