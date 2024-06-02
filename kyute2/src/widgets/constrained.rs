use crate::{
    core::{WeakWidget, WeakWidgetPtr},
    BoxConstraints, Ctx, Geometry, HitTestResult, LayoutCtx, PaintCtx, Widget, WidgetCtx, WidgetPod, WidgetPtr,
    WidgetPtrAny,
};
use kurbo::Point;

pub struct Constrained {
    weak: WeakWidgetPtr<Self>,
    pub constraints: BoxConstraints,
    pub content: WidgetPtr,
}

impl Constrained {
    pub fn new(constraints: BoxConstraints, content: WidgetPtr<impl Widget>) -> WidgetPtr<Self> {
        WidgetPod::new_cyclic(move |weak| Constrained {
            weak,
            constraints,
            content,
        })
    }
}

impl WeakWidget for Constrained {
    fn weak_self(&self) -> WeakWidgetPtr<Self> {
        self.weak.clone()
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

    fn paint(&mut self, ctx: &mut PaintCtx) {
        self.content.paint(ctx)
    }
}
