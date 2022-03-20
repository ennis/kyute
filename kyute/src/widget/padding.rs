use crate::{
    widget::{prelude::*, LayoutWrapper},
    Length, SideOffsets,
};

/// A widgets that insets its content by a specified padding.
pub struct Padding<W> {
    top: Length,
    right: Length,
    bottom: Length,
    left: Length,
    inner: LayoutWrapper<W>,
}

impl<W: Widget + 'static> Padding<W> {
    /// Creates a new widget with the specified padding.
    #[composable]
    pub fn new(
        top: impl Into<Length>,
        right: impl Into<Length>,
        bottom: impl Into<Length>,
        left: impl Into<Length>,
        inner: W,
    ) -> Padding<W> {
        Padding {
            top: top.into(),
            right: right.into(),
            bottom: bottom.into(),
            left: left.into(),
            inner: LayoutWrapper::new(inner),
        }
    }

    /// Returns a reference to the inner widget.
    pub fn inner(&self) -> &W {
        self.inner.inner()
    }

    /// Returns a mutable reference to the inner widget.
    pub fn inner_mut(&mut self) -> &mut W {
        self.inner.inner_mut()
    }
}

impl<W: Widget> Widget for Padding<W> {
    fn widget_id(&self) -> Option<WidgetId> {
        self.inner.widget_id()
    }

    fn event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        self.inner.event(ctx, event, env);
    }

    fn layout(&self, ctx: &mut LayoutCtx, constraints: BoxConstraints, env: &Environment) -> Measurements {
        let padding = SideOffsets::new(
            self.top.to_dips(ctx.scale_factor, constraints.max.height),
            self.right.to_dips(ctx.scale_factor, constraints.max.width),
            self.bottom.to_dips(ctx.scale_factor, constraints.max.height),
            self.left.to_dips(ctx.scale_factor, constraints.max.width),
        );

        let mut m = self.inner.layout(ctx, constraints.deflate(padding), env);
        m.bounds = m.bounds.outer_rect(padding);
        m.clip_bounds = m.clip_bounds.outer_rect(padding);
        self.inner.set_offset(Offset::new(padding.left, padding.top));
        m
    }

    fn paint(&self, ctx: &mut PaintCtx, env: &Environment) {
        self.inner.paint(ctx, env)
    }
}
