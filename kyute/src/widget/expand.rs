use crate::layout::BoxConstraints;
use crate::renderer::Theme;
use crate::visual::{Cursor, Node};
use crate::widget::{LayoutCtx, Widget};

/// Expands the child widget to fill all its available space.
pub struct Expand<W>(pub W);

impl<A: 'static, W> Widget<A> for Expand<W>
where
    W: Widget<A>,
{
    fn layout(
        self,
        ctx: &mut LayoutCtx<A>,
        tree_cursor: &mut Cursor,
        constraints: &BoxConstraints,
        theme: &Theme,
    ) {
        self.0.layout(
            ctx,
            tree_cursor,
            &BoxConstraints::tight(constraints.biggest()),
            theme,
        )
    }
}
