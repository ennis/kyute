use kurbo::Point;

use crate::{BoxConstraints, Ctx, Geometry, HitTestResult, LayoutCtx, PaintCtx, Widget, WidgetPod, WidgetPtr};

/// A widget that does nothing.
#[derive(Copy, Clone, Default)]
pub struct Null;

fn null() -> WidgetPtr<Null> {
    WidgetPod::new(Null)
}

////////////////////////////////////////////////////////////////////////////////////////////////////

impl Widget for Null {
    fn mount(&mut self, cx: &mut Ctx) {}

    fn hit_test(&mut self, _result: &mut HitTestResult, _position: Point) -> bool {
        false
    }

    fn layout(&mut self, _cx: &mut LayoutCtx, _bc: &BoxConstraints) -> Geometry {
        Geometry::ZERO
    }

    fn paint(&mut self, _cx: &mut PaintCtx) {}
}
