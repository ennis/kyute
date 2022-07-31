use crate::{
    event::{PointerButton, PointerEventKind},
    widget::{prelude::*, Label},
    Signal, UnitExt,
};
use std::{
    convert::TryInto,
    fmt::{Debug, Display},
};

#[derive(Clone, Debug)]
struct DropDownChoice<T> {
    value: T,
    name: String,
    item_id: u16,
}

/// Formatter for drop-down options.
pub trait Formatter<T> {
    fn format(&self, value: &T) -> String;
}

/// Drop-down formatter that uses the Display impl of a type.
pub struct DisplayFormatter;

impl<T: Display> Formatter<T> for DisplayFormatter {
    fn format(&self, value: &T) -> String {
        format!("{}", value)
    }
}

/// Drop-down formatter that uses the Debug impl of a type.
pub struct DebugFormatter;

impl<T: Debug> Formatter<T> for DebugFormatter {
    fn format(&self, value: &T) -> String {
        format!("{:?}", value)
    }
}

type DropDownInner = impl Widget;

/// Selects one option among choices with a drop-down menu.
pub struct DropDown<T> {
    id: WidgetId,
    choices: Vec<DropDownChoice<T>>,
    selected_item_changed: Signal<(usize, T)>,
    inner: DropDownInner,
}

fn drop_down_inner(choice: String) -> DropDownInner {
    let inner = Label::new(choice).min_height(26.dip()).padding(5.dip());
    inner
}

impl<T: Clone + PartialEq + 'static> DropDown<T> {
    #[composable]
    pub fn with_selected(selected: T, choices: Vec<T>, formatter: impl Formatter<T>) -> DropDown<T> {
        let selected_index = choices
            .iter()
            .position(|x| x == &selected)
            .expect("selected value was not in the list of choices");
        DropDown::with_selected_index(selected_index, choices, formatter)
    }
}

impl<T: Clone + 'static> DropDown<T> {
    /// Creates a new drop down with the specified choices.
    #[composable]
    pub fn with_selected_index(selected_index: usize, choices: Vec<T>, formatter: impl Formatter<T>) -> DropDown<T> {
        let inner = drop_down_inner(formatter.format(&choices[selected_index]));

        // create menu IDs for each choice
        let mut choices_with_ids = Vec::new();
        for (i, choice) in choices.into_iter().enumerate() {
            let name = formatter.format(&choice);
            choices_with_ids.push(DropDownChoice {
                value: choice,
                name,
                item_id: i.try_into().unwrap(),
            })
        }

        DropDown {
            id: WidgetId::here(),
            choices: choices_with_ids,
            inner,
            selected_item_changed: Signal::new(),
        }
    }

    /// Returns whether TODO.
    pub fn selected_item_changed(&self) -> Option<T> {
        self.selected_item_changed.value().map(|x| x.1)
    }

    pub fn on_selected_item_changed(self, f: impl FnOnce(T)) -> Self {
        if let Some(item) = self.selected_item_changed() {
            f(item)
        }
        self
    }

    fn create_context_menu(&self) -> kyute_shell::Menu {
        let mut menu = kyute_shell::Menu::new_popup();
        for choice in self.choices.iter() {
            menu.add_item(&choice.name, choice.item_id as usize, None, false, false);
        }
        menu
    }
}

impl<T: Clone + 'static> Widget for DropDown<T> {
    fn widget_id(&self) -> Option<WidgetId> {
        Some(self.id)
    }

    fn layout(&self, ctx: &mut LayoutCtx, constraints: &LayoutParams, env: &Environment) -> Layout {
        self.inner.layout(ctx, constraints, env)
    }

    fn event(&self, ctx: &mut EventCtx, event: &mut Event, _env: &Environment) {
        match event {
            Event::Pointer(p) => match p.kind {
                PointerEventKind::PointerDown if p.button == Some(PointerButton::LEFT) => {
                    // show the context menu
                    trace!("dropdown PointerDown {:?}", p.position);
                    ctx.track_popup_menu(self.create_context_menu(), p.window_position);
                    ctx.set_handled();
                }
                PointerEventKind::PointerOver => {}
                PointerEventKind::PointerOut => {}
                _ => {}
            },
            Event::MenuCommand(id) => {
                trace!("menu command: {}", *id);
                self.selected_item_changed
                    .signal((*id, self.choices[*id].value.clone()));
                ctx.set_handled();
            }
            _ => {}
        }
    }

    fn paint(&self, ctx: &mut PaintCtx) {
        self.inner.paint(ctx)
    }
}
