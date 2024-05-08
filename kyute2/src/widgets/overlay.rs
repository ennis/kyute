//! Stacking widget.
use crate::{IntoWidgetPod, Widget, WidgetPtr};

#[derive(Copy, Clone, Debug)]
pub enum ZOrder {
    /// Draw B above A
    Above,
    /// Draw B below A
    Below,
}

/// Overlays one widget on top of the other.
pub struct Overlay {
    above: WidgetPtr,
    below: WidgetPtr,
    //z_order: ZOrder,
}

// Overlay is a wrapper over both A and B; in practice, they'll both be considered as children of the widget owning the overlay.
// I.e. when something changes inside, both A and B will be rebuilt

// Builders are not considered widget wrappers, but

impl Overlay {
    pub fn new(above: impl Widget + 'static, below: impl Widget + 'static) -> Self {
        Overlay { above, below }
    }
}

impl Widget for Overlay {
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
        self.a.natural_height(width)
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
}
