use crate::layout::{BoxConstraints, Layout, Offset, Size};
use crate::renderer::Theme;
use crate::visual::reconciliation::NodePlace;
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
    fn layout<'a>(
        self,
        ctx: &mut LayoutCtx<A>,
        place: &'a mut dyn NodePlace,
        constraints: &BoxConstraints,
        theme: &Theme,
    ) -> &'a mut Node {
        let node: &mut Node<LayoutBox> = place.get_or_insert_default();
        let child = self
            .inner
            .layout(ctx, &mut node.visual.inner, constraints, theme);

        let off = self.baseline - child.layout.baseline.unwrap_or(child.layout.size.height);
        let height = child.layout.size.height + off;
        child.layout.offset.y = off;

        let width = child.layout.size.width;
        node.layout.offset = Offset::zero();
        node.layout.size = constraints.constrain(Size::new(width, height));
        node.layout.baseline = Some(self.baseline);
        node
    }
}
