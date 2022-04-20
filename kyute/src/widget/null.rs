use crate::{core::WindowPaintCtx, widget::prelude::*, GpuFrameCtx};

/// Null widget. Takes no space, ignores all events.
#[derive(Clone, Debug)]
pub struct Null;

impl Widget for Null {
    fn widget_id(&self) -> Option<WidgetId> {
        None
    }

    fn layout(&self, _ctx: &mut LayoutCtx, _constraints: BoxConstraints, _env: &Environment) -> Measurements {
        Measurements::default()
    }

    fn event(&self, _ctx: &mut EventCtx, _event: &mut Event, _env: &Environment) {}

    fn paint(&self, ctx: &mut PaintCtx) {}
}
