//! Frame decorations
use kurbo::{Insets, PathEl, Point, Rect, RoundedRect, Size};
use smallvec::SmallVec;
use tracing::warn;

use crate::{
    drawing,
    drawing::{BoxShadow, Paint, Shape, ToSkia},
    environment::Environment,
    skia,
    widgets::Padding,
    BoxConstraints, Color, Event, Geometry, HitTestResult, LayoutCtx, PaintCtx, TreeCtx, Widget,
};
use crate::drawing::Decoration;

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
        self.decoration.paint(ctx, self.size.to_rect());
        self.content.paint(ctx);
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

/*
// TODO this should take a "Decoration" directly, not a Border
pub struct DecoratedBox<D, W> {
    pub decoration: D,
    pub content: W,
}

impl<D, W> DecoratedBox<D, W> {
    pub fn new(decoration: D, content: W) -> Self {
        Self { decoration, content }
    }
}

impl<D, W> Widget for DecoratedBox<D, W>
where
    D: Decoration + 'static,
    W: Widget,
{
    type Element = DecoratedBoxElement<D, W::Element>;

    fn build(self, cx: &mut TreeCtx, _id: ElementId) -> Self::Element {
        let padding = self.decoration.insets();
        DecoratedBoxElement {
            decoration: self.decoration,
            content: PaddingElement {
                padding,
                content: cx.build(self.content),
                size: Default::default(),
            },
        }
    }

    fn update(self, cx: &mut TreeCtx, element: &mut Self::Element) -> ChangeFlags {
        let mut flags = ChangeFlags::empty();
        if element.decoration != self.decoration {
            let padding = self.decoration.insets();
            if element.content.padding != padding {
                element.content.padding = padding;
                flags |= ChangeFlags::GEOMETRY;
            }
            element.decoration = self.decoration;
            flags |= ChangeFlags::PAINT;
        }
        flags | cx.update(self.content, &mut element.content.content)
    }
}
*/
