//! Frame decorations
use kurbo::{Point, Size};

use crate::{
    drawing::Decoration, environment::Environment, widgets::Padding, Binding, BoxConstraints, Event, Geometry,
    HitTestResult, LayoutCtx, PaintCtx, TreeCtx, Widget,
};

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct DecoratedBox<D, T> {
    decoration: D,
    size: Size,
    content: Padding<T>,
}

impl<D: Decoration, T: Widget> DecoratedBox<D, T> {
    pub fn new(decoration: D, content: T) -> Self {
        let padding = decoration.insets();
        Self {
            decoration,
            size: Default::default(),
            content: Padding::new(padding, content),
        }
    }
}

impl<D, T> Widget for DecoratedBox<D, T>
where
    T: Widget,
    D: Decoration + 'static,
{
    fn update(&mut self, cx: &mut TreeCtx) {
        /*if self.decoration.update(cx) {
            // TODO layout is not always necessary. Depending on what changed,
            // a repaint might be sufficient.
            self.content.padding = self.decoration.value_ref().insets();
            cx.mark_needs_layout();
        }*/

        self.content.update(cx)
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

    fn environment(&self) -> Environment {
        self.content.environment()
    }
    fn event(&mut self, ctx: &mut TreeCtx, event: &mut Event) {
        self.content.event(ctx, event)
    }

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
