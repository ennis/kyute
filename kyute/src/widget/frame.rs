//! Frame containers
use crate::{
    widget::{prelude::*, WidgetWrapper},
    LayoutConstraints, LengthOrPercentage,
};
use kyute_common::RoundToPixel;

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
}

impl<W: Widget + 'static> WidgetWrapper for Frame<W> {
    type Inner = WidgetPod<W>;

    fn inner(&self) -> &Self::Inner {
        &self.inner
    }

    fn inner_mut(&mut self) -> &mut Self::Inner {
        &mut self.inner
    }

    fn layout(&self, ctx: &mut LayoutCtx, constraints: &LayoutConstraints, env: &Environment) -> Layout {
        // calculate width and height
        let width = self.width.compute(constraints, constraints.max.width);
        let height = self.height.compute(constraints, constraints.max.height);

        let mut sub = *constraints;
        sub.max.width = constraints.max.width.min(width);
        sub.max.height = constraints.max.height.min(height);
        sub.min.width = constraints.min.width.max(width);
        sub.min.height = constraints.min.height.max(height);

        if ctx.speculative {
            return Layout::new(Size::new(width, height));
        }

        // measure child
        let sublayout = self.inner.layout(ctx, &sub, env);

        // position the content box
        let size = sub.max;
        let content_offset = sublayout.content_box_offset(size).round_to_pixel(ctx.scale_factor);
        self.inner.set_offset(content_offset);
        Layout::new(size)
    }
}
