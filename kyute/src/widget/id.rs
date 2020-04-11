use crate::layout::BoxConstraints;
use crate::renderer::Theme;
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
    type Visual = W::Visual;

    fn layout(
        self,
        ctx: &mut LayoutCtx<A>,
        node: Option<Node<Self::Visual>>,
        constraints: &BoxConstraints,
        theme: &Theme,
    ) -> Node<Self::Visual> {
        // TODO ID?
        self.inner.layout(ctx, node, constraints, theme)
    }
}
