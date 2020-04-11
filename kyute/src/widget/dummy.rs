use crate::event::Event;
use crate::layout::{BoxConstraints, Layout, PaintLayout, Point, Size};
use crate::renderer::Theme;
use crate::visual::{DummyVisual, Node, PaintCtx, Visual};
use crate::widget::{LayoutCtx, Widget};
use crate::Bounds;
use std::any::Any;

/// Dummy widget that does nothing.
pub struct DummyWidget;

impl<A: 'static> Widget<A> for DummyWidget {
    type Visual = DummyVisual;

    fn layout(
        self,
        ctx: &mut LayoutCtx<A>,
        node: Option<Node<Self::Visual>>,
        constraints: &BoxConstraints,
        theme: &Theme,
    ) -> Node<DummyVisual> {
        Node::new(Layout::default(), None, DummyVisual)
    }
}
