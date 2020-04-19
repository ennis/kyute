use crate::layout::BoxConstraints;
use crate::renderer::Theme;
use crate::visual::reconciliation::NodePlace;
use crate::visual::Node;
use crate::widget::{LayoutCtx, Widget};

/// Expands the child widget to fill all its available space.
pub struct ConstrainedBox<W> {
    constraints: BoxConstraints,
    inner: W,
}

impl<W> ConstrainedBox<W> {
    pub fn new(constraints: BoxConstraints, inner: W) -> ConstrainedBox<W> {
        dbg!(constraints);
        ConstrainedBox {
            constraints,
            inner
        }
    }
}

impl<A: 'static, W> Widget<A> for ConstrainedBox<W>
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
        let constraints = constraints.enforce(&self.constraints);
        self.inner.layout(
            ctx,
            place,
            &constraints,
            theme,
        )
    }
}
