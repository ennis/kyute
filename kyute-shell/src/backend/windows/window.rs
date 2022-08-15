//! Platform-specific window creation
use crate::{
    application::Application,
    backend::{
        windows::{application::CLASS_NAME, util::ToWide},
        Layer, Menu, PlatformError,
    },
    error::Error,
    window::{WindowHandler, WindowLevel},
};
use kyute_common::{imbl::HashSet, Point, PointI, Size, SizeI};
use once_cell::sync::Lazy;
use raw_window_handle::HasRawWindowHandle;
use std::{
    borrow::BorrowMut,
    cell::{Cell, RefCell},
    ffi::c_void,
    mem, ptr,
    rc::{Rc, Weak},
};
use threadbound::ThreadBound;
use windows::{
    core::implement,
    Win32::{
        Foundation::{GetLastError, HINSTANCE, HWND, LPARAM, LRESULT, POINT, POINTL, WPARAM},
        Graphics::{Direct2D::Common::D2D1_COLOR_F, DirectComposition::IDCompositionTarget, Gdi::ClientToScreen},
        System::{
            Com::IDataObject,
            Ole::{IDropTarget, IDropTarget_Impl, RegisterDragDrop, RevokeDragDrop},
        },
        UI::WindowsAndMessaging::{
            CreateWindowExW, DefWindowProcW, DestroyMenu, DrawMenuBar, GetWindowLongPtrW, SetMenu, SetWindowLongPtrW,
            TrackPopupMenu, CREATESTRUCTW, CW_USEDEFAULT, GWLP_USERDATA, HMENU, TPM_LEFTALIGN, WINDOW_EX_STYLE,
            WM_CREATE, WM_NCDESTROY, WM_POINTERDOWN, WM_POINTERENTER, WM_POINTERLEAVE, WM_POINTERUP, WM_POINTERUPDATE,
            WS_CHILD, WS_EX_NOACTIVATE, WS_EX_NOREDIRECTIONBITMAP, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_MAXIMIZEBOX,
            WS_MINIMIZEBOX, WS_OVERLAPPED, WS_OVERLAPPEDWINDOW, WS_POPUP, WS_SYSMENU, WS_THICKFRAME,
        },
    },
};

////////////////////////////////////////////////////////////////////////////////////////////////////
// DropTarget
////////////////////////////////////////////////////////////////////////////////////////////////////

#[implement(IDropTarget)]
pub(crate) struct DropTarget {}

impl IDropTarget_Impl for DropTarget {
    fn DragEnter(
        &self,
        pdataobj: &Option<IDataObject>,
        grfkeystate: u32,
        pt: &POINTL,
        pdweffect: *mut u32,
    ) -> ::windows::core::Result<()> {
        trace!("DragEnter, pt={},{}", pt.x, pt.y);
        Ok(())
    }

    fn DragOver(&self, grfkeystate: u32, pt: &POINTL, pdweffect: *mut u32) -> ::windows::core::Result<()> {
        trace!("DragOver, pt={},{}", pt.x, pt.y);
        Ok(())
    }

    fn DragLeave(&self) -> ::windows::core::Result<()> {
        trace!("DragLeave");
        Ok(())
    }

