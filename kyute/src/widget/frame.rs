//! Frame containers
use crate::{
    widget::{prelude::*, WidgetWrapper},
    LayoutConstraints,
};
use std::cmp::min;

/// A container with a fixed width and height, into which an unique widget is placed.
pub struct Frame<W> {
    width: Length,
    height: Length,
    inner: WidgetPod<W>,
}

impl<W> Frame<W> {
    pub fn new(width: Length, height: Length, inner: W) -> Frame<W> {
        Frame { inner, width, height }
    }
}

impl<W: Widget + 'static> WidgetWrapper for Frame<W> {
    type Inner = W;

    fn inner(&self) -> &Self::Inner {
        &self.inner
    }
    fn inner_mut(&mut self) -> &mut Self::Inner {
        &mut self.inner
    }

    fn layout(&self, ctx: &mut LayoutCtx, constraints: &LayoutConstraints, env: &Environment) -> Layout {
        // calculate width and height
        let width = constraints.resolve_width(self.width);
        let height = constraints.resolve_height(self.height);

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
        let content_offset = sublayout.content_box_offset(size);
        self.inner.set_offset(content_offset);
        Layout::new(size)
    }
}
