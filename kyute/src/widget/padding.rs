use crate::layout::{BoxConstraints, EdgeInsets, Layout, Offset, Size};
use crate::renderer::Theme;
use crate::visual::reconciliation::NodePlace;
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
    fn layout<'a>(
        self,
        ctx: &mut LayoutCtx<A>,
        place: &'a mut dyn NodePlace,
        constraints: &BoxConstraints,
        theme: &Theme,
    ) -> &'a mut Node {
        let Padding { inner, insets } = self;

        let node: &mut Node<LayoutBox> = place.get_or_insert_default();
        let child = inner.layout(
            ctx,
            &mut node.visual.inner,
            &constraints.deflate(&insets),
            theme,
        );

        child.layout.offset = Offset::new(insets.left, insets.top);

        node.layout = Layout {
            offset: Offset::zero(),
            size: Size::new(
                child.layout.size.width + insets.left + insets.right,
                child.layout.size.height + insets.top + insets.bottom,
            ),
            baseline: child.layout.baseline.map(|b| b + insets.top),
        };

        node
    }
}
