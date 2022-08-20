//! Platform-specific window creation
use crate::{
    application::Application,
    backend::{
        windows::{application::CLASS_NAME, util::ToWide},
        Layer, Menu, PlatformError,
    },
    error::Error,
    input::pointer::{PointerButton, PointerButtons, PointerId, PointerInputEvent, PointerType},
    window::{WindowHandler, WindowLevel},
};
use keyboard_types::Modifiers;
use kyute_common::{Point, PointI, Size, SizeI};
use once_cell::sync::Lazy;
use raw_window_handle::HasRawWindowHandle;
use std::{
    borrow::BorrowMut,
    cell::{Cell, RefCell},
    collections::{HashMap, HashSet},
    ffi::c_void,
    mem, ptr,
    rc::{Rc, Weak},
    sync::Arc,
};
use threadbound::ThreadBound;
use windows::{
    core::{implement, PCWSTR},
    Win32::{
        Foundation::{GetLastError, HINSTANCE, HWND, LPARAM, LRESULT, POINT, POINTL, WPARAM},
        Graphics::{Direct2D::Common::D2D1_COLOR_F, DirectComposition::IDCompositionTarget, Gdi::ClientToScreen},
        System::{
            Com::IDataObject,
            Ole::{IDropTarget, IDropTarget_Impl, RegisterDragDrop, RevokeDragDrop},
        },
        UI::{
            HiDpi::GetDpiForWindow,
            Input::Pointer::EnableMouseInPointer,
            WindowsAndMessaging::{
                CreateWindowExW, DefWindowProcW, DestroyMenu, DrawMenuBar, GetWindowLongPtrW, SetMenu,
                SetWindowLongPtrW, ShowWindow, TrackPopupMenu, CREATESTRUCTW, CW_USEDEFAULT, GWLP_USERDATA, HMENU,
                POINTER_MESSAGE_FLAG_FIFTHBUTTON, POINTER_MESSAGE_FLAG_FIRSTBUTTON, POINTER_MESSAGE_FLAG_FOURTHBUTTON,
                POINTER_MESSAGE_FLAG_SECONDBUTTON, POINTER_MESSAGE_FLAG_THIRDBUTTON, SW_SHOWDEFAULT, TPM_LEFTALIGN,
                WINDOW_EX_STYLE, WM_CREATE, WM_DPICHANGED, WM_NCDESTROY, WM_POINTERDOWN, WM_POINTERENTER,
                WM_POINTERLEAVE, WM_POINTERUP, WM_POINTERUPDATE, WS_CHILD, WS_EX_NOACTIVATE, WS_EX_NOREDIRECTIONBITMAP,
                WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_MAXIMIZEBOX, WS_MINIMIZEBOX, WS_OVERLAPPED, WS_OVERLAPPEDWINDOW,
                WS_POPUP, WS_SYSMENU, WS_THICKFRAME,
            },
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

/// Pointer input state.
#[derive(Copy, Clone, Default)]
struct PointerInputState {
    buttons: PointerButtons,
    position: Point,
}

// Most of the fields are interior mutable because they are set *after* the window is created,
// because the pointer to WindowState is passed to CreateWindow
struct WindowState {
    hwnd: Cell<HWND>,
    menu: Cell<Option<HMENU>>,
    composition_target: RefCell<Option<IDCompositionTarget>>,
    pointer_input_state: RefCell<HashMap<PointerId, PointerInputState>>,
    modifiers_state: Cell<Modifiers>,
    handler: Arc<dyn WindowHandler>,
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
            WM_DPICHANGED => {
                let dpi = (wparam.0 & 0xFFFF) as f64;
                let scale_factor = dpi / 96.0;
                self.handler.scale_factor_changed(scale_factor);
                LRESULT(0)
            }
            WM_POINTERDOWN | WM_POINTERUPDATE | WM_POINTERUP => {
                let mut ptr_states = self.pointer_input_state.borrow_mut();
                let pointer_id = PointerId((wparam.0 & 0xFFFF) as u64);
                let ptr_state = ptr_states.entry(pointer_id).or_insert_with(PointerInputState::default);

                let pointer_flags = (wparam.0 as u32 & 0xFFFF0000) >> 16;

                let mut buttons = PointerButtons::new();
                if pointer_flags & POINTER_MESSAGE_FLAG_FIRSTBUTTON != 0 {
                    buttons.set(PointerButton::LEFT);
                }
                if pointer_flags & POINTER_MESSAGE_FLAG_SECONDBUTTON != 0 {
                    buttons.set(PointerButton::RIGHT);
                }
                if pointer_flags & POINTER_MESSAGE_FLAG_THIRDBUTTON != 0 {
                    buttons.set(PointerButton::MIDDLE);
                }
                if pointer_flags & POINTER_MESSAGE_FLAG_FOURTHBUTTON != 0 {
                    buttons.set(PointerButton::X1);
                }
                if pointer_flags & POINTER_MESSAGE_FLAG_FIFTHBUTTON != 0 {
                    buttons.set(PointerButton::X2);
                }

                let x = ((lparam.0 as u32) & 0xFFFF) as u16 as i16 as i32;
                let y = (((lparam.0 as u32) & 0xFFFF0000) >> 16) as u16 as i16 as i32;
                let position = Point::new(x as f64, y as f64);
                let modifiers = self.modifiers_state.get();

                trace!(
                    "{}: pos={:?} btns={:?} flags={:016b}",
                    match umsg {
                        WM_POINTERDOWN => "WM_POINTERDOWN",
                        WM_POINTERUPDATE => "WM_POINTERUPDATE",
                        WM_POINTERUP => "WM_POINTERUP",
                        _ => unreachable!(),
                    },
                    position,
                    buttons,
                    pointer_flags
                );

                let event = PointerInputEvent {
                    pointer_id,
                    button: None,
                    repeat_count: 0,
                    contact_width: 0.0,
                    contact_height: 0.0,
                    pressure: 0.0,
                    tangential_pressure: 0.0,
                    tilt_x: 0,
                    tilt_y: 0,
                    twist: 0,
                    pointer_type: PointerType::Mouse,
                    position,
                    modifiers,
                    buttons: Default::default(),
                    primary: false,
                };

                DefWindowProcW(hwnd, umsg, wparam, lparam)
            }
            _ => DefWindowProcW(hwnd, umsg, wparam, lparam),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// wndproc
////////////////////////////////////////////////////////////////////////////////////////////////////

/*// Window map
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
}*/

pub(crate) unsafe extern "system" fn win_proc_dispatch(
    hwnd: HWND,
    u_msg: u32,
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
    if u_msg == WM_CREATE {
        eprintln!("WM_CREATE");
        let create_struct = &*(l_param.0 as *const CREATESTRUCTW);
        let window_ptr = create_struct.lpCreateParams;
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, window_ptr as isize);
    }

    // recover WindowState pointer
    let window_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const WindowState;

    if window_ptr.is_null() {
        DefWindowProcW(hwnd, u_msg, w_param, l_param)
    } else {
        let result = (*window_ptr).window_proc(hwnd, u_msg, w_param, l_param);
        // We leaked the Rc<WindowState> object when we transferred it to the window userdata,
        // so it's our responsibility to free it when the window is being destroyed.
        if u_msg == WM_NCDESTROY && !window_ptr.is_null() {
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
            drop(Rc::from_raw(window_ptr));
        }
        result
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// WindowBuilder
////////////////////////////////////////////////////////////////////////////////////////////////////

pub(crate) struct WindowBuilder {
    title: String,
    position: Option<Point>,
    size: Option<SizeI>,
    parent: Option<WindowHandle>,
    level: WindowLevel,
    show_titlebar: bool,
    resizeable: bool,
}

impl WindowBuilder {
    pub fn new() -> WindowBuilder {
        WindowBuilder {
            title: "".to_string(),
            position: None,
            size: None,
            parent: None,
            level: WindowLevel::Normal,
            show_titlebar: true,
            resizeable: true,
        }
    }

    pub fn set_title(&mut self, title: String) {
        self.title = title;
    }

    pub fn build(self, handler: Arc<dyn WindowHandler>) -> Result<WindowHandle, PlatformError> {
        // ensure that the window class is registered
        Application::instance();

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

            if !self.resizeable {
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
                pointer_input_state: RefCell::new(Default::default()),
                modifiers_state: Cell::new(Default::default()),
                handler,
            });

            let handle = WindowHandle {
                state: Rc::downgrade(&window_state),
            };

            let hwnd_parent = match self.parent {
                Some(parent) => parent.hwnd(),
                None => HWND::default(),
            };

            let hwnd = CreateWindowExW(
                dw_ex_style,
                PCWSTR(class_name.as_ptr()),
                PCWSTR(self.title.to_wide().as_ptr()),
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

            // install our IDropTarget handler
            let drop_target: IDropTarget = DropTarget {}.into();
            RevokeDragDrop(hwnd);
            if let Err(e) = RegisterDragDrop(hwnd, &drop_target) {
                warn!("RegisterDragDrop failed: {}", e);
            }

            ShowWindow(hwnd, SW_SHOWDEFAULT);

            Ok(handle)
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Window
////////////////////////////////////////////////////////////////////////////////////////////////////

/*/// Encapsulates a Win32 window and associated resources for drawing to it.
pub struct Window {
    window: winit::window::Window,
    hwnd: HWND,
    hinstance: HINSTANCE,
    menu: Option<HMENU>,
    composition_target: IDCompositionTarget,
}*/

impl WindowHandle {
    fn hwnd(&self) -> HWND {
        if let Some(state) = self.state.upgrade() {
            state.hwnd.get()
        } else {
            error!("window was closed");
            HWND::default()
        }
    }

    pub fn get_scale_factor(&self) -> f64 {
        if let Some(state) = self.state.upgrade() {
            let dpi = unsafe { GetDpiForWindow(state.hwnd.get()) };
            (dpi as f64) / 96.0
        } else {
            error!("window was closed");
            1.0
        }
    }

    /// Sets this window's main menu bar.
    pub fn set_menu(&self, new_menu: Option<Menu>) -> Result<(), Error> {
        if let Some(state) = self.state.upgrade() {
            unsafe {
                // SAFETY: TODO
                if let Some(current_menu) = state.menu.take() {
                    SetMenu(state.hwnd.get(), None);
                    DestroyMenu(current_menu);
                }
                if let Some(menu) = new_menu {
                    let hmenu = menu.into_hmenu();
                    SetMenu(state.hwnd.get(), hmenu);
                    state.menu.set(Some(hmenu));
                }
            }
            Ok(())
        } else {
            Err(Error::WindowClosed)
        }
    }

    /// Shows a context menu at the specified pixel location.
    pub fn show_context_menu(&self, menu: Menu, at: PointI) -> Result<(), Error> {
        if let Some(state) = self.state.upgrade() {
            unsafe {
                let hmenu = menu.into_hmenu();
                /*let scale_factor = self.window.scale_factor();
                let x = at.x * scale_factor;
                let y = at.y * scale_factor;*/
                let mut point = POINT { x: at.x, y: at.y };
                let hwnd = state.hwnd.get();
                ClientToScreen(hwnd, &mut point);
                if TrackPopupMenu(hmenu, TPM_LEFTALIGN, point.x, point.y, 0, hwnd, ptr::null()) == false {
                    tracing::warn!("TrackPopupMenu failed");
                }
            }
            Ok(())
        } else {
            Err(Error::WindowClosed)
        }
    }

    /// Sets the root composition layer.
    pub fn set_root_composition_layer(&self, layer: &Layer) -> Result<(), Error> {
        if let Some(state) = self.state.upgrade() {
            let composition_target = state.composition_target.borrow();
            if let Some(composition_target) = composition_target.as_ref() {
                unsafe {
                    composition_target.SetRoot(layer.visual()).expect("SetRoot failed");
                    Application::instance()
                        .backend
                        .composition_device
                        .get_ref()
                        .unwrap()
                        .Commit()
                        .expect("Commit failed");
                }
            }
            Ok(())
        } else {
            Err(Error::WindowClosed)
        }
    }

    pub fn scale_factor(&self) -> f64 {
        // TODO
        warn!("unimplemented: scale_factor");
        1.0
    }

    /*/// Returns the logical size of the window's _client area_ in DIPs.
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
    }*/

    /*/// Sets the current cursor icon.
    pub fn set_cursor_icon(&mut self, cursor_icon: CursorIcon) {
        //self.window.set_cursor_icon(cursor_icon)
    }*/
}
