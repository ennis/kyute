use crate::{
    align_boxes, cache, composable,
    core2::{EventCtx, LayoutCtx, PaintCtx},
    event::{PointerButton, PointerEventKind},
    util::Counter,
    widget::Text,
    Alignment, BoxConstraints, Cache, Data, Environment, Event, Key, Measurements, Rect,
    SideOffsets, Size, Widget, WidgetPod,
};
use std::{convert::TryInto, fmt::Display};
use tracing::trace;

// FIXME: use something else than display
#[derive(Clone, Debug, Data)]
struct DropDownChoice<T: Data + Display> {
    value: T,
    item_id: u16,
}

static ITEM_ID_COUNTER: Counter = Counter::new();

#[derive(Clone)]
pub struct DropDown<T: Data + Display> {
    choices: Vec<DropDownChoice<T>>,
    label: WidgetPod<Text>,
    selected_index: usize,
    new_selected_item: Key<Option<(usize, T)>>,
}

impl<T: Data + Display> DropDown<T> {
    /// Creates a new drop down with the specified choices.
    #[composable(uncached)]
    pub fn new(choices: Vec<T>, selected_index: usize) -> WidgetPod<DropDown<T>> {
        let new_selected_item = cache::state(|| None);
        let label = Text::new(format!("{}", choices[selected_index]));

        // create menu IDs for each choice
        let mut choices_with_ids = Vec::new();
        for (i, choice) in choices.into_iter().enumerate() {
            choices_with_ids.push(DropDownChoice {
                value: choice,
                item_id: i.try_into().unwrap(),
            })
        }

        WidgetPod::new(DropDown {
            choices: choices_with_ids,
            selected_index,
            label,
            new_selected_item,
        })
    }

    /// Returns whether TODO.
    #[composable(uncached)]
    pub fn new_selected_item(&self) -> Option<T> {
        if let Some((i, item)) = self.new_selected_item.update(None) {
            Some(item)
        } else {
            None
        }
    }

    fn create_context_menu(&self) -> kyute_shell::Menu {
        let mut menu = kyute_shell::Menu::new_popup();
        for choice in self.choices.iter() {
            menu.add_item(
                &format!("{}", choice.value),
                choice.item_id as usize,
                None,
                false,
                false,
            );
        }
        menu
    }
}

impl<T: Data + Display> Widget for DropDown<T> {
    fn debug_name(&self) -> &str {
        std::any::type_name::<Self>()
    }

    fn event(&self, ctx: &mut EventCtx, event: &mut Event, _env: &Environment) {
        match event {
            Event::Pointer(p) => match p.kind {
                PointerEventKind::PointerDown if p.button == Some(PointerButton::RIGHT) => {
                    // show the context menu
                    ctx.track_popup_menu(self.create_context_menu(), p.window_position):
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
                ctx.set_state(
                    self.new_selected_item,
                    Some((*id, self.choices[*id].value.clone())),
                );
                ctx.set_handled();
            }
            _ => {}
        }
    }

    fn layout(
        &self,
        ctx: &mut LayoutCtx,
        constraints: BoxConstraints,
        env: &Environment,
    ) -> Measurements {
        // TODO
        self.label.layout(ctx, constraints, env)
    }

    fn paint(&self, ctx: &mut PaintCtx, bounds: Rect, env: &Environment) {
        // TODO
        self.label.paint(ctx, bounds, env);
    }
}
