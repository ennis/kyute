use crate::{widget::prelude::*, Length};

#[derive(Clone)]
pub struct ConstrainedBox<W> {
    min_width: Length,
    min_height: Length,
    max_width: Length,
    max_height: Length,
    inner: W,
}

impl<W> ConstrainedBox<W> {
    pub fn new(inner: W) -> ConstrainedBox<W> {
        ConstrainedBox {
            min_width: Length::zero(),
            min_height: Length::zero(),
            max_width: Length::Proportional(1.0),
            max_height: Length::Proportional(1.0),
            inner,
        }
    }

    /// Constrain the minimum width of the container.
    pub fn min_width(mut self, width: impl Into<Length>) -> Self {
        self.set_min_width(width);
        self
    }

    /// Constrain the minimum width of the container.
    pub fn set_min_width(&mut self, width: impl Into<Length>) {
        self.min_width = width.into();
    }

    /// Constrain the minimum height of the container.
    pub fn min_height(mut self, height: impl Into<Length>) -> Self {
        self.set_min_height(height);
        self
    }

    /// Constrain the minimum height of the container.
    pub fn set_min_height(&mut self, height: impl Into<Length>) {
        self.min_height = height.into();
    }

    /// Constrain the maximum width of the container.
    pub fn max_width(mut self, width: impl Into<Length>) -> Self {
        self.set_max_width(width);
        self
    }

    /// Constrain the minimum width of the container.
    pub fn set_max_width(&mut self, width: impl Into<Length>) {
        self.max_width = width.into();
    }

    /// Constrain the minimum height of the container.
    pub fn max_height(mut self, height: impl Into<Length>) -> Self {
        self.set_max_height(height);
        self
    }

    /// Constrain the minimum height of the container.
    pub fn set_max_height(&mut self, height: impl Into<Length>) {
        self.max_height = height.into();
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

    fn layer(&self) -> &LayerHandle {
        self.inner.layer()
    }

    fn layout(&self, ctx: &mut LayoutCtx, constraints: BoxConstraints, env: &Environment) -> Measurements {
        let additional_constraints = BoxConstraints::new(
            self.min_width.to_dips(ctx.scale_factor, constraints.max_width())
                ..self.max_width.to_dips(ctx.scale_factor, constraints.max_width()),
            self.min_height.to_dips(ctx.scale_factor, constraints.max_height())
                ..self.max_height.to_dips(ctx.scale_factor, constraints.max_height()),
        );

        let constraints = constraints.enforce(additional_constraints);
        self.inner.layout(ctx, constraints, env)
    }

    fn event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        self.inner.route_event(ctx, event, env);
    }
}
