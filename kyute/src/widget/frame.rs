//! Frame containers
use crate::{core::DebugNode, widget::prelude::*, LayerPaintCtx, LayoutParams, LengthOrPercentage};
use kyute_common::RoundToPixel;
use kyute_shell::animation::Layer;

/// A container with a fixed width and height, into which an unique widget is placed.
pub struct Frame<W> {
    width: LengthOrPercentage,
    height: LengthOrPercentage,
    inner: WidgetPod<W>,
}

impl<W: Widget + 'static> Frame<W> {
    pub fn new(width: LengthOrPercentage, height: LengthOrPercentage, inner: W) -> Frame<W> {
        Frame {
            inner: WidgetPod::new(inner),
            width,
            height,
        }
    }

    pub fn inner(&self) -> &WidgetPod<W> {
        &self.inner
    }

    pub fn inner_mut(&mut self) -> &mut WidgetPod<W> {
        &mut self.inner
    }
}

impl<W: Widget + 'static> Widget for Frame<W> {
    fn widget_id(&self) -> Option<WidgetId> {
        self.inner.widget_id()
    }

    fn layout(&self, ctx: &mut LayoutCtx, constraints: &LayoutParams, env: &Environment) -> Geometry {
        // calculate width and height
        let width = self.width.compute(constraints, constraints.max.width, env);
        let height = self.height.compute(constraints, constraints.max.height, env);

        let mut sub = *constraints;
        sub.max.width = constraints.max.width.min(width);
        sub.max.height = constraints.max.height.min(height);
        sub.min.width = constraints.min.width.max(width);
        sub.min.height = constraints.min.height.max(height);

        if ctx.speculative {
            return Geometry::new(Size::new(width, height));
        }

        // measure child
        let sublayout = self.inner.layout(ctx, &sub, env);

        // position the content box
        // TODO baseline
        let size = sub.max;
        let content_offset = sublayout
            .place_into(&Measurements::new(size))
            .round_to_pixel(ctx.scale_factor);
        self.inner.set_offset(content_offset);
        Geometry::new(size)
    }

    fn event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        self.inner.route_event(ctx, event, env)
    }

    fn paint(&self, ctx: &mut PaintCtx) {
        self.inner.paint(ctx)
    }
}
