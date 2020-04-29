use crate::layout::BoxConstraints;
use crate::renderer::Theme;
use crate::visual::{NodeArena, NodeCursor, NodeData};
use crate::widget::LayoutCtx;
use crate::Widget;
use generational_indextree::NodeId;
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
        nodes: &mut NodeArena,
        cursor: &mut NodeCursor,
        constraints: &BoxConstraints,
        theme: &Theme,
    ) -> NodeId {
        // TODO ID?
        self.inner.layout(ctx, nodes, cursor, constraints, theme)
    }
}
