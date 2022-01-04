use crate::{composable, util::Counter, Cache, Data, Key};
use std::{convert::TryInto, sync::Arc};

#[derive(Clone, Debug)]
pub struct Action {
    id: u32,
    triggered: (bool, Key<bool>),
    // TODO keyboard shortcut(s)
}

impl Data for Action {
    fn same(&self, other: &Self) -> bool {
        // same actions if same ID
        // ignore "triggered" which is transient state
        self.id == other.id
    }
}

static ACTION_ID_COUNTER: Counter = Counter::new();

impl Action {
    /// Creates a new action.
    #[composable]
    pub fn new() -> Action {
        let id: u32 = Cache::memoize((), || ACTION_ID_COUNTER.next().try_into().unwrap());
        let triggered = Cache::state(|| false);
        if triggered.0 {
            Cache::replace_state(triggered.1, false);
        }
        Action { id, triggered }
    }

    /// Returns whether the action was triggered.
    pub fn triggered(&self) -> bool {
        self.triggered.0
    }
}

#[derive(Clone, Debug, Data)]
pub enum MenuItem {
    Action { text: String, action: Action },
    Separator,
    Submenu { text: String, menu: Menu },
}

impl MenuItem {
    /// Creates a new menu item from an action.
    pub fn new(text: impl Into<String>, action: Action) -> MenuItem {
        MenuItem::Action {
            text: text.into(),
            action,
        }
    }

    /// Creates a new separator item.
    pub fn separator() -> MenuItem {
        MenuItem::Separator
    }

    /// Creates a submenu item.
    pub fn submenu(text: impl Into<String>, submenu: Menu) -> MenuItem {
        MenuItem::Submenu {
            text: text.into(),
            menu: submenu,
        }
    }
}

/// A collection of menu items.
#[derive(Clone, Debug, Data)]
pub struct Menu {
    #[data(same_fn = "compare_menu_items")]
    items: Vec<MenuItem>,
}

// Work around the absence of `Data` for Vec. It's important to have precise change detection
// for menus because we don't want to keep re-creating native window menus too often.
// TODO impl Data for Vec always?
// TODO find a way to intelligently build cached collections
fn compare_menu_items(a: &Vec<MenuItem>, b: &Vec<MenuItem>) -> bool {
    (a.len() == b.len()) && (a.iter().zip(b.iter()).all(|(x, y)| x.same(y)))
}

impl Menu {
    #[composable(uncached)]
    pub fn new(items: Vec<MenuItem>) -> Menu {
        Menu { items }
    }

    pub(crate) fn to_shell_menu(&self) -> kyute_shell::Menu {
        let mut menu = kyute_shell::Menu::new();
        for item in self.items.iter() {
            match item {
                MenuItem::Action { action, text } => {
                    menu.add_item(text, action.id as usize, false, false);
                }
                MenuItem::Separator => {
                    menu.add_separator();
                }
                MenuItem::Submenu {
                    text,
                    menu: submenu,
                } => {
                    menu.add_submenu(text, submenu.to_shell_menu());
                }
            }
        }
        menu
    }
}
