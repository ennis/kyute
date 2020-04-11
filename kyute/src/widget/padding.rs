use crate::layout::{BoxConstraints, EdgeInsets, Layout, Offset, Size};
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

impl<A: 'static, W: Widget<A>> Widget<A> for Padding<W> {
    type Visual = LayoutBox<W::Visual>;

    fn layout(
        self,
        ctx: &mut LayoutCtx<A>,
        node: Option<Node<Self::Visual>>,
        constraints: &BoxConstraints,
        theme: &Theme,
    ) -> Node<Self::Visual> {
        let Padding { inner, insets } = self;
        let mut child = inner.layout(
            ctx,
            node.map(|node| node.visual.inner),
            &constraints.deflate(&insets),
            theme,
        );
        child.layout.offset = Offset::new(insets.left, insets.top);

        let layout = Layout {
            offset: Offset::zero(),
            size: Size::new(
                child.layout.size.width + insets.left + insets.right,
                child.layout.size.height + insets.top + insets.bottom,
            ),
            baseline: child.layout.baseline.map(|b| b + insets.top),
        };

        Node::new(layout, None, LayoutBox::new(child))
    }
}
