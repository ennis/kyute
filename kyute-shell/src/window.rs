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
use std::{ptr, sync::Arc};

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
    /// Called when this handler has been attached to a window and the window has been created.
    fn connect(&self, window_handle: WindowHandle) {}

    /// Called when the scale factor of the window has changed.
    fn scale_factor_changed(&self, scale_factor: f64) {}

    /// The window was resized.
    fn resize(&self, size: SizeI) {}
    fn pointer_up(&self, event: &PointerInputEvent) {}
    fn pointer_down(&self, event: &PointerInputEvent) {}
    fn pointer_move(&self, event: &PointerInputEvent) {}
    fn key_up(&self, event: &KeyboardEvent) {}
    fn key_down(&self, event: &KeyboardEvent) {}
    fn close_requested(&self) {}
}

pub struct WindowBuilder {
    backend: backend::WindowBuilder,
}

impl WindowBuilder {
    pub fn new() -> WindowBuilder {
        WindowBuilder {
            backend: backend::WindowBuilder::new(),
        }
    }

    /// Sets the title of the window.
    pub fn title(mut self, title: impl Into<String>) -> WindowBuilder {
        self.backend.set_title(title.into());
        self
    }

    /// Builds the window, with the specified window event handler.
    pub fn build(self, handler: Arc<dyn WindowHandler>) -> Result<WindowHandle, Error> {
        let handle = self.backend.build(handler)?;
        Ok(WindowHandle(handle))
    }
}

/// Encapsulates a window and associated resources for drawing to it.
pub struct WindowHandle(pub(crate) backend::WindowHandle);

impl WindowHandle {
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

    /*/// Returns the logical size of the window's _client area_ in DIPs.
    pub fn logical_inner_size(&self) -> Size {
        self.0.logical_inner_size()
    }

    /// Returns the size of the window's _client area_ in physical pixels.
    pub fn physical_inner_size(&self) -> SizeI {
        self.0.physical_inner_size()
    }*/
}
