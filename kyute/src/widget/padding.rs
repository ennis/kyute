use crate::layout::{BoxConstraints, Layout, Offset, Size, EdgeInsets};
use crate::renderer::Theme;
use crate::visual::{LayoutBox, Node};
use crate::widget::{LayoutCtx, Widget};

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

impl<A: 'static, W: Widget<A>> Widget<A> for Padding<W>
{
    type Visual = LayoutBox;

    fn layout(
        self,
        ctx: &mut LayoutCtx<A>,
        node: Option<Node<LayoutBox>>,
        constraints: &BoxConstraints,
        theme: &Theme
    ) -> Node<LayoutBox>
    {
        let Padding { inner, insets } = self;
        let mut node = node.unwrap_or(Node::new(Layout::default(), None, LayoutBox));
        let mut child = inner.layout_single_child(ctx, &mut node.children, &constraints.deflate(&insets), theme);
        child.layout.offset = Offset::new(insets.left, insets.top);
        node.layout.baseline = child.layout.baseline.map(|b| b + insets.top);
        node.layout.size = Size::new(
            child.layout.size.width + insets.left + insets.right,
            child.layout.size.height + insets.top + insets.bottom);

        drop(child);
        node
    }
}
