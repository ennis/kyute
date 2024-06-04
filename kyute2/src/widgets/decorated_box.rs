//! Frame decorations
use kurbo::{Point, Size};

use crate::{
    drawing::Decoration, environment::Environment, widgets::Padding, BoxConstraints, Ctx, Event, Geometry,
    HitTestResult, LayoutCtx, PaintCtx, Widget,
};

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct DecoratedBox<D, W> {
    decoration: D,
    size: Size,
    content: Padding<W>,
}

impl<D: Decoration, W> DecoratedBox<D, W> {
    pub fn new(decoration: D, content: W) -> Self {
        let padding = decoration.insets();
        DecoratedBox {
            decoration,
            size: Default::default(),
            content: Padding::new(padding, content),
        }
    }
}

impl<D, W> Widget for DecoratedBox<D, W>
where
    D: Decoration + 'static,
    W: Widget,
{
    fn mount(&mut self, cx: &mut Ctx) {
        self.content.mount(cx);
    }

    fn update(&mut self, cx: &mut Ctx) {
        self.content.update(cx);
    }

    fn environment(&self) -> Environment {
        self.content.environment()
    }

    fn event(&mut self, cx: &mut Ctx, event: &mut Event) {
        self.content.event(cx, event);
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
