use crate::{backend, Shortcut};
use std::mem;

pub struct Menu(backend::Menu);

impl Menu {
    /// Creates a new menu.
    pub fn new() -> Menu {
        Menu(backend::Menu::new())
    }

    /// Creates a new menu.
    pub fn new_popup() -> Menu {
        Menu(backend::Menu::new_popup())
    }

    pub(crate) fn into_inner(self) -> backend::Menu {
        self.0
    }

    pub fn add_item(&mut self, text: &str, id: usize, shortcut: Option<&Shortcut>, checked: bool, disabled: bool) {
        self.0.add_item(text, id, shortcut, checked, disabled)
    }

    pub fn add_submenu(&mut self, text: &str, submenu: Menu) {
        self.0.add_submenu(text, submenu.0)
    }

    pub fn add_separator(&mut self) {
        self.0.add_separator()
    }
}

impl Default for Menu {
    fn default() -> Self {
        Self::new()
    }
}
