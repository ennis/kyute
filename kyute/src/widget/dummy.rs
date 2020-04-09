use crate::event::{Event, EventCtx};
use crate::layout::{BoxConstraints, Layout, PaintLayout, Point, Size};
use crate::renderer::Theme;
use crate::visual::{Node, PaintCtx, Visual};
use crate::widget::{LayoutCtx, Widget};
use crate::Bounds;
use std::any::Any;

/// Dummy widget that does nothing.
pub struct DummyWidget;

impl<A: 'static> Widget<A> for DummyWidget
{
    type Visual = DummyVisual;

    fn layout(
        self,
        ctx: &mut LayoutCtx<A>,
        node: Option<Node<Self::Visual>>,
        constraints: &BoxConstraints,
        theme: &Theme
    ) -> Node<DummyVisual>
    {
        Node::new(Layout::default(), None, DummyVisual)
    }
}

pub struct DummyVisual;

impl Visual for DummyVisual {
    fn paint(&mut self, _ctx: &mut PaintCtx, _theme: &Theme) {}

    fn hit_test(&mut self, _point: Point, _bounds: Bounds) -> bool {
        false
    }

    fn event(&mut self, _event_ctx: &mut EventCtx, _event: &Event) {}

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
