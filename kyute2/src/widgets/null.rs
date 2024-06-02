use kurbo::Point;

use crate::{
    core::{WeakWidget, WeakWidgetPtr},
    BoxConstraints, Ctx, Event, Geometry, HitTestResult, LayoutCtx, PaintCtx, Widget, WidgetCtx, WidgetPod, WidgetPtr,
};

/// A widget that does nothing.
#[derive(Clone, Default)]
pub struct Null {
    weak: WeakWidgetPtr<Self>,
}

impl Null {
    /// Creates a new null widget.
    pub fn new() -> WidgetPtr<Null> {
        WidgetPod::new_cyclic(|weak| Null { weak })
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

impl WeakWidget for Null {
    fn weak_self(&self) -> WeakWidgetPtr<Self> {
        self.weak.clone()
    }
}

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
