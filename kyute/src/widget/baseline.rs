use crate::layout::{BoxConstraints, Layout, Offset, Size};
use crate::renderer::{Renderer};
use crate::visual::{Node, Cursor, LayoutBox};
use crate::widget::{Widget, LayoutCtx};

/// .
pub struct Baseline<W> {
    inner: W,
    baseline: f64,
}

impl<W> Baseline<W> {
    pub fn new(baseline: f64, inner: W) -> Baseline<W> {
        Baseline { inner, baseline }
    }
}

impl<A: 'static, W: Widget<A>> Widget<A> for Baseline<W>
{
    fn layout(self, ctx: &mut LayoutCtx<A>, tree_cursor: &mut Cursor, constraints: &BoxConstraints)
    {
        let mut node = &mut *tree_cursor.open(None, || LayoutBox);

        self.inner.layout(ctx, &mut node.cursor(), constraints);

        let mut child = node.children.first_mut().unwrap().borrow_mut();
        let off = self.baseline - child.layout.baseline.unwrap_or(child.layout.size.height);
        let height = child.layout.size.height + off;
        child.layout.offset.y = off;

        let width = child.layout.size.width;
        node.layout.offset = Offset::new(0.0,0.0);
        node.layout.size = constraints.constrain(Size::new(width, height));
        node.layout.baseline = Some(self.baseline)
    }
}
