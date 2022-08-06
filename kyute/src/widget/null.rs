use crate::widget::prelude::*;

/// Null widget. Takes no space, ignores all events.
#[derive(Clone, Debug)]
pub struct Null;

impl Widget for Null {
    fn widget_id(&self) -> Option<WidgetId> {
        None
    }

    fn layout(&self, _ctx: &mut LayoutCtx, constraints: &LayoutParams, _env: &Environment) -> BoxLayout {
        BoxLayout::new(constraints.min)
    }

    fn event(&self, _ctx: &mut EventCtx, _event: &mut Event, _env: &Environment) {}

    fn paint(&self, _ctx: &mut PaintCtx) {}
}
