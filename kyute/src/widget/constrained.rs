use crate::widget::prelude::*;

pub struct ConstrainedBox<W> {
    constraints: BoxConstraints,
    inner: W,
}

impl<W> ConstrainedBox<W> {
    pub fn new(constraints: BoxConstraints, inner: W) -> ConstrainedBox<W> {
        ConstrainedBox { constraints, inner }
    }

    /// Returns a reference to the inner widget.
    pub fn inner(&self) -> &W {
        &self.inner
    }

    /// Returns a mutable reference to the inner widget.
    pub fn inner_mut(&mut self) -> &mut W {
        &mut self.inner
    }
}

impl<W: Widget> Widget for ConstrainedBox<W> {
    fn widget_id(&self) -> Option<WidgetId> {
        self.inner.widget_id()
    }

    fn event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        self.inner.event(ctx, event, env);
    }

    fn layout(&self, ctx: &mut LayoutCtx, constraints: BoxConstraints, env: &Environment) -> Measurements {
        let constraints = constraints.enforce(self.constraints);
        self.inner.layout(ctx, constraints, env).constrain(constraints)
    }

    fn paint(&self, ctx: &mut PaintCtx, env: &Environment) {
        self.inner.paint(ctx, env)
    }
}
