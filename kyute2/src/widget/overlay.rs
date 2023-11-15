//! Stacking widget.
use crate::widget::prelude::*;
use std::any::Any;

////////////////////////////////////////////////////////////////////////////////////////////////////
// Widget definition
////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, Debug)]
pub enum ZOrder {
    /// Draw B above A
    Above,
    /// Draw B below A
    Below,
}

/// Overlays one widget on top of the other.
///
/// The widget's layout and identity is defined by `A`, events are only forwarded to A.
pub struct Overlay<A, B> {
    a: A,
    b: B,
    z_order: ZOrder,
}

impl<A: Widget + 'static, B: Widget + 'static> Overlay<A, B> {
    pub fn new(a: A, b: B, z_order: ZOrder) -> Overlay<A, B> {
        Overlay { a, b, z_order }
    }

    /// Returns a reference to the inner widget (A).
    pub fn inner(&self) -> &A {
        &self.a
    }

    /// Returns a mutable reference to the inner widget.
    pub fn inner_mut(&mut self) -> &mut A {
        &mut self.a
    }
}

impl<A: Widget, B: Widget> Widget for Overlay<A, B> {
    type Element = OverlayElement<A::Element, B::Element>;

    fn build(self, cx: &mut TreeCtx, element_id: ElementId) -> Self::Element {
        OverlayElement {
            a: cx.build(self.a),
            b: cx.build(self.b),
            z_order: self.z_order,
        }
    }

    fn update(self, cx: &mut TreeCtx, element: &mut Self::Element) -> ChangeFlags {
        let mut flags = ChangeFlags::empty();
        flags |= cx.update(self.a, &mut element.a);
        flags |= cx.update(self.b, &mut element.b);
        flags
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Element
////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct OverlayElement<A, B> {
    a: A,
    b: B,
    z_order: ZOrder,
}

impl<A: Element, B: Element> Element for OverlayElement<A, B> {
    fn id(&self) -> ElementId {
        self.a.id()
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, params: &BoxConstraints) -> Geometry {
        let sublayout = ctx.layout(&mut self.a, params);
        let b_constraints = BoxConstraints {
            min: sublayout.size,
            max: sublayout.size,
            ..*params
        };
        let _sublayout_b = ctx.layout(&mut self.b, &b_constraints);
        sublayout
    }

    fn event(&mut self, ctx: &mut EventCtx, event: &mut Event) -> ChangeFlags {
        ctx.event(&mut self.a, event)
    }

    fn natural_width(&mut self, height: f64) -> f64 {
        self.a.natural_width(height)
    }

    fn natural_height(&mut self, width: f64) -> f64 {
        self.b.natural_height(width)
    }

    fn natural_baseline(&mut self, params: &BoxConstraints) -> f64 {
        self.a.natural_baseline(params)
    }

    fn hit_test(&self, ctx: &mut HitTestResult, position: Point) -> bool {
        self.a.hit_test(ctx, position)
    }

    fn paint(&mut self, ctx: &mut PaintCtx) {
        match self.z_order {
            ZOrder::Above => {
                ctx.paint(&mut self.a);
                ctx.paint(&mut self.b);
            }
            ZOrder::Below => {
                ctx.paint(&mut self.b);
                ctx.paint(&mut self.a);
            }
        }
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn debug(&self, visitor: &mut DebugWriter) {
        visitor.type_name("OverlayElement");
        visitor.property("id", self.id());
        visitor.property("z_order", self.z_order);
        visitor.child("a", &self.a);
        visitor.child("b", &self.b);
    }
}
