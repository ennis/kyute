use crate::widget::prelude::*;

pub struct Constrained {
    pub constraints: BoxConstraints,
    pub content: WidgetPtr,
}

impl Constrained {
    pub fn new(constraints: BoxConstraints, content: impl Widget + 'static) -> Self {
        Self {
            constraints,
            content: WidgetPod::new(content),
        }
    }
}

impl Widget for Constrained {
    fn update(&self, cx: &mut TreeCtx) {
        self.content.update(cx)
    }

    fn event(&self, ctx: &mut TreeCtx, event: &mut Event) {}

    fn hit_test(&self, ctx: &mut HitTestResult, position: Point) -> bool {
        self.content.hit_test(ctx, position)
    }

    fn layout(&self, ctx: &mut LayoutCtx, params: &BoxConstraints) -> Geometry {
        let mut subconstraints = *params;
        subconstraints.min.width = subconstraints.min.width.max(self.constraints.min.width);
        subconstraints.min.height = subconstraints.min.height.max(self.constraints.min.height);
        subconstraints.max.width = subconstraints.max.width.min(self.constraints.max.width);
        subconstraints.max.height = subconstraints.max.height.min(self.constraints.max.height);
        self.content.layout(ctx, &subconstraints)
    }

    fn paint(&self, ctx: &mut PaintCtx) {
        self.content.paint(ctx)
    }
}
