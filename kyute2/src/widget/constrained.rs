use crate::widget::prelude::*;
use std::any::Any;

pub struct ConstrainedElement<E> {
    constraints: BoxConstraints,
    content: E,
}

impl<E> Element for ConstrainedElement<E>
where
    E: Element,
{
    fn id(&self) -> ElementId {
        self.content.id()
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, params: &BoxConstraints) -> Geometry {
        let mut subconstraints = *params;
        subconstraints.min.width = subconstraints.min.width.max(self.constraints.min.width);
        subconstraints.min.height = subconstraints.min.height.max(self.constraints.min.height);
        subconstraints.max.width = subconstraints.max.width.min(self.constraints.max.width);
        subconstraints.max.height = subconstraints.max.height.min(self.constraints.max.height);
        ctx.layout(&mut self.content, &subconstraints)
    }

    fn event(&mut self, ctx: &mut EventCtx, event: &mut Event) -> ChangeFlags {
        self.content.event(ctx, event)
    }

    fn natural_width(&mut self, height: f64) -> f64 {
        self.content.natural_width(height)
    }

    fn natural_height(&mut self, width: f64) -> f64 {
        self.content.natural_height(width)
    }

    fn natural_baseline(&mut self, params: &BoxConstraints) -> f64 {
        self.content.natural_baseline(params)
    }

    fn hit_test(&self, ctx: &mut HitTestResult, position: Point) -> bool {
        self.content.hit_test(ctx, position)
    }

    fn paint(&mut self, ctx: &mut PaintCtx) {
        self.content.paint(ctx)
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn debug(&self, w: &mut DebugWriter) {
        w.type_name("ConstrainedElement");
        w.property("constraints", self.constraints);
        w.child("", &self.content)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct Constrained<W> {
    pub constraints: BoxConstraints,
    pub content: W,
}

impl<W> Constrained<W> {
    pub fn new(constraints: BoxConstraints, content: W) -> Self {
        Self { constraints, content }
    }
}

impl<W> Widget for Constrained<W>
where
    W: Widget,
{
    type Element = ConstrainedElement<W::Element>;

    fn build(self, cx: &mut TreeCtx, _id: ElementId) -> Self::Element {
        ConstrainedElement {
            constraints: self.constraints,
            content: cx.build(self.content),
        }
    }

    fn update(self, cx: &mut TreeCtx, element: &mut Self::Element) -> ChangeFlags {
        let mut flags = ChangeFlags::empty();
        if element.constraints != self.constraints {
            element.constraints = self.constraints;
            flags |= ChangeFlags::GEOMETRY;
        }
        flags | cx.update(self.content, &mut element.content)
    }
}
