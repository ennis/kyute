use crate::layout::BoxConstraints;
use crate::renderer::Theme;
use crate::visual::reconciliation::NodePlace;
use crate::visual::Node;
use crate::widget::LayoutCtx;
use crate::Widget;
use std::hash::Hash;

/// Identifies a widget.
pub struct Id<W> {
    inner: W,
}

impl<W> Id<W> {
    pub fn new(_id: impl Hash, inner: W) -> Id<W> {
        Id { inner }
    }
}

impl<A: 'static, W: Widget<A>> Widget<A> for Id<W> {
    fn layout<'a>(
        self,
        ctx: &mut LayoutCtx<A>,
        place: &'a mut dyn NodePlace,
        constraints: &BoxConstraints,
        theme: &Theme,
    ) -> &'a mut Node {
        // TODO ID?
        self.inner.layout(ctx, place, constraints, theme)
    }
}
