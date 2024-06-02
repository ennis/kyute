use crate::{
    core::{WeakWidget, WeakWidgetPtr},
    BoxConstraints, Ctx, Environment, Event, Geometry, HitTestResult, LayoutCtx, PaintCtx, Widget, WidgetCtx,
    WidgetPod, WidgetPtr, WidgetPtrAny,
};
use kurbo::{Insets, Point, Size, Vec2};

pub struct Padding<T> {
    weak: WeakWidgetPtr<Self>,
    pub padding: Insets,
    pub content: WidgetPtr<T>,
}

impl<T: Widget> Padding<T> {
    pub fn new(padding: Insets, content: WidgetPtr<T>) -> WidgetPtr<Self> {
        WidgetPod::new_cyclic(move |weak| Padding { weak, padding, content })
    }

    fn offset(&self) -> Vec2 {
        Vec2::new(self.padding.x0, self.padding.y0)
    }
}

impl<T: Widget> WeakWidget for Padding<T> {
    fn weak_self(&self) -> WeakWidgetPtr<Self> {
        self.weak.clone()
    }
}

impl<T: Widget> Widget for Padding<T> {
    fn mount(&mut self, cx: &mut Ctx) {
        self.content.as_dyn().mount(cx)
    }

    /*fn natural_width(&mut self, height: f64) -> f64 {
        self.content.natural_width((height - self.padding.y_value()).max(0.0))
    }

    fn natural_height(&mut self, width: f64) -> f64 {
        self.content.natural_height((width - self.padding.x_value()).max(0.0))
    }

    fn natural_baseline(&mut self, params: &BoxConstraints) -> f64 {
        self.content.natural_baseline(&params.deflate(self.padding)) + self.padding.y0
    }*/

    fn hit_test(&mut self, ctx: &mut HitTestResult, position: Point) -> bool {
        // FIXME: do we need to hit-test the blank space?
        // It's unclear what we should do here since it's a wrapper widget
        ctx.test_with_offset(self.offset(), position, |result, position| {
            self.content.as_dyn().hit_test(result, position)
        })
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, constraints: &BoxConstraints) -> Geometry {
        let child_geometry = self.content.layout(ctx, &constraints.deflate(self.padding));
        let offset = self.offset();
        let size = Size {
            width: child_geometry.size.width + self.padding.x_value(),
            height: child_geometry.size.height + self.padding.y_value(),
        };

        Geometry {
            size,
            baseline: child_geometry.baseline.map(|baseline| baseline + offset.y),
            bounding_rect: child_geometry.bounding_rect + offset,
            paint_bounding_rect: child_geometry.paint_bounding_rect + offset,
        }
    }

    fn paint(&mut self, ctx: &mut PaintCtx) {
        ctx.with_offset(self.offset(), |ctx| self.content.paint(ctx));
    }
}
