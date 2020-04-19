use crate::layout::BoxConstraints;
use crate::renderer::Theme;
use crate::visual::reconciliation::NodePlace;
use crate::visual::Node;
use crate::widget::{LayoutCtx, Widget};

/// Expands the child widget to fill all its available space.
pub struct Expand<W>(pub W);

impl<A: 'static, W> Widget<A> for Expand<W>
where
    W: Widget<A>,
{
    fn layout<'a>(
        self,
        ctx: &mut LayoutCtx<A>,
        place: &'a mut dyn NodePlace,
        constraints: &BoxConstraints,
        theme: &Theme,
    ) -> &'a mut Node {
        self.0.layout(
            ctx,
            place,
            &BoxConstraints::tight(constraints.biggest()),
            theme,
        )
    }
}
