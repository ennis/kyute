use std::any::Any;

use kurbo::{Affine, Insets, Point, Size, Vec2};

use crate::widget::{prelude::*, WidgetVisitor};

pub struct Padding<E> {
    pub padding: Insets,
    //pub size: Size,
    pub content: E,
}

impl<W> Padding<W> {
    pub fn new(padding: Insets, content: W) -> Self {
        Self { padding, content }
    }
}

impl<E: Widget> Widget for Padding<E> {
    fn id(&self) -> WidgetId {
        self.content.id()
    }

    fn visit_child(&mut self, cx: &mut TreeCtx, id: WidgetId, visitor: &mut WidgetVisitor) {
        self.content.visit_child(cx, id, visitor)
    }

    fn update(&mut self, cx: &mut TreeCtx) -> ChangeFlags {
        self.content.update(cx)
    }

    fn event(&mut self, ctx: &mut TreeCtx, event: &mut Event) -> ChangeFlags {
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

    fn layout(&mut self, ctx: &mut LayoutCtx, constraints: &BoxConstraints) -> Geometry {
        let child_geometry = ctx.layout(&mut self.content, &constraints.deflate(self.padding));

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

    /*fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn debug(&self, w: &mut DebugWriter) {
        w.type_name("PaddingElement");
        w.property("padding", self.padding);
        w.property("size", self.size);
        w.child("", &self.content);
    }*/

    fn paint(&mut self, ctx: &mut PaintCtx) {
        ctx.with_transform(&Affine::translate(Vec2::new(self.padding.x0, self.padding.y0)), |ctx| {
            ctx.paint(&mut self.content);
        });
    }
}
