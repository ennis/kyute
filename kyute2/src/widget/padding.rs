use std::any::Any;

use kurbo::{Affine, Insets, Point, Size, Vec2};

use crate::widget::prelude::*;

pub struct PaddingElement<E> {
    pub padding: Insets,
    pub size: Size,
    pub content: E,
}

impl<E: Element> Element for PaddingElement<E> {
    fn id(&self) -> ElementId {
        self.content.id()
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, constraints: &BoxConstraints) -> Geometry {
        let child_geometry = ctx.layout(&mut self.content, &constraints.deflate(self.padding));

        let offset = Vec2::new(self.padding.x0, self.padding.y0);
        self.size = Size {
            width: child_geometry.size.width + self.padding.x_value(),
            height: child_geometry.size.height + self.padding.y_value(),
        };

        Geometry {
            size: self.size,
            baseline: child_geometry.baseline.map(|baseline| baseline + offset.y),
            bounding_rect: child_geometry.bounding_rect + offset,
            paint_bounding_rect: child_geometry.paint_bounding_rect + offset,
        }
    }

    fn event(&mut self, ctx: &mut EventCtx, event: &mut Event) -> ChangeFlags {
        let offset = Vec2::new(self.padding.x0, self.padding.y0);
        event.with_transform(&Affine::translate(offset), |event| ctx.event(&mut self.content, event))
    }

    fn natural_width(&mut self, height: f64) -> f64 {
        self.content.natural_width((height - self.padding.y_value()).max(0.0))
    }

    fn natural_height(&mut self, width: f64) -> f64 {
        self.content.natural_height((width - self.padding.x_value()).max(0.0))
    }

    fn natural_baseline(&mut self, params: &BoxConstraints) -> f64 {
        self.content.natural_baseline(&params.deflate(self.padding)) + self.padding.y0
    }

    fn hit_test(&self, ctx: &mut HitTestResult, position: Point) -> bool {
        let offset = Vec2::new(self.padding.x0, self.padding.y0);
        let local_position = position - offset;
        if self.content.hit_test(ctx, local_position) {
            return true;
        }
        false
    }

    fn paint(&mut self, ctx: &mut PaintCtx) {
        ctx.with_transform(&Affine::translate(Vec2::new(self.padding.x0, self.padding.y0)), |ctx| {
            ctx.paint(&mut self.content);
        });
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn debug(&self, w: &mut DebugWriter) {
        w.type_name("PaddingElement");
        w.property("padding", self.padding);
        w.property("size", self.size);
        w.child("", &self.content);
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct Padding<W> {
    pub padding: Insets,
    pub content: W,
}

impl<W> Padding<W> {
    pub fn new(padding: Insets, content: W) -> Self {
        Self { padding, content }
    }
}

impl<W> Widget for Padding<W>
where
    W: Widget,
{
    type Element = PaddingElement<W::Element>;

    fn build(self, cx: &mut TreeCtx, _id: ElementId) -> Self::Element {
        PaddingElement {
            padding: self.padding,
            size: Default::default(),
            content: cx.build(self.content),
        }
    }

    fn update(self, cx: &mut TreeCtx, node: &mut Self::Element) -> ChangeFlags {
        node.padding = self.padding;
        cx.update(self.content, &mut node.content)
    }
}
