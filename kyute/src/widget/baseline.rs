use crate::layout::{BoxConstraints, Layout, Offset, Size};
use crate::renderer::Theme;
use crate::visual::{LayoutBox, Node};
use crate::widget::{LayoutCtx, Widget};

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
    type Visual = LayoutBox<W::Visual>;

    fn layout(
        self,
        ctx: &mut LayoutCtx<A>,
        node: Option<Node<Self::Visual>>,
        constraints: &BoxConstraints,
        theme: &Theme,
    ) -> Node<Self::Visual> {
        let mut child =
            self.inner
                .layout(ctx, node.map(|node| node.visual.inner), constraints, theme);

        let off = self.baseline - child.layout.baseline.unwrap_or(child.layout.size.height);
        let height = child.layout.size.height + off;
        child.layout.offset.y = off;

        let width = child.layout.size.width;
        let layout = Layout {
            offset: Offset::new(0.0, 0.0),
            size: constraints.constrain(Size::new(width, height)),
            baseline: Some(self.baseline),
        };

        Node::new(layout, None, LayoutBox::new(child))
    }
}
