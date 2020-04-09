use crate::layout::{BoxConstraints, Layout, Offset, Size};
use crate::renderer::Theme;
use crate::visual::{ LayoutBox, Node};
use crate::widget::{LayoutCtx, Widget};

/// .
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
    type Visual = LayoutBox;

    fn layout(
        self,
        ctx: &mut LayoutCtx<A>,
        node: Option<Node<LayoutBox>>,
        constraints: &BoxConstraints,
        theme: &Theme
    ) -> Node<LayoutBox>
    {
        let mut node = node.unwrap_or(Node::new(Layout::default(), None, LayoutBox));

        let mut child = self.inner.layout_single_child(ctx, &mut node.children, constraints, theme);

        let off = self.baseline - child.layout.baseline.unwrap_or(child.layout.size.height);
        let height = child.layout.size.height + off;
        child.layout.offset.y = off;

        let width = child.layout.size.width;
        node.layout.offset = Offset::new(0.0, 0.0);
        node.layout.size = constraints.constrain(Size::new(width, height));
        node.layout.baseline = Some(self.baseline);

        drop(child);
        node
    }
}
