use crate::layout::BoxConstraints;
use crate::renderer::Theme;
use crate::visual::{LayoutBox, NodeArena, NodeCursor, NodeData};
use crate::widget::{LayoutCtx, Widget};
use crate::{Alignment, Layout};
use generational_indextree::NodeId;

pub struct Align<W> {
    alignment: Alignment,
    inner: W,
}

impl<W> Align<W> {
    pub fn new(alignment: Alignment, inner: W) -> Align<W> {
        Align { alignment, inner }
    }
}

impl<A: 'static, W> Widget<A> for Align<W>
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
        let node_id = cursor.get_or_insert_default::<LayoutBox>(nodes);
        let child_id = self
            .inner
            .layout_child(ctx, nodes, node_id, &constraints.loosen(), theme);

        let mut node_layout = nodes[node_id].get().layout;
        let mut child_layout = nodes[child_id].get().layout;

        node_layout.size = constraints.constrain(child_layout.size);
        dbg!(node_layout.size);

        Layout::align(&mut node_layout, &mut child_layout, self.alignment);

        nodes[node_id].get_mut().layout = node_layout;
        nodes[child_id].get_mut().layout = child_layout;
        node_id
    }
}
