use crate::{backend::windows::util::ToWide, Shortcut};
use std::mem;
use windows::{
    core::PCWSTR,
    Win32::UI::WindowsAndMessaging::{
        AppendMenuW, CreateMenu, CreatePopupMenu, DestroyMenu, HMENU, MF_CHECKED, MF_DISABLED, MF_POPUP, MF_SEPARATOR,
        MF_STRING,
    },
};

pub struct Menu {
    hmenu: HMENU,
    accels: Vec<(usize, Shortcut)>,
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
            CreateMenu().unwrap()
        };
        Menu { hmenu, accels: vec![] }
    }

    /// Creates a new menu.
    pub fn new_popup() -> Menu {
        let hmenu = unsafe {
            // SAFETY: no particular requirements
            CreatePopupMenu().unwrap()
        };
        Menu { hmenu, accels: vec![] }
    }

    pub(crate) fn into_hmenu(self) -> HMENU {
        let hmenu = self.hmenu;
        mem::forget(self);
        hmenu
    }

    pub fn add_item(&mut self, text: &str, id: usize, shortcut: Option<&Shortcut>, checked: bool, disabled: bool) {
        // TODO: checked, disabled
        let text = if let Some(shortcut) = shortcut {
            format!("{}\t{}", text, shortcut)
        } else {
            text.to_string()
        };

        unsafe {
            let mut flags = MF_STRING;
            if checked {
                flags |= MF_CHECKED;
            }
            if disabled {
                flags |= MF_DISABLED;
            }
            // SAFETY: TODO
            AppendMenuW(self.hmenu, flags, id, PCWSTR(text.to_wide().as_ptr()));
        };
    }

    pub fn add_submenu(&mut self, text: &str, submenu: Menu) {
        let sub_hmenu = submenu.into_hmenu();
        unsafe {
            // SAFETY: TODO
            AppendMenuW(
                self.hmenu,
                MF_POPUP,
                sub_hmenu.0 as usize,
                PCWSTR(text.to_wide().as_ptr()),
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

impl Default for Menu {
    fn default() -> Self {
        Self::new()
    }
}
