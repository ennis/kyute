//! Baseline alignment.
use crate::{
    core::WindowPaintCtx,
    widget::{prelude::*, LayoutWrapper},
    GpuFrameCtx,
};
use kyute_common::RoundToPixel;

////////////////////////////////////////////////////////////////////////////////////////////////////
// Definition
////////////////////////////////////////////////////////////////////////////////////////////////////

/// A widget that aligns its child according to a fixed baseline.
#[derive(Clone)]
pub struct Baseline<Inner> {
    inner: Inner,
    baseline: f64,
}

impl<Inner: Widget + 'static> Baseline<Inner> {
    #[composable]
    pub fn new(baseline: f64, inner: Inner) -> Baseline<Inner> {
        Baseline { inner, baseline }
    }

    /// Returns a reference to the inner widget.
    pub fn inner(&self) -> &Inner {
        &self.inner
    }

    /// Returns a mutable reference to the inner widget.
    pub fn inner_mut(&mut self) -> &mut Inner {
        &mut self.inner
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// impl Widget
////////////////////////////////////////////////////////////////////////////////////////////////////

impl<Inner: Widget> Widget for Baseline<Inner> {
    fn widget_id(&self) -> Option<WidgetId> {
        // inherit the identity of the contents
        self.inner.widget_id()
    }

    fn layer(&self) -> &LayerHandle {
        self.inner.layer()
    }

    fn layout(&self, ctx: &mut LayoutCtx, constraints: BoxConstraints, env: &Environment) -> Measurements {
        let m = self.inner.layout(ctx, constraints, env);
        // baselines are not guaranteed to fall on a pixel boundary, round it manually
        let y_offset = (self.baseline - m.baseline.unwrap_or(m.size.height)).round_to_pixel(ctx.scale_factor);
        let offset = Offset::new(0.0, y_offset);
        self.inner.layer().set_offset(offset);
        Measurements::new(
            constraints
                .constrain(Size::new(m.width(), m.height() + y_offset))
                .into(),
        )
    }

    fn event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        self.inner.event(ctx, event, env);
    }
}
