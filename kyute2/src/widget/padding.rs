use std::any::Any;

use crate::widget::prelude::*;
use kurbo::{Affine, Insets, Point, Size, Vec2};

pub struct Padding<E> {
    pub padding: Insets,
    pub content: E,
}

impl<W> Padding<W> {
    pub fn new(padding: Insets, content: W) -> Self {
        Self { padding, content }
    }
}

impl<E: Widget> Widget for Padding<E> {
    fn update(&self, cx: &mut TreeCtx) {
        self.content.update(cx)
    }

    fn event(&self, ctx: &mut TreeCtx, event: &mut Event) {
        let offset = Vec2::new(self.padding.x0, self.padding.y0);
        event.with_transform(&Affine::translate(offset), |event| self.content.event(ctx, event))
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

    fn hit_test(&self, ctx: &mut HitTestResult, position: Point) -> bool {
        let offset = Vec2::new(self.padding.x0, self.padding.y0);
        let local_position = position - offset;
        self.content.hit_test(ctx, local_position)
    }

    fn layout(&self, ctx: &mut LayoutCtx, constraints: &BoxConstraints) -> Geometry {
        let child_geometry = self.content.layout(ctx, &constraints.deflate(self.padding));
        let offset = Vec2::new(self.padding.x0, self.padding.y0);
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

    fn paint(&self, ctx: &mut PaintCtx) {
        ctx.with_transform(&Affine::translate(Vec2::new(self.padding.x0, self.padding.y0)), |ctx| {
            self.content.paint(ctx)
        });
    }
}
