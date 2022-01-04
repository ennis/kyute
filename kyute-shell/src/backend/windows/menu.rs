use crate::backend::windows::str_to_wstr;
use std::mem;
use windows::Win32::{
    Foundation::PWSTR,
    System::Com::{CoCreateInstance, CoInitialize, CLSCTX_INPROC_SERVER},
    UI::{
        Shell::{FileOpenDialog, IFileDialog, FOS_ALLOWMULTISELECT},
        WindowsAndMessaging::{
            AppendMenuW, CreateMenu, DestroyMenu, InsertMenuItemW, SetMenu, HMENU, MENUITEMINFOW,
            MFT_STRING, MF_POPUP, MF_SEPARATOR, MF_STRING, MIIM_FTYPE, MIIM_STRING,
        },
    },
};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    platform::windows::WindowExtWindows,
    window::WindowBuilder,
};

pub struct Menu {
    hmenu: HMENU,
}

impl Drop for Menu {
    fn drop(&mut self) {
        unsafe {
            // SAFETY: hmenu is valid
            DestroyMenu(self.hmenu);
        }
    }
}

impl Menu {
    /// Creates a new menu.
    pub fn new() -> Menu {
        let hmenu = unsafe {
            // SAFETY: no particular requirements
            CreateMenu()
        };
        Menu { hmenu }
    }

    pub(crate) fn into_hmenu(self) -> HMENU {
        let hmenu = self.hmenu;
        mem::forget(self);
        hmenu
    }

    pub fn add_item(&mut self, text: &str, id: usize, checked: bool, disabled: bool) {
        // TODO: checked, disabled
        let w_text = str_to_wstr(text);
        unsafe {
            // SAFETY: TODO
            AppendMenuW(
                self.hmenu,
                MF_STRING,
                id,
                PWSTR(w_text.as_ptr() as *mut u16),
            )
        };
    }

    pub fn add_submenu(&mut self, text: &str, submenu: Menu) {
        let sub_hmenu = submenu.into_hmenu();
        let w_text = str_to_wstr(text);
        unsafe {
            // SAFETY: TODO
            AppendMenuW(
                self.hmenu,
                MF_POPUP,
                sub_hmenu as usize,
                PWSTR(w_text.as_ptr() as *mut u16),
            );
        }
    }

    pub fn add_separator(&mut self) {
        unsafe {
            // SAFETY: `self.handle` is valid
            AppendMenuW(self.hmenu, MF_SEPARATOR, 0, None);
        }
    }
}
