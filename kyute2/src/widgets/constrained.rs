use crate::{
    environment::Environment, BoxConstraints, Event, Geometry, HitTestResult, LayoutCtx, PaintCtx, Widget, WidgetCtx,
};
use kurbo::Point;

pub struct Constrained<T> {
    pub constraints: BoxConstraints,
    pub content: T,
}

impl<T: Widget> Constrained<T> {
    pub fn new(constraints: BoxConstraints, content: T) -> Self {
        Self { constraints, content }
    }
}

impl<T: Widget> Widget for Constrained<T> {
    fn mount(&mut self, cx: &mut WidgetCtx) {
        todo!()
    }

    fn update(&mut self, cx: &mut WidgetCtx) {
        self.content.update(cx)
    }

    fn environment(&self) -> Environment {
        self.content.environment()
    }

    fn event(&mut self, ctx: &mut WidgetCtx, event: &mut Event) {
        self.content.event(ctx, event)
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
