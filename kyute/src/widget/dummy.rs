use crate::event::{Event, EventCtx};
use crate::layout::{BoxConstraints, Layout, PaintLayout, Point, Size};
use crate::renderer::Theme;
use crate::visual::{Cursor, Node, PaintCtx, Visual};
use crate::widget::{LayoutCtx, Widget};
use crate::Bounds;
use std::any::Any;

/// Dummy widget that does nothing.
pub struct DummyWidget;

impl<A: 'static> Widget<A> for DummyWidget {
    fn layout(
        self,
        ctx: &mut LayoutCtx<A>,
        tree_cursor: &mut Cursor,
        constraints: &BoxConstraints,
        _theme: &Theme,
    ) {
        tree_cursor.overwrite(None, Layout::new(constraints.smallest()), DummyVisual);
    }
}

pub struct DummyVisual;

impl Visual for DummyVisual {
    fn paint(&mut self, _ctx: &mut PaintCtx, _theme: &Theme) {}

    fn hit_test(&mut self, _point: Point, _bounds: Bounds) -> bool {
        false
    }

    fn event(&mut self, _event_ctx: &EventCtx, _event: &Event) {}

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
