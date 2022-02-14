//! Baseline alignment.
use crate::{
    composable, widget::LayoutWrapper, BoxConstraints, Environment, Event, EventCtx, LayoutCtx,
    Measurements, Offset, PaintCtx, Rect, Size, Widget, WidgetPod,
};

/// A widget that aligns its child according to a fixed baseline.
#[derive(Clone)]
pub struct Baseline<Inner> {
    inner: LayoutWrapper<Inner>,
    baseline: f64,
}

impl<Inner: Widget + 'static> Baseline<Inner> {
    #[composable(uncached)]
    pub fn new(baseline: f64, inner: Inner) -> Baseline<Inner> {
        Baseline {
            inner: LayoutWrapper::new(inner),
            baseline,
        }
    }
}

impl<Inner: Widget> Widget for Baseline<Inner> {
    fn event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        self.inner.event(ctx, event, env);
    }

    fn layout(
        &self,
        ctx: &mut LayoutCtx,
        constraints: BoxConstraints,
        env: &Environment,
    ) -> Measurements {
        let m = self.inner.layout(ctx, constraints, env);
        // baselines are not guaranteed to fall on a pixel boundary, round it manually
        // FIXME should do pixel snapping instead
        let y_offset = (self.baseline - m.baseline.unwrap_or(m.bounds.size.height)).round();
        let offset = Offset::new(0.0, y_offset);
        self.inner.set_offset(offset);
        Measurements::new(
            constraints
                .constrain(Size::new(m.width(), m.height() + y_offset))
                .into(),
        )
    }

    fn paint(&self, ctx: &mut PaintCtx, bounds: Rect, env: &Environment) {
        self.inner.paint(ctx, bounds, env);
    }
}
