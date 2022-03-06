use crate::{
    event::{PointerButton, PointerEventKind},
    theme,
    widget::{prelude::*, Container, Label},
    SideOffsets, Signal,
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

/// Selects one option among choices with a drop-down menu.
#[derive(Clone)]
pub struct DropDown<T> {
    id: WidgetId,
    choices: Vec<DropDownChoice<T>>,
    selected_item_changed: Signal<(usize, T)>,
    inner: Container<Label>,
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
        let inner = Container::new(Label::new(formatter.format(&choices[selected_index])))
            .min_height(theme::BUTTON_HEIGHT)
            .baseline(theme::BUTTON_LABEL_BASELINE)
            .content_padding(SideOffsets::new_all_same(5.0))
            .box_style(theme::DROP_DOWN);

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

    fn debug_name(&self) -> &str {
        std::any::type_name::<Self>()
    }

    fn event(&self, ctx: &mut EventCtx, event: &mut Event, _env: &Environment) {
        match event {
            Event::Pointer(p) => match p.kind {
                PointerEventKind::PointerDown if p.button == Some(PointerButton::LEFT) => {
                    // show the context menu
                    trace!("dropdown PointerDown {:?}", p.position);
                    ctx.track_popup_menu(self.create_context_menu(), p.window_position);
                    ctx.request_redraw();
                    ctx.set_handled();
                }
                PointerEventKind::PointerOver => {
                    //trace!("button PointerOver");
                    ctx.request_redraw();
                }
                PointerEventKind::PointerOut => {
                    //trace!("button PointerOut");
                    ctx.request_redraw();
                }
                _ => {}
            },
            Event::MenuCommand(id) => {
                trace!("menu command: {}", *id);
                ctx.cache_mut()
                    .signal(&self.selected_item_changed, (*id, self.choices[*id].value.clone()));
                ctx.set_handled();
            }
            _ => {}
        }
    }

    fn layout(&self, ctx: &mut LayoutCtx, constraints: BoxConstraints, env: &Environment) -> Measurements {
        self.inner.layout(ctx, constraints, env)
    }

    fn paint(&self, ctx: &mut PaintCtx, bounds: Rect, transform: Transform, env: &Environment) {
        //let style = self.style.resolve(env).unwrap();
        //let box_style = style.box_style.resolve(env).unwrap();
        //let label_color = style.label_color.resolve(env).unwrap();
        self.inner.paint(ctx, bounds, transform, env);
    }
}
