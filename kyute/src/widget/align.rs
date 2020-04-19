use crate::layout::BoxConstraints;
use crate::renderer::Theme;
use crate::visual::reconciliation::NodePlace;
use crate::visual::{Node, LayoutBox};
use crate::widget::{LayoutCtx, Widget};
use crate::{Alignment, Layout};

pub struct Align<W> {
    alignment: Alignment,
    inner: W,
}

impl<W> Align<W> {
    pub fn new(alignment: Alignment, inner: W) -> Align<W> {
        Align {
            alignment,
            inner
        }
    }
}

impl<A: 'static, W> Widget<A> for Align<W>
    where
        W: Widget<A>,
{
    fn layout<'a>(
        self,
        ctx: &mut LayoutCtx<A>,
        place: &'a mut dyn NodePlace,
        constraints: &BoxConstraints,
        theme: &Theme,
    ) -> &'a mut Node
    {
        let node: &mut Node<LayoutBox> = place.get_or_insert_default();

        let child = self
            .inner
            .layout(ctx, &mut node.visual.inner, &constraints.loosen(), theme);

        node.layout.size = constraints.constrain(child.layout.size);
        dbg!(node.layout.size);
        Layout::align(&mut node.layout, &mut child.layout, self.alignment);
        node
    }
}
