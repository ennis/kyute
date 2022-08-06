use crate::{composable, event::PointerButton, widget::prelude::*, Data, PointerEventKind, WidgetId};
use std::cell::Cell;

pub use kyute_shell::Shortcut;

#[derive(Clone, Debug, Data)]
pub struct Action {
    pub(crate) shortcut: Option<Shortcut>,
    #[data(ignore)]
    pub(crate) index: Cell<usize>,
    // ignore "triggered" which is transient state
    #[data(ignore)]
    pub(crate) triggered: Signal<()>,
}

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

    #[composable]
    fn new_inner(shortcut: Option<Shortcut>) -> Action {
        //let id: u32 = cache::once(|| ACTION_ID_COUNTER.next().try_into().unwrap());
        Action {
            triggered: Signal::new(),
            shortcut,
            index: Cell::new(0),
        }
    }

    /// Returns whether the action was triggered.
    pub fn triggered(&self) -> bool {
        self.triggered.signalled()
    }

    pub fn on_triggered(self, f: impl FnOnce()) -> Self {
        if self.triggered.signalled() {
            f()
        }
        self
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
fn compare_menu_items(a: &[MenuItem], b: &[MenuItem]) -> bool {
    (a.len() == b.len()) && (a.iter().zip(b.iter()).all(|(x, y)| x.same(y)))
}

impl Menu {
    #[composable]
    pub fn new(items: Vec<MenuItem>) -> Menu {
        Menu { items }
    }

    pub(crate) fn to_shell_menu(&self, popup: bool) -> kyute_shell::Menu {
        let mut menu = if popup {
            kyute_shell::Menu::new_popup()
        } else {
            kyute_shell::Menu::new()
        };
        for item in self.items.iter() {
            match item {
                MenuItem::Action { action, text } => {
                    menu.add_item(
                        text,
                        action.index.get() as usize,
                        action.shortcut.as_ref(),
                        false,
                        false,
                    );
                }
                MenuItem::Separator => {
                    menu.add_separator();
                }
                MenuItem::Submenu { text, menu: submenu } => {
                    menu.add_submenu(text, submenu.to_shell_menu(popup));
                }
            }
        }
        menu
    }

    /*pub(crate) fn build_action_map(&self, actions_by_id: &mut HashMap<u32, Action>) {
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
    }*/

    // FIXME: should be done automatically so that nobody forgets to call it.
    pub(crate) fn assign_menu_item_indices(&self) {
        self.assign_menu_item_indices_inner(&mut 0);
    }

    fn assign_menu_item_indices_inner(&self, index: &mut usize) {
        for item in self.items.iter() {
            match item {
                MenuItem::Action { action, .. } => {
                    action.index.set(*index);
                    *index += 1;
                }
                MenuItem::Submenu { menu, .. } => {
                    menu.assign_menu_item_indices_inner(index);
                }
                MenuItem::Separator => {}
            }
        }
    }

    /// Find the action with the given ID.
    pub(crate) fn find_action_by_index(&self, index: usize) -> Option<&Action> {
        for item in self.items.iter() {
            match item {
                MenuItem::Action { action, .. } => {
                    if action.index.get() == index {
                        return Some(action);
                    }
                }
                MenuItem::Submenu { menu, .. } => {
                    if let Some(action) = menu.find_action_by_index(index) {
                        return Some(action);
                    }
                }
                MenuItem::Separator => {}
            }
        }
        None
    }
}

#[derive(Clone)]
pub struct ContextMenu<Content> {
    id: WidgetId,
    menu: Menu,
    content: Content,
}

impl<Content> ContextMenu<Content> {
    #[composable]
    pub fn new(menu: Menu, content: Content) -> ContextMenu<Content> {
        ContextMenu {
            id: WidgetId::here(),
            menu,
            content,
        }
    }
}

impl<Content: Widget + 'static> Widget for ContextMenu<Content> {
    fn widget_id(&self) -> Option<WidgetId> {
        Some(self.id)
    }

    fn layout(&self, ctx: &mut LayoutCtx, constraints: &LayoutParams, env: &Environment) -> BoxLayout {
        self.content.layout(ctx, constraints, env)
    }

    fn event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        self.content.route_event(ctx, event, env);
        if !ctx.handled {
            match *event {
                Event::Pointer(ref pointer_event)
                    if pointer_event.kind == PointerEventKind::PointerDown
                        && pointer_event.button == Some(PointerButton::RIGHT) =>
                {
                    let menu = self.menu.to_shell_menu(true);
                    self.menu.assign_menu_item_indices();
                    ctx.track_popup_menu(menu, pointer_event.window_position);
                }
                Event::MenuCommand(index) => {
                    if let Some(action) = self.menu.find_action_by_index(index) {
                        action.triggered.signal(());
                    }
                }
                _ => {}
            }
        }
    }

    fn paint(&self, ctx: &mut PaintCtx) {
        self.content.paint(ctx)
    }
}
