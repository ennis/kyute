//! Stacking widget.
use crate::{widget::prelude::*, RouteEventCtx};
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
    #[composable]
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

    fn id(&self) -> WidgetId {
        self.a.id()
    }

    fn build(self, cx: &mut TreeCtx, env: &Environment) -> Self::Element {
        OverlayElement {
            a: self.a.build(cx, env),
            b: self.b.build(cx, env),
            z_order: self.z_order,
        }
    }

    fn update(self, cx: &mut TreeCtx, element: &mut Self::Element, env: &Environment) -> ChangeFlags {
        let mut flags = ChangeFlags::empty();
        flags |= self.a.update(cx, &mut element.a, env);
        flags |= self.b.update(cx, &mut element.b, env);
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
    fn id(&self) -> WidgetId {
        self.a.id()
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, params: &LayoutParams) -> Geometry {
        let sublayout = self.a.layout(ctx, params);
        let b_constraints = LayoutParams {
            min: sublayout.size,
            max: sublayout.size,
            ..*params
        };
        let _sublayout_b = self.b.layout(ctx, &b_constraints);
        sublayout
    }

    fn event(&mut self, ctx: &mut EventCtx, event: &mut Event) -> ChangeFlags {
        self.a.event(ctx, event)
    }

    fn natural_size(&mut self, axis: Axis, params: &LayoutParams) -> f64 {
        self.a.natural_size(axis, params)
    }

    fn natural_baseline(&mut self, params: &LayoutParams) -> f64 {
        self.a.natural_baseline(params)
    }

    fn hit_test(&self, ctx: &mut HitTestResult, position: Point) -> bool {
        self.a.hit_test(ctx, position)
    }

    fn paint(&mut self, ctx: &mut PaintCtx) {
        match self.z_order {
            ZOrder::Above => {
                self.a.paint(ctx);
                self.b.paint(ctx);
            }
            ZOrder::Below => {
                self.b.paint(ctx);
                self.a.paint(ctx);
            }
        }
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn debug(&self, visitor: &mut DebugWriter) {
        visitor.type_name("OverlayElement");
        visitor.property("z_order", self.z_order);
        visitor.child("a", &self.a);
        visitor.child("b", &self.b);
    }
}
