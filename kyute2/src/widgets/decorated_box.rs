//! Frame decorations
use kurbo::{Point, Size};

use crate::{
    drawing::Decoration, environment::Environment, widgets::Padding, BoxConstraints, Ctx, Event, Geometry,
    HitTestResult, LayoutCtx, PaintCtx, Widget, WidgetPod, WidgetPtr,
};

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct DecoratedBox<D> {
    decoration: D,
    size: Size,
    content: WidgetPtr,
}

impl<D: Decoration + 'static> DecoratedBox<D> {
    pub fn new(decoration: D, content: WidgetPtr) -> WidgetPtr<Self> {
        let padding = decoration.insets();
        WidgetPod::new_cyclic(|weak| DecoratedBox {
            decoration,
            size: Default::default(),
            content: Padding::new(padding, content.with_parent(weak)),
        })
    }
}

impl<D> Widget for DecoratedBox<D>
where
    D: Decoration + 'static,
{
    fn mount(&mut self, cx: &mut Ctx) {
        self.content.mount(cx);
    }

    /*fn natural_width(&mut self, height: f64) -> f64 {
        self.content.natural_width(height)
    }

    fn natural_height(&mut self, width: f64) -> f64 {
        self.content.natural_height(width)
    }

    fn natural_baseline(&mut self, params: &BoxConstraints) -> f64 {
        self.content.natural_baseline(params)
    }*/

    fn hit_test(&mut self, ctx: &mut HitTestResult, position: Point) -> bool {
        self.content.hit_test(ctx, position) || self.size.to_rect().contains(position)
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, constraints: &BoxConstraints) -> Geometry {
        let mut geometry = self.content.layout(ctx, constraints);
        // assume that the decoration expands the paint bounds
        geometry.bounding_rect = geometry.bounding_rect.union(geometry.size.to_rect());
        geometry.paint_bounding_rect = geometry.paint_bounding_rect.union(geometry.size.to_rect());
        self.size = geometry.size;
        geometry
    }

    fn paint(&mut self, ctx: &mut PaintCtx) {
        ctx.with_canvas(|canvas| {
            self.decoration.paint(canvas, self.size.to_rect());
        });
        self.content.paint(ctx);
    }
}