    fn Drop(
        &self,
        pdataobj: &Option<IDataObject>,
        grfkeystate: u32,
        pt: &POINTL,
        pdweffect: *mut u32,
    ) -> ::windows::core::Result<()> {
        trace!("Drop, pt={},{}", pt.x, pt.y);
        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// WindowHandle
////////////////////////////////////////////////////////////////////////////////////////////////////

/// Handle to a window.
#[derive(Clone)]
pub(crate) struct WindowHandle {
    /// Weak ref to the window state.
    ///
    /// Becomes invalid once the window is closed (`WindowHandle::close`).
    /// Window handles don't prevent the underlying window to be closed, so this can become invalid.
    /// The only strong ref and sole owner of the window state is the pointer stored in the window userdata.
    state: Weak<WindowState>,
}

// Most of the field are interior mutability because they are set *after* the window is created,
// because the pointer to WindowState is passed to CreateWindow
struct WindowState {
    hwnd: Cell<HWND>,
    menu: Cell<Option<HMENU>>,
    composition_target: RefCell<Option<IDCompositionTarget>>,
}

impl WindowState {
    unsafe fn window_proc(&self, hwnd: HWND, umsg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        // we need to handle:
        // - pointer events: receive them, update internal knowledge of pointer position, build the pointer event, invoke the window handler
        // - keyboard events: same, also update modifiers
        // - text, etc: convert to unicode characters
        // - resize: resize internal composition target, invoke handler
        // - drag/drop: ???

        //
        match umsg {
            WM_POINTERDOWN | WM_POINTERUPDATE | WM_POINTERUP => {
                let pointer_id = wparam & 0xFFFF;
                let pointer_flags = wparam & 0xFFFF0000 >> 16;
                let x = (lparam & 0xFFFF) as u16 as i16 as i32;
                let y = (lparam & 0xFFFF0000 >> 16) as u16 as i16 as i32;
                LRESULT(0)
            }
            _ => DefWindowProcW(hwnd, umsg, wparam, lparam),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// wndproc
////////////////////////////////////////////////////////////////////////////////////////////////////

// Window map
static WINDOWS: Lazy<ThreadBound<RefCell<HashSet<HWND>>>> =
    Lazy::new(|| ThreadBound::new(RefCell::new(HashSet::default())));

/// Registers one of our windows.
fn register_window(hwnd: HWND) {
    let mut windows = WINDOWS.get_ref().unwrap().borrow_mut();
    windows.insert(hwnd);
}

/// Registers one of our windows.
fn unregister_window(hwnd: HWND) {
    let mut windows = WINDOWS.get_ref().unwrap().borrow_mut();
    windows.remove(&hwnd);
}

fn is_registered_window(hwnd: HWND) -> bool {
    // check if this is a registered window
    let mut windows = WINDOWS.get_ref().unwrap().borrow();
    windows.contains(&hwnd)
}

pub(crate) unsafe extern "system" fn win_proc_dispatch(
    hwnd: HWND,
    u_msg: UINT,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    /*// check if this is a registered window
    if !is_registered_window(hwnd) {
        // unknown window
        return LRESULT(0);
    }*/

    // On WM_CREATE, stash the pointer to the WindowState object in the window userdata.
    // This points to an `Rc<WindowState>` object.
    if msg == WM_CREATE {
        let create_struct = &*(l_param as *const CREATESTRUCTW);
        let window_ptr = create_struct.lpCreateParams;
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, window_ptr as isize);
    }

    // recover WindowState pointer
    let window_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const WindowState;

    if window_ptr.is_null() {
        DefWindowProcW(hwnd, msg, w_param, l_param)
    } else {
        let result = (*window_ptr).window_proc(hwnd, u_msg, w_param, l_param);
        // We leaked the Rc<WindowState> object when we transferred it to the window userdata,
        // so it's our responsibility to free it when the window is being destroyed.
        if msg == WM_NCDESTROY && !window_ptr.is_null() {
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
            drop(Rc::from_raw(window_ptr));
        }
        result
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// WindowBuilder
////////////////////////////////////////////////////////////////////////////////////////////////////

pub(crate) struct WindowBuilder<'a> {
    title: String,
    position: Option<Point>,
    size: Option<SizeI>,
    parent: Option<WindowHandle>,
    level: WindowLevel,
    show_titlebar: bool,
    resizeable: bool,
}

impl WindowBuilder {
    pub fn build<Handler, Init>(self, init: Init) -> Result<WindowHandle, PlatformError>
    where
        Init: FnOnce(crate::window::WindowHandle) -> Handler,
        Handler: WindowHandler,
    {
        unsafe {
            let class_name = CLASS_NAME.to_wide();

            // TODO: pos_x and pos_y are only scaled for windows with parents. But they need to be
            // scaled for windows without parents too.
            let (pos_x, pos_y) = match self.position {
                Some(pos) => (pos.x as i32, pos.y as i32),
                None => (CW_USEDEFAULT, CW_USEDEFAULT),
            };

            let (width, height) = match self.size {
                Some(size) => (size.width, size.height),
                None => (CW_USEDEFAULT, CW_USEDEFAULT),
            };

            let mut dw_style;
            let mut dw_ex_style;
            let mut focusable;

            // determine window style flags
            match self.level {
                WindowLevel::Normal => {
                    dw_style = WS_OVERLAPPEDWINDOW;
                    dw_ex_style = WINDOW_EX_STYLE::default();
                    focusable = true;
                }
                WindowLevel::Floating => {
                    dw_style = WS_POPUP;
                    dw_ex_style = WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW;
                    focusable = false;
                }
                WindowLevel::Menu => {
                    dw_style = WS_CHILD;
                    dw_ex_style = WINDOW_EX_STYLE::default();
                    focusable = true;
                }
                WindowLevel::Modal => {
                    dw_style = WS_OVERLAPPED;
                    dw_ex_style = WS_EX_TOPMOST;
                    focusable = true;
                }
                WindowLevel::PopUp => {
                    dw_style = WS_POPUP;
                    dw_ex_style = WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW;
                    focusable = false;
                }
            }

            if !self.resizable {
                dw_style &= !(WS_THICKFRAME | WS_MAXIMIZEBOX);
            }

            if !self.show_titlebar {
                dw_style &= !(WS_MINIMIZEBOX | WS_SYSMENU | WS_OVERLAPPED);
            }

            dw_ex_style |= WS_EX_NOREDIRECTIONBITMAP;

            let window_state = Rc::new(WindowState {
                hwnd: Default::default(),
                menu: Default::default(),
                composition_target: RefCell::new(None),
            });

            let handle = WindowHandle {
                state: Rc::downgrade(&window_state),
            };

            // create window handler now that we have a handle
            let handler = init(crate::window::WindowHandle(handle.clone()));

            let hwnd_parent = match self.parent {
                Some(parent) => parent.hwnd(),
                None => HWND::default(),
            };

            let hwnd = CreateWindowExW(
                dw_ex_style,
                class_name.as_ptr(),
                self.title.to_wide().as_ptr(),
                dw_style,
                pos_x,
                pos_y,
                width,
                height,
                hwnd_parent,
                HMENU::default(),
                HINSTANCE::default(),
                Rc::into_raw(window_state) as *const c_void,
            );

            if hwnd == HWND::default() {
                return Err(windows::core::Error::from_win32().into());
            }

            Ok(handle)
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Window
////////////////////////////////////////////////////////////////////////////////////////////////////

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
            self.composition_target.SetRoot(layer.visual()).expect("SetRoot failed");
            Application::instance()
                .backend
                .composition_device
                .get_ref()
                .unwrap()
                .Commit()
                .expect("Commit failed");
            //DrawMenuBar(self.hwnd);
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

    /// Sets the current cursor icon.
    pub fn set_cursor_icon(&mut self, cursor_icon: CursorIcon) {
        self.window.set_cursor_icon(cursor_icon)
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

        // Register this window as a drop target.
        let drop_target: IDropTarget = DropTarget {}.into();
        unsafe {
            // winit installs its own IDropTarget handler, remove it.
            RevokeDragDrop(hwnd);
            RegisterDragDrop(hwnd, &drop_target).expect("RegisterDragDrop failed");
        }

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

impl WindowHandle {
    fn hwnd(&self) -> HWND {
        if let Some(state) = self.state.upgrade() {
            state.hwnd.get()
        } else {
            error!("window was closed");
            HWND::default()
        }
    }
}
