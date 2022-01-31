use crate::{
    align_boxes, core2::WindowPaintCtx, layout::BoxConstraints, widget::LayoutWrapper, Alignment,
    Environment, Event, EventCtx, GpuFrameCtx, LayoutCtx, Measurements, Offset, PaintCtx, Rect,
    Widget, WidgetPod, composable
};
use kyute_shell::drawing::ToSkia;
use std::cell::Cell;

pub struct Align<W> {
    alignment: Alignment,
    inner: WidgetPod<W>,
}

impl<W: Widget+'static> Align<W> {
    #[composable(uncached)]
    pub fn new(alignment: Alignment, inner: W) -> Align<W> {
        Align {
            alignment,
            inner: WidgetPod::new(inner)
        }
    }
}

impl<W: Widget> Widget for Align<W> {
    fn event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        self.inner.event(ctx, event, env);
    }

    fn layout(
        &self,
        ctx: &mut LayoutCtx,
        constraints: BoxConstraints,
        env: &Environment,
    ) -> Measurements {
        let child_measurements = self.inner.layout(ctx, constraints.loosen(), env);
        let mut m = Measurements::new(constraints.constrain(child_measurements.size()).into());
        let offset = align_boxes(self.alignment, &mut m, child_measurements);
        self.inner.set_child_offset(offset);
        m
    }

    fn paint(&self, ctx: &mut PaintCtx, bounds: Rect, env: &Environment) {
        self.inner.paint(ctx, bounds, env)
    }
}
