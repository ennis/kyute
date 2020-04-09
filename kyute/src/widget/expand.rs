use crate::layout::BoxConstraints;
use crate::renderer::Theme;
use crate::visual::Node;
use crate::widget::{LayoutCtx, Widget};

/// Expands the child widget to fill all its available space.
pub struct Expand<W>(pub W);

impl<A: 'static, W> Widget<A> for Expand<W>
where
    W: Widget<A>,
{
    type Visual = W::Visual;

    fn layout(
        self,
        ctx: &mut LayoutCtx<A>,
        node: Option<Node<Self::Visual>>,
        constraints: &BoxConstraints,
        theme: &Theme
    ) -> Node<Self::Visual>
    {
        self.0.layout(
            ctx,
            node,
            &BoxConstraints::tight(constraints.biggest()),
            theme,
        )
    }
}
