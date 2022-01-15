//! Baseline alignment.
use kyute::{Widget, WidgetPod, composable};
use crate::{BoxConstraints, Environment, Event, EventCtx, GpuFrameCtx, LayoutCtx, Measurements, Offset, PaintCtx, Rect};
use crate::core2::WindowPaintCtx;

/// A widget that aligns its child according to a fixed baseline.
pub struct Baseline {
    inner: WidgetPod,
    baseline: f64,
}

impl Baseline {
    #[composable]
    pub fn new(baseline: f64, inner: WidgetPod) -> WidgetPod<Baseline> {
        WidgetPod::new(Baseline { inner, baseline })
    }
}

impl Widget for Baseline {

    fn event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        self.inner.event(ctx, event, env)
    }

    fn layout(
        &self,
        ctx: &mut LayoutCtx,
        constraints: BoxConstraints,
        env: &Environment,
    ) -> Measurements {
        let child_measurements = self.inner.layout(ctx, constraints, env);

        // baselines are not guaranteed to fall on a pixel boundary, round it manually
        // FIXME should do pixel snapping instead
        let y_offset = (self.baseline
            - child_measurements
            .baseline
            .unwrap_or(child_measurements.size.height))
            .round();

        self.inner.set_child_offset(Offset::new(0.0, y_offset));

        let width = child_measurements.size.width;
        let height = child_measurements.size.height + y_offset;

        let measurements = Measurements {
            size: constraints.constrain((width, height).into()),
            baseline: Some(self.baseline),
            is_window: false
        };

        // TODO: layout() should return an arbitrary box, in local coordinates, not necessarily something
        // with the origin at the top-left corner.
        // This way we'd be able to translate the inner widget without the need for a separate widgetpod.
        measurements
    }

    fn paint(&self, ctx: &mut PaintCtx, bounds: Rect, env: &Environment) {
        self.inner.paint(ctx, bounds, env)
    }
}