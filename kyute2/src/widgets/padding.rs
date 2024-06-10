use std::time::Duration;

use kurbo::{Insets, Point, Size, Vec2};
use winit::event::WindowEvent;

use crate::{
    BoxConstraints, Ctx, Environment, Event, Geometry, HitTestResult, LayoutCtx, PaintCtx, Widget, WidgetPod, WidgetPtr,
};

pub struct Padding {
    pub padding: Insets,
    pub content: WidgetPtr,
}

impl Padding {
    pub fn new(padding: Insets, content: WidgetPtr) -> WidgetPtr<Self> {
        WidgetPod::new_cyclic(|weak| Padding {
            padding,
            content: content.with_parent(weak),
        })
    }

    fn offset(&self) -> Vec2 {
        Vec2::new(self.padding.x0, self.padding.y0)
    }
}

impl Widget for Padding {
    fn mount(&mut self, cx: &mut Ctx) {
        self.content.mount(cx)
    }

    /*fn update(&mut self, cx: &mut Ctx) {
        cx.with_offset(self.offset(), |cx| {
            self.content.update(cx);
        });
    }

    fn environment(&self) -> Environment {
        self.content.environment()
    }

    fn event(&mut self, cx: &mut Ctx, event: &mut Event) {
        event.with_offset(self.offset(), |event| {
            self.content.event(cx, event);
        });
    }*/

    /*fn natural_width(&mut self, height: f64) -> f64 {
        self.content.natural_width((height - self.padding.y_value()).max(0.0))
    }

    fn natural_height(&mut self, width: f64) -> f64 {
        self.content.natural_height((width - self.padding.x_value()).max(0.0))
    }

    fn natural_baseline(&mut self, params: &BoxConstraints) -> f64 {
        self.content.natural_baseline(&params.deflate(self.padding)) + self.padding.y0
    }*/

    fn hit_test(&mut self, result: &mut HitTestResult, position: Point) -> bool {
        self.content.hit_test(result, position)
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, constraints: &BoxConstraints) -> Geometry {
        let child_geometry = self.content.layout(ctx, &constraints.deflate(self.padding));
        let offset = self.offset();
        let size = Size {
            width: child_geometry.size.width + self.padding.x_value(),
            height: child_geometry.size.height + self.padding.y_value(),
        };

        self.content.set_offset(offset);

        Geometry {
            size,
            baseline: child_geometry.baseline.map(|baseline| baseline + offset.y),
            bounding_rect: child_geometry.bounding_rect + offset,
            paint_bounding_rect: child_geometry.paint_bounding_rect + offset,
        }
    }

    fn paint(&mut self, ctx: &mut PaintCtx) {
        self.content.paint(ctx);
    }
}
