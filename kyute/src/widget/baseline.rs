use crate::layout::{BoxConstraints, Layout, Offset, Size};
use crate::renderer::Theme;
use crate::visual::{LayoutBox, NodeArena, NodeCursor, NodeData};
use crate::widget::{LayoutCtx, Widget};
use generational_indextree::NodeId;

/// A widget that aligns its child according to a fixed baseline.
pub struct Baseline<W> {
    inner: W,
    baseline: f64,
}

impl<W> Baseline<W> {
    pub fn new(baseline: f64, inner: W) -> Baseline<W> {
        Baseline { inner, baseline }
    }
}

impl<A: 'static, W: Widget<A>> Widget<A> for Baseline<W> {
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
            .layout_child(ctx, nodes, node_id, constraints, theme);

        let mut child_layout = nodes[child_id].get().layout;
        let off = self.baseline - child_layout.baseline.unwrap_or(child_layout.size.height);
        let height = child_layout.size.height + off;
        child_layout.offset.y = off;

        let width = child_layout.size.width;
        let node_layout = Layout::new(constraints.constrain((width, height).into()))
            .with_baseline(Some(self.baseline));

        nodes[child_id].get_mut().layout = child_layout;
        nodes[node_id].get_mut().layout = node_layout;
        node_id
    }
}
