//! Selectable wrapper

use crate::widget::{prelude::*, Clickable};
use std::marker::PhantomData;

pub struct Selectable<W, T> {
    inner: Clickable<W>,
    selected: bool,
    _phantom: PhantomData<T>,
}

impl<W, T> Selectable<W, T>
where
    W: Widget + 'static,
    T: PartialEq,
{
    #[composable]
    pub fn new(selection: &mut T, this_item: T, widget: W) -> Selectable<W, T> {
        let inner = Clickable::new(widget);

        // FIXME: we don't give the opportunity to inhibit selection, but whatever
        let selected = if inner.clicked() {
            *selection = this_item;
            true
        } else {
            *selection == this_item
        };

        Selectable {
            inner,
            selected,
            _phantom: PhantomData,
        }
    }

    pub fn is_selected(&self) -> bool {
        self.selected
    }
}

impl<W: Widget + 'static, T> Widget for Selectable<W, T> {
    fn widget_id(&self) -> Option<WidgetId> {
        self.inner.widget_id()
    }

    fn event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        self.inner.event(ctx, event, env)
    }

    fn layout(&self, ctx: &mut LayoutCtx, constraints: BoxConstraints, env: &Environment) -> Measurements {
        self.inner.layout(ctx, constraints, env)
    }

    fn paint(&self, ctx: &mut PaintCtx, env: &Environment) {
        self.inner.paint(ctx, env)
    }
}
