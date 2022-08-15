//! window creation
use crate::{
    animation::Layer,
    application::Application,
    backend,
    error::Error,
    input::{keyboard::KeyboardEvent, pointer::PointerInputEvent},
    Menu,
};
use kyute_common::{PointI, Size, SizeI};
use raw_window_handle::HasRawWindowHandle;
use std::ptr;

/// Window levels, like those in AppKit.
#[derive(Copy, Clone, Debug)]
pub enum WindowLevel {
    /// Normal
    Normal,
    /// Floating panel
    Floating,
    /// PopUp panel
    PopUp,
    /// Modal panel
    Modal,
    /// Menus
    Menu,
}

// on recomp, the window handler can change (because it contains the widget tree)
// so really, the window handler is mutable
// WindowHandler -> Arc<dyn WindowHandler>
// - a copy is stored in the comp cache
// - on recomp, fetch copy, set (with interior mutability) the widget tree

// Why multiple handlers instead of a single event loop?
// -> this way, no need to traverse the whole tree when delivering an event to a particular window

pub trait WindowHandler {
    fn scale_factor_changed(&self, scale_factor: f64) {}
    fn resize(&self, size: SizeI) {}
    fn pointer_up(&self, event: &PointerInputEvent) {}
    fn pointer_down(&self, event: &PointerInputEvent) {}
    fn pointer_move(&self, event: &PointerInputEvent) {}
    fn key_up(&self, event: &KeyboardEvent) {}
    fn key_down(&self, event: &KeyboardEvent) {}
}

pub struct WindowBuilder {
    backend: backend::WindowBuilder,
}

impl WindowBuilder {
    /// Sets the title of the window.
    pub fn title(mut self, title: impl Into<String>) -> WindowBuilder {
        self.backend.set_title(title);
        self
    }

    /// Builds the window, with the specified function to create the window handler.
    pub fn build<Handler, Init>(self, init: Init) -> WindowHandle
    where
        Handler: WindowHandler,
        Init: FnOnce(WindowHandle) -> Handler,
    {
        let handle = self.backend.build(init);
        WindowHandle(handle)
    }
}

/// Encapsulates a window and associated resources for drawing to it.
pub struct WindowHandle(pub(crate) backend::WindowHandle);

impl WindowHandle {
    /*/// Returns the underlying winit [`Window`].
    ///
    /// [`Window`]: winit::Window
    pub fn window(&self) -> &winit::window::Window {
        &self.window
    }*/

    /// Returns the underlying winit [`WindowId`].
    /// Equivalent to calling `self.window().id()`.
    ///
    /// [`WindowId`]: winit::WindowId
    pub fn id(&self) -> WindowId {
        self.0.id()
    }

    /// Sets this window's main menu bar.
    pub fn set_menu(&mut self, new_menu: Option<Menu>) {
        self.0.set_menu(new_menu.map(Menu::into_inner))
    }

    /// Shows a context menu at the specified pixel location.
    pub fn show_context_menu(&self, menu: Menu, at: PointI) {
        self.0.show_context_menu(menu.into_inner(), at);
    }

    /// Sets the root composition layer.
    pub fn set_root_composition_layer(&self, layer: &Layer) {
        self.0.set_root_composition_layer(&layer.0);
    }

    /// Returns the scale factor.
    pub fn scale_factor(&self) -> f64 {
        self.0.scale_factor()
    }

    /// Returns the logical size of the window's _client area_ in DIPs.
    pub fn logical_inner_size(&self) -> Size {
        self.0.logical_inner_size()
    }

    /// Returns the size of the window's _client area_ in physical pixels.
    pub fn physical_inner_size(&self) -> SizeI {
        self.0.physical_inner_size()
    }

    pub fn set_cursor_icon(&mut self, cursor_icon: CursorIcon) {
        self.0.set_cursor_icon(cursor_icon)
    }

    /// Creates a new window from the options given in the provided [`WindowBuilder`].
    ///
    /// To create the window with an OpenGL context, `with_gl` should be `true`.
    ///
    /// [`WindowBuilder`]: winit::WindowBuilder
    pub fn from_builder<T>(
        event_loop: &EventLoopWindowTarget<T>,
        mut builder: WindowBuilder,
        parent_window: Option<&Window>,
    ) -> Result<Window, Error> {
        backend::Window::new(event_loop, builder, parent_window.map(|w| &w.0)).map(Window)
    }

    /// Creates a new window with the given title.
    pub fn new<T>(event_loop: &EventLoopWindowTarget<T>, title: impl Into<String>) -> Result<Window, Error> {
        backend::Window::new(event_loop, winit::window::WindowBuilder::new().with_title(title), None).map(Window)
    }
}
