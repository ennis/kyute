use crate::{
    BoxConstraints, ChangeFlags, Event, Geometry, HitTestResult, LayoutCtx, PaintCtx, TreeCtx, Widget, WidgetId,
};
use kurbo::Point;

/// A widget that does nothing.
#[derive(Copy, Clone, Default)]
pub struct Null;

////////////////////////////////////////////////////////////////////////////////////////////////////

impl Widget for Null {
    fn id(&self) -> WidgetId {
        WidgetId::ANONYMOUS
    }

    fn update(&mut self, _cx: &mut TreeCtx) -> ChangeFlags {
        ChangeFlags::empty()
    }

    fn event(&mut self, _cx: &mut TreeCtx, _event: &mut Event) -> ChangeFlags {
        ChangeFlags::empty()
    }

    fn hit_test(&self, _result: &mut HitTestResult, _position: Point) -> bool {
        false
    }

    fn layout(&mut self, _cx: &mut LayoutCtx, _bc: &BoxConstraints) -> Geometry {
        Geometry::ZERO
    }

    fn paint(&mut self, _cx: &mut PaintCtx) {}
}
