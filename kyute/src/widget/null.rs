use crate::widget::prelude::*;

/// Null widget. Takes no space, ignores all events.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Null;

impl Widget for Null {
    fn widget_identity(&self) -> Option<&WidgetIdentity> {
        None
    }

    fn event(&self, _ctx: &mut EventCtx, _event: &mut Event, _env: &Environment) {}

    fn layout(&self, _ctx: &mut LayoutCtx, _constraints: BoxConstraints, _env: &Environment) -> Measurements {
        Measurements::default()
    }

    fn paint(&self, _ctx: &mut PaintCtx, _bounds: Rect, _env: &Environment) {}
}
