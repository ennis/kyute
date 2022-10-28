//! window creation
use crate::{animation::Layer, application::Application, backend, error::Error, Menu};
use kyute_common::{PointI, Size, SizeI};
use raw_window_handle::HasRawWindowHandle;
use std::ptr;
use winit::{
    event_loop::EventLoopWindowTarget,
    window::{CursorIcon, WindowBuilder, WindowId},
};

/// Encapsulates a window and associated resources for drawing to it.
pub struct Window(backend::Window);

impl Window {
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

    pub fn composition_commit(&self) {
        self.0.composition_commit()
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
