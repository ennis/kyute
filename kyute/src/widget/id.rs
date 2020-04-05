use crate::layout::BoxConstraints;
use crate::renderer::Theme;
use crate::visual::{Cursor, Node};
use crate::widget::LayoutCtx;
use crate::Widget;
use std::hash::Hash;

/// Identifies a widget.
pub struct Id<W> {
    inner: W,
}

impl<W> Id<W> {
    pub fn new(_id: impl Hash, inner: W) -> Id<W> {
        Id { inner }
    }
}

impl<A: 'static, W: Widget<A>> Widget<A> for Id<W> {
    fn layout(
        self,
        ctx: &mut LayoutCtx<A>,
        tree_cursor: &mut Cursor,
        constraints: &BoxConstraints,
        theme: &Theme,
    ) {
        // TODO ID?
        self.inner.layout(ctx, tree_cursor, constraints, theme)
    }
}
