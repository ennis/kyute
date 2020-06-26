use crate::layout::{BoxConstraints, Measurements};
use crate::{Environment, LayoutBox, LayoutCtx, TypedWidget, Widget, Offset};
use generational_indextree::NodeId;

/// A widget that aligns its child according to a fixed baseline.
pub struct Baseline<W> {
    inner: W,
    baseline: f64,
}

impl<W> Baseline<W> {
    pub fn new(baseline: f64, inner: W) -> Baseline<W> {
        Baseline { inner, baseline }
    }
}

impl<W: Widget> TypedWidget for Baseline<W> {
    type Visual = LayoutBox;

    fn layout(
        self,
        context: &mut LayoutCtx,
        previous_visual: Option<Box<Self::Visual>>,
        constraints: &BoxConstraints,
        env: Environment,
    ) -> (Box<Self::Visual>, Measurements) {
        let visual = previous_visual.unwrap_or_default();
        let (child_id, child_measurements) = context.emit_child(self.inner, constraints, env, None);

        // baselines are not guaranteed to fall on a pixel boundary, round it manually
        // FIXME should do pixel snapping instead
        let y_offset = (self.baseline
            - child_measurements
                .baseline
                .unwrap_or(child_measurements.size.height))
        .round();
        context.set_child_offset(child_id, Offset::new(0.0, y_offset));

        let width = child_measurements.size.width;
        let height = child_measurements.size.height + y_offset;

        let measurements = Measurements {
            size: constraints.constrain((width, height).into()),
            baseline: Some(self.baseline),
        };

        (visual, measurements)
    }
}
