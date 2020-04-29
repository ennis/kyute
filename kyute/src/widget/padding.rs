use crate::layout::{BoxConstraints, EdgeInsets, Layout, Offset, Size};
use crate::renderer::Theme;
use crate::visual::{LayoutBox, NodeArena, NodeCursor, NodeData};
use crate::widget::{LayoutCtx, Widget};
use generational_indextree::NodeId;

/// Padding.
pub struct Padding<W> {
    inner: W,
    insets: EdgeInsets,
}

impl<W> Padding<W> {
    pub fn new(insets: EdgeInsets, inner: W) -> Padding<W> {
        Padding { inner, insets }
    }
}

impl<A: 'static, W: Widget<A>> Widget<A> for Padding<W> {
    fn layout<'a>(
        self,
        ctx: &mut LayoutCtx<A>,
        nodes: &mut NodeArena,
        cursor: &mut NodeCursor,
        constraints: &BoxConstraints,
        theme: &Theme,
    ) -> NodeId {
        let Padding { inner, insets } = self;

        let node_id = cursor.get_or_insert_default::<LayoutBox>(nodes);
        let child_id =
            inner.layout_child(ctx, nodes, node_id, &constraints.deflate(&insets), theme);

        let mut child_layout = nodes[child_id].get().layout;
        child_layout.offset = Offset::new(insets.left, insets.top);

        let node_layout = Layout {
            offset: Offset::zero(),
            size: Size::new(
                child_layout.size.width + insets.left + insets.right,
                child_layout.size.height + insets.top + insets.bottom,
            ),
            baseline: child_layout.baseline.map(|b| b + insets.top),
        };

        nodes[node_id].get_mut().layout = node_layout;
        nodes[child_id].get_mut().layout = child_layout;
        node_id
    }
}
