//! Stacking wdiget.
use crate::widget::prelude::*;

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

////////////////////////////////////////////////////////////////////////////////////////////////////
// impl Widget
////////////////////////////////////////////////////////////////////////////////////////////////////

impl<A: Widget + 'static, B: Widget + 'static> Widget for Overlay<A, B> {
    fn widget_id(&self) -> Option<WidgetId> {
        self.a.widget_id()
    }

    fn layout(&self, ctx: &mut LayoutCtx, constraints: &LayoutConstraints, env: &Environment) -> Layout {
        let sublayout = self.a.layout(ctx, constraints, env);
        let b_constraints = LayoutConstraints {
            min: sublayout.measurements.size,
            max: sublayout.measurements.size,
            ..*constraints
        };
        let _sublayout_b = self.b.layout(ctx, &b_constraints, env);
        sublayout
    }

    fn event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        self.a.route_event(ctx, event, env);
    }

    fn paint(&self, ctx: &mut PaintCtx) {
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
}
