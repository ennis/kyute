use crate::{
    align_boxes, composable, core2::WindowPaintCtx, layout::BoxConstraints, widget::LayoutWrapper,
    Alignment, Environment, Event, EventCtx, GpuFrameCtx, LayoutCtx, Measurements, Offset,
    PaintCtx, Rect, SideOffsets, Widget, WidgetPod,
};
use kyute_shell::drawing::ToSkia;
use std::cell::Cell;

/// A widgets that insets its content by a specified padding.
pub struct Padding<W> {
    padding: SideOffsets,
    inner: WidgetPod<W>,
}

impl<W: Widget + 'static> Padding<W> {
    /// Creates a new widget with the specified padding.
    #[composable(uncached)]
    pub fn new(padding: SideOffsets, inner: W) -> Padding<W> {
        Padding {
            padding,
            inner: WidgetPod::new(inner),
        }
    }
}

impl<W: Widget> Widget for Padding<W> {
    fn event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        self.inner.event(ctx, event, env);
    }

    fn layout(
        &self,
        ctx: &mut LayoutCtx,
        constraints: BoxConstraints,
        env: &Environment,
    ) -> Measurements {
        let mut m = self
            .inner
            .layout(ctx, constraints.deflate(self.padding), env);
        m.bounds = m.bounds.outer_rect(self.padding);
        self.inner
            .set_child_offset(Offset::new(self.padding.left, self.padding.top));
        m
    }

    fn paint(&self, ctx: &mut PaintCtx, bounds: Rect, env: &Environment) {
        self.inner.paint(ctx, bounds, env)
    }
}
