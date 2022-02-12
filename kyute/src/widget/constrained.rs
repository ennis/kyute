use crate::{
    layout::BoxConstraints, Color, Environment, Event, EventCtx, LayoutCtx, Measurements, PaintCtx,
    Rect, Widget,
};

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
        let constraints = constraints.enforce(self.constraints);
        self.inner
            .layout(ctx, constraints, env)
            .constrain(constraints)
    }

    fn paint(&self, ctx: &mut PaintCtx, bounds: Rect, env: &Environment) {
        use kyute::style::*;
        ctx.draw_styled_box(
            bounds,
            &BoxStyle::new().fill(Color::new(0.0, 0.8, 0.0, 0.1)),
            env,
        );
        self.inner.paint(ctx, bounds, env)
    }
}
