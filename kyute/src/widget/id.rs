use crate::layout::BoxConstraints;
use crate::renderer::Renderer;
use crate::visual::{Node, Cursor};
use crate::Widget;
use std::hash::Hash;
use crate::widget::LayoutCtx;

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
    fn layout(self, ctx: &mut LayoutCtx<A>, tree_cursor: &mut Cursor, constraints: &BoxConstraints) {
        // TODO ID?
        self.inner.layout(ctx, tree_cursor, constraints)
    }
}
