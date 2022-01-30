use crate::{
    align_boxes, core2::WindowPaintCtx, layout::BoxConstraints, widget::LayoutWrapper, Alignment,
    Environment, Event, EventCtx, GpuFrameCtx, LayoutCtx, Measurements, Offset, PaintCtx, Rect,
    Widget,
};
use std::cell::Cell;

pub struct ConstrainedBox<W> {
    constraints: BoxConstraints,
    inner: W,
}

impl<W> ConstrainedBox<W> {
    pub fn new(constraints: BoxConstraints, inner: W) -> ConstrainedBox<W> {
        ConstrainedBox { constraints, inner }
    }
}

impl<W: Widget> Widget for ConstrainedBox<W> {
    fn event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        self.inner.event(ctx, event, env);
    }

    fn layout(
        &self,
        ctx: &mut LayoutCtx,
        constraints: BoxConstraints,
        env: &Environment,
    ) -> Measurements {
        let constraints = constraints.enforce(&self.constraints);
        self.inner.layout(ctx, constraints, env)
    }

    fn paint(&self, ctx: &mut PaintCtx, bounds: Rect, env: &Environment) {
        self.inner.paint(ctx, bounds, env)
    }
}
