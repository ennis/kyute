use crate::{
    BoxConstraints, ChangeFlags, Event, Geometry, HitTestResult, LayoutCtx, PaintCtx, TreeCtx, Widget, WidgetId,
};
use kurbo::Point;

/// A widget that does nothing.
#[derive(Copy, Clone, Default)]
pub struct Null;

////////////////////////////////////////////////////////////////////////////////////////////////////

impl Widget for Null {
    fn update(&mut self, _cx: &mut TreeCtx) {}

    fn event(&mut self, _cx: &mut TreeCtx, _event: &mut Event) {}

    fn hit_test(&mut self, _result: &mut HitTestResult, _position: Point) -> bool {
        false
    }

    fn layout(&mut self, _cx: &mut LayoutCtx, _bc: &BoxConstraints) -> Geometry {
        Geometry::ZERO
    }

    fn paint(&mut self, _cx: &mut PaintCtx) {}
}
