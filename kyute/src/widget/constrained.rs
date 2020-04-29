use crate::layout::BoxConstraints;
use crate::renderer::Theme;
use crate::visual::{NodeArena, NodeCursor, NodeData};
use crate::widget::{LayoutCtx, Widget};
use generational_indextree::NodeId;

/// Expands the child widget to fill all its available space.
pub struct ConstrainedBox<W> {
    constraints: BoxConstraints,
    inner: W,
}

impl<W> ConstrainedBox<W> {
    pub fn new(constraints: BoxConstraints, inner: W) -> ConstrainedBox<W> {
        ConstrainedBox { constraints, inner }
    }
}

impl<A: 'static, W> Widget<A> for ConstrainedBox<W>
where
    W: Widget<A>,
{
    fn layout(
        self,
        ctx: &mut LayoutCtx<A>,
        nodes: &mut NodeArena,
        cursor: &mut NodeCursor,
        constraints: &BoxConstraints,
        theme: &Theme,
    ) -> NodeId {
        let constraints = constraints.enforce(&self.constraints);
        self.inner.layout(ctx, nodes, cursor, &constraints, theme)
    }
}
