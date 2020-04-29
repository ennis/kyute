use crate::layout::BoxConstraints;
use crate::renderer::Theme;
use crate::visual::NodeData;
use crate::visual::{NodeArena, NodeCursor};
use crate::widget::{LayoutCtx, Widget};
use generational_indextree::NodeId;

/// Expands the child widget to fill all its available space.
pub struct Expand<W>(pub W);

impl<A: 'static, W> Widget<A> for Expand<W>
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
        self.0.layout(
            ctx,
            nodes,
            cursor,
            &BoxConstraints::tight(constraints.biggest()),
            theme,
        )
    }
}
