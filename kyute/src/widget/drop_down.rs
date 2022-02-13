use crate::{
    composable,
    core2::{EventCtx, LayoutCtx, PaintCtx},
    event::{PointerButton, PointerEventKind},
    state::Signal,
    style::{BoxStyle, ColorRef, PaintCtxExt, ValueRef},
    theme,
    widget::{Container, Label},
    BoxConstraints, Data, Environment, Event, Measurements, Rect, Widget, WidgetPod,
};
use euclid::SideOffsets2D;
use std::{convert::TryInto, fmt::Display};
use tracing::trace;

/*/// Visual style for a drop-down control
#[derive(Clone, Debug, Default)]
pub struct DropDownStyle {
    pub box_style: ValueRef<BoxStyle>,
    pub label_color: ColorRef,
}*/

#[derive(Clone, Debug, Data)]
struct DropDownChoice<T: Data + Display> {
    value: T,
    item_id: u16,
}

/// Selects one option among choices with a drop-down menu.
#[derive(Clone)]
pub struct DropDown<T: Data + Display> {
    choices: Vec<DropDownChoice<T>>,
    //style: ValueRef<DropDownStyle>,
    selected_index: usize,
    selected_item_changed: Signal<(usize, T)>,
    inner: WidgetPod<Container<Label>>,
}

impl<T: Data + Display> DropDown<T> {
    /// Creates a new drop down with the specified choices.
    #[composable(uncached)]
    pub fn new(choices: Vec<T>, selected_index: usize) -> DropDown<T> {
        // issue: propagating box style to the container?
        // -> user can set style after `new`
        //
        // Solution A:
        // - always use a container with default style keys (`theme::DROP_DOWN_BOX`)
        // - in paint, override `theme::DROP_DOWN_BOX` with the resolved box style.

        let inner = WidgetPod::new(
            Container::new(Label::new(format!("{}", choices[selected_index])))
                .min_height(theme::BUTTON_HEIGHT)
                .baseline(theme::BUTTON_LABEL_BASELINE)
                .content_padding(SideOffsets2D::new_all_same(5.0))
                .box_style(theme::DROP_DOWN),
        );

        // create menu IDs for each choice
        let mut choices_with_ids = Vec::new();
        for (i, choice) in choices.into_iter().enumerate() {
            choices_with_ids.push(DropDownChoice {
                value: choice,
                item_id: i.try_into().unwrap(),
            })
        }

        DropDown {
            choices: choices_with_ids,
            selected_index,
            inner,
            selected_item_changed: Signal::new(),
        }
    }

    /// Returns whether TODO.
    #[composable(uncached)]
    pub fn selected_item_changed(&self) -> Option<T> {
        self.selected_item_changed.value().map(|x| x.1)
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
                self.selected_item_changed
                    .signal(ctx, (*id, self.choices[*id].value.clone()));
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
        self.inner.layout(ctx, constraints, env)
    }

    fn paint(&self, ctx: &mut PaintCtx, bounds: Rect, env: &Environment) {
        //let style = self.style.resolve(env).unwrap();
        //let box_style = style.box_style.resolve(env).unwrap();
        //let label_color = style.label_color.resolve(env).unwrap();
        self.inner.paint(ctx, bounds, env);
    }
}
