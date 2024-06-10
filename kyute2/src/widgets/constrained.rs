use std::time::Duration;

use kurbo::Point;
use winit::event::WindowEvent;

use crate::{
    BoxConstraints, Ctx, Environment, Event, Geometry, HitTestResult, LayoutCtx, PaintCtx, Widget, WidgetPod, WidgetPtr,
};

pub struct Constrained {
    pub constraints: BoxConstraints,
    pub content: WidgetPtr,
}

impl Constrained {
    pub fn new(constraints: BoxConstraints, content: WidgetPtr) -> WidgetPtr<Self> {
        WidgetPod::new_cyclic(|weak| Constrained {
            constraints,
            content: content.with_parent(weak),
        })
    }
}

impl Widget for Constrained {
    fn mount(&mut self, cx: &mut Ctx) {
        self.content.mount(cx)
    }

    fn hit_test(&mut self, ctx: &mut HitTestResult, position: Point) -> bool {
        self.content.hit_test(ctx, position)
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, params: &BoxConstraints) -> Geometry {
        let mut subconstraints = *params;
        subconstraints.min.width = subconstraints.min.width.max(self.constraints.min.width);
        subconstraints.min.height = subconstraints.min.height.max(self.constraints.min.height);
        subconstraints.max.width = subconstraints.max.width.min(self.constraints.max.width);
        subconstraints.max.height = subconstraints.max.height.min(self.constraints.max.height);
        self.content.layout(ctx, &subconstraints)
    }

    /*fn window_event(&mut self, _cx: &mut Ctx, _event: &WindowEvent, _time: Duration) {
        self.content.window_event(_cx, _event, _time)
    }*/

    fn paint(&mut self, ctx: &mut PaintCtx) {
        self.content.paint(ctx)
    }
}
