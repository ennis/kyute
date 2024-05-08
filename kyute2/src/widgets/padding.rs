use crate::{BoxConstraints, Environment, Event, Geometry, HitTestResult, LayoutCtx, PaintCtx, TreeCtx, Widget};
use kurbo::{Insets, Point, Size, Vec2};

pub struct Padding<T> {
    pub padding: Insets,
    pub content: T,
}

impl<T: Widget> Padding<T> {
    pub fn new(padding: Insets, content: T) -> Self {
        Self { padding, content }
    }

    fn offset(&self) -> Vec2 {
        Vec2::new(self.padding.x0, self.padding.y0)
    }
}

impl<T: Widget> Widget for Padding<T> {
    fn update(&mut self, cx: &mut TreeCtx) {
        self.content.update(cx)
    }

    fn environment(&self) -> Environment {
        self.content.environment()
    }

    fn event(&mut self, ctx: &mut TreeCtx, event: &mut Event) {
        event.with_offset(self.offset(), |event| self.content.event(ctx, event));
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
        ctx.test_with_offset(self.offset(), position, |result, position| {
            self.content.hit_test(result, position)
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
