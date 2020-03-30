use crate::layout::{PaintLayout, BoxConstraints, Point, Layout, Size};
use crate::renderer::{Painter, Renderer};
use crate::visual::{Node, Visual, Cursor, PaintCtx};
use crate::event::{Event, EventCtx};
use crate::widget::{Widget, LayoutCtx};
use std::any::Any;

/// Dummy widget that does nothing.
pub struct DummyWidget;

impl<A: 'static> Widget<A> for DummyWidget {
    fn layout(self, ctx: &mut LayoutCtx<A>, tree_cursor: &mut Cursor, constraints: &BoxConstraints) {
        tree_cursor.overwrite(None, Layout::new(constraints.smallest()), DummyVisual);
    }
}

pub struct DummyVisual;

impl Visual for DummyVisual {
    fn paint(&mut self, _ctx: &mut PaintCtx) {}

    fn hit_test(&mut self, _point: Point, _layout: &PaintLayout) -> bool {
        false
    }

    fn event(&mut self, _event_ctx: &EventCtx, _event: &Event) {
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
