use crate::{
    composable, layout::BoxConstraints, widget::LayoutWrapper, Environment, Event, EventCtx,
    LayoutCtx, Measurements, Offset, PaintCtx, Rect, SideOffsets, Widget, WidgetPod,
};

/// A widgets that insets its content by a specified padding.
pub struct Padding<W> {
    padding: SideOffsets,
    inner: LayoutWrapper<W>,
}

impl<W: Widget + 'static> Padding<W> {
    /// Creates a new widget with the specified padding.
    #[composable(uncached)]
    pub fn new(padding: SideOffsets, inner: W) -> Padding<W> {
        Padding {
            padding,
            inner: LayoutWrapper::new(inner),
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
            .set_offset(Offset::new(self.padding.left, self.padding.top));
        m
    }

    fn paint(&self, ctx: &mut PaintCtx, bounds: Rect, env: &Environment) {
        self.inner.paint(ctx, bounds, env)
    }
}
