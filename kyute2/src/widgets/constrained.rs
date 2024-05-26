use crate::{
    environment::Environment, BoxConstraints, Ctx, Event, Geometry, HitTestResult, LayoutCtx, PaintCtx, Widget,
    WidgetCtx, WidgetPod, WidgetPtrAny,
};
use kurbo::Point;

pub struct Constrained {
    pub constraints: BoxConstraints,
    pub content: WidgetPtrAny,
}

impl Constrained {
    pub fn new(constraints: BoxConstraints, content: impl Widget) -> Self {
        Self {
            constraints,
            content: WidgetPod::new(content),
        }
    }
}

impl Widget for Constrained {
    fn mount(&mut self, cx: &mut WidgetCtx<Self>) {
        self.content.dyn_mount(cx)
    }

    fn hit_test(&mut self, ctx: &mut HitTestResult, position: Point) -> bool {
        self.content.dyn_hit_test(ctx, position)
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, params: &BoxConstraints) -> Geometry {
        let mut subconstraints = *params;
        subconstraints.min.width = subconstraints.min.width.max(self.constraints.min.width);
        subconstraints.min.height = subconstraints.min.height.max(self.constraints.min.height);
        subconstraints.max.width = subconstraints.max.width.min(self.constraints.max.width);
        subconstraints.max.height = subconstraints.max.height.min(self.constraints.max.height);
        self.content.layout(ctx, &subconstraints)
    }

    fn paint(&mut self, ctx: &mut PaintCtx) {
        self.content.paint(ctx)
    }
}
