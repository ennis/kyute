use crate::{
    drawing::{Paint, ToSkia},
    widget::prelude::*,
};
use std::any::Any;
use tracing::trace;

pub struct Background {
    paint: Paint,
    // computed
    size: Size,
    // TODO: round corners & borders
}

impl Background {
    pub fn new(paint: Paint) -> Background {
        Background {
            paint,
            size: Size::ZERO,
        }
    }
}

impl Widget for Background {
    type Element = Background;

    fn build(self, cx: &mut TreeCtx, element_id: ElementId) -> Self::Element {
        self
    }

    fn update(self, cx: &mut TreeCtx, element: &mut Self::Element) -> ChangeFlags {
        element.paint = self.paint;
        // TODO: compare paints
        ChangeFlags::empty()
        //ChangeFlags::PAINT
    }
}

impl Element for Background {
    fn id(&self) -> ElementId {
        ElementId::ANONYMOUS
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, constraints: &BoxConstraints) -> Geometry {
        self.size = constraints.max;
        Geometry::new(constraints.max)
    }

    fn event(&mut self, ctx: &mut EventCtx, event: &mut Event) -> ChangeFlags {
        ChangeFlags::empty()
    }

    fn natural_width(&mut self, height: f64) -> f64 {
        self.size.width
    }

    fn natural_height(&mut self, width: f64) -> f64 {
        self.size.height
    }

    fn natural_baseline(&mut self, params: &BoxConstraints) -> f64 {
        0.0
    }

    fn hit_test(&self, ctx: &mut HitTestResult, position: Point) -> bool {
        let hit = self.size.to_rect().contains(position);
        trace!("background hit test: {}", hit);
        hit
    }

    fn paint(&mut self, ctx: &mut PaintCtx) {
        let mut surface = ctx.surface.surface();
        let canvas = surface.canvas();
        let bounds = Rect::from_origin_size(Point::ZERO, self.size);
        canvas.draw_rect(bounds.to_skia(), &self.paint.to_sk_paint(bounds));
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn debug(&self, visitor: &mut DebugWriter) {
        visitor.type_name("Background");
        visitor.property("size", self.size);
    }
}
