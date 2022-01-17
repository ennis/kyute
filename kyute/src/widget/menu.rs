use crate::{cache, composable, util::Counter, Cache, Data, Key};
use std::{collections::HashMap, convert::TryInto};

/// Keyboard shortcut.
// This is a newtype so that we can impl Data on it.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Shortcut(kyute_shell::Shortcut);

impl Shortcut {
    pub fn new(modifiers: keyboard_types::Modifiers, key: keyboard_types::Key) -> Shortcut {
        Shortcut(kyute_shell::Shortcut::new(modifiers, key))
    }

    pub fn modifiers(&self) -> keyboard_types::Modifiers {
        self.0.modifiers
    }

    pub fn key(&self) -> &keyboard_types::Key {
        &self.0.key
    }

    pub fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl Data for Shortcut {
    fn same(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

#[derive(Clone, Debug, Data)]
pub struct Action {
    id: u32,
    pub(crate) shortcut: Option<Shortcut>,
    // ignore "triggered" which is transient state
    #[data(ignore)]
    pub(crate) triggered: bool,
    #[data(ignore)]
    pub(crate) triggered_state: Key<bool>,
}

// FIXME: WM_COMMAND menu ids are 16-bit, so we can exhaust IDs quickly if we keep creating new actions
static ACTION_ID_COUNTER: Counter = Counter::new();

impl Action {
    /// Creates a new action.
    // FIXME does this need to be cached? it's cheap to create
    #[composable]
    pub fn new() -> Action {
        Self::new_inner(None)
    }

    /// Creates a new action with the specified keyboard shortcut.
    // TODO remove, replace with a function that mutates an existing action: `Action::new().shortcut(...)`
    #[composable]
    pub fn with_shortcut(shortcut: Shortcut) -> Action {
        Self::new_inner(Some(shortcut))
    }

    #[composable(uncached)]
    fn new_inner(shortcut: Option<Shortcut>) -> Action {
        let id: u32 = cache::once(|| ACTION_ID_COUNTER.next().try_into().unwrap());
        let triggered_state = cache::state(|| false);
        let triggered = triggered_state.get();
        if triggered {
            triggered_state.set(false);
        }
        Action {
            id,
            triggered,
            shortcut,
            triggered_state,
        }
    }

    /// Returns whether the action was triggered.
    pub fn triggered(&self) -> bool {
        self.triggered
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
                    menu.add_item(
                        text,
                        action.id as usize,
                        action.shortcut.as_ref().map(|s| &s.0),
                        false,
                        false,
                    );
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

    pub(crate) fn build_action_map(&self, actions_by_id: &mut HashMap<u32, Action>) {
        for item in self.items.iter() {
            match item {
                MenuItem::Action { action, .. } => {
                    actions_by_id.insert(action.id, action.clone());
                }
                MenuItem::Submenu { menu, .. } => {
                    menu.build_action_map(actions_by_id);
                }
                MenuItem::Separator => {}
            }
        }
    }
}
