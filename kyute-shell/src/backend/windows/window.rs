//! Platform-specific window creation
use crate::{
    application::Application,
    backend::{Layer, Menu, PlatformError},
    error::Error,
};
use kyute_common::{PointI, Size, SizeI};
use raw_window_handle::HasRawWindowHandle;
use std::ptr;
use windows::Win32::{
    Foundation::{HINSTANCE, HWND, POINT},
    Graphics::{Direct2D::Common::D2D1_COLOR_F, DirectComposition::IDCompositionTarget, Gdi::ClientToScreen},
    UI::WindowsAndMessaging::{DestroyMenu, SetMenu, TrackPopupMenu, HMENU, TPM_LEFTALIGN},
};
use winit::{
    event_loop::EventLoopWindowTarget,
    platform::windows::{WindowBuilderExtWindows, WindowExtWindows},
    window::{WindowBuilder, WindowId},
};

/// Encapsulates a Win32 window and associated resources for drawing to it.
pub struct Window {
    window: winit::window::Window,
    hwnd: HWND,
    hinstance: HINSTANCE,
    menu: Option<HMENU>,
    composition_target: IDCompositionTarget,
}

impl Window {
    /// Returns the underlying winit [`Window`].
    ///
    /// [`Window`]: winit::Window
    pub fn window(&self) -> &winit::window::Window {
        &self.window
    }

    /// Returns the underlying winit [`WindowId`].
    /// Equivalent to calling `self.window().id()`.
    ///
    /// [`WindowId`]: winit::WindowId
    pub fn id(&self) -> WindowId {
        self.window.id()
    }

    /// Sets this window's main menu bar.
    pub fn set_menu(&mut self, new_menu: Option<Menu>) {
        unsafe {
            // SAFETY: TODO
            if let Some(current_menu) = self.menu.take() {
                SetMenu(self.hwnd, None);
                DestroyMenu(current_menu);
            }
            if let Some(menu) = new_menu {
                let hmenu = menu.into_hmenu();
                SetMenu(self.hwnd, hmenu);
                self.menu = Some(hmenu);
            }
        }
    }

    /// Shows a context menu at the specified pixel location.
    pub fn show_context_menu(&self, menu: Menu, at: PointI) {
        unsafe {
            let hmenu = menu.into_hmenu();
            /*let scale_factor = self.window.scale_factor();
            let x = at.x * scale_factor;
            let y = at.y * scale_factor;*/
            let mut point = POINT { x: at.x, y: at.y };
            ClientToScreen(self.hwnd, &mut point);
            if TrackPopupMenu(hmenu, TPM_LEFTALIGN, point.x, point.y, 0, self.hwnd, ptr::null()) == false {
                tracing::warn!("failed to track popup menu");
            }
        }
    }

    /// Sets the root composition layer.
    pub fn set_root_composition_layer(&self, layer: &Layer) {
        unsafe {
            //layer.visual.EnableRedrawRegions();
            self.composition_target
                .SetRoot(layer.visual().clone())
                .expect("SetRoot failed");
            Application::instance()
                .backend
                .composition_device
                .get_ref()
                .unwrap()
                .Commit()
                .expect("Commit failed");
        }
    }

    pub fn scale_factor(&self) -> f64 {
        self.window.scale_factor()
    }

    /// Returns the logical size of the window's _client area_ in DIPs.
    pub fn logical_inner_size(&self) -> Size {
        let (w, h): (f64, f64) = self
            .window
            .inner_size()
            .to_logical::<f64>(self.window.scale_factor())
            .into();
        Size::new(w, h)
    }

    /// Returns the size of the window's _client area_ in physical pixels.
    pub fn physical_inner_size(&self) -> SizeI {
        let winit::dpi::PhysicalSize { width, height } = self.window.inner_size();
        SizeI::new(width as i32, height as i32)
    }

    /// Creates a new window from the options given in the provided [`WindowBuilder`].
    ///
    /// To create the window with an OpenGL context, `with_gl` should be `true`.
    ///
    /// [`WindowBuilder`]: winit::WindowBuilder
    pub fn new<T>(
        event_loop: &EventLoopWindowTarget<T>,
        mut builder: WindowBuilder,
        parent_window: Option<&Window>,
    ) -> Result<Window, Error> {
        let app = Application::instance();

        if let Some(parent_window) = parent_window {
            builder = builder.with_parent_window(parent_window.hwnd.0 as *mut _);
        }
        builder = builder.with_no_redirection_bitmap(true);
        let window = builder
            .build(event_loop)
            .map_err(|e| Error::Platform(PlatformError::Winit(e)))?;
        let hinstance = HINSTANCE(window.hinstance() as isize);
        let hwnd = HWND(window.hwnd() as isize);

        // create composition target
        let composition_device = app
            .backend
            .composition_device
            .get_ref()
            .expect("could not acquire composition device outside of main thread");
        let composition_target = unsafe {
            composition_device
                .CreateTargetForHwnd(hwnd, false)
                .expect("CreateTargetForHwnd failed")
        };

        // create a swap chain for the window
        //let device = app.gpu_device();
        //let surface = graal::surface::get_vulkan_surface(window.raw_window_handle());
        //let swapchain_size = window.inner_size().into();
        // ensure that the surface can be drawn to with the device that we created. must be called to
        // avoid validation errors.
        //unsafe {
        //    assert!(device.is_compatible_for_presentation(surface));
        //}
        //let swap_chain = unsafe { device.create_swapchain(surface, swapchain_size) };

        let pw = Window {
            window,
            hwnd,
            hinstance,
            // TODO menu initializer
            menu: None,
            composition_target,
        };

        Ok(pw)
    }
}
