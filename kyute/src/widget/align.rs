use crate::{
    layout, layout::BoxConstraints, Alignment, Environment, LayoutBox, LayoutCtx, Measurements,
    TypedWidget, Widget,
};

pub struct Align<W> {
    alignment: Alignment,
    inner: W,
}

impl<W> Align<W> {
    pub fn new(alignment: Alignment, inner: W) -> Align<W> {
        Align { alignment, inner }
    }
}

impl<W: Widget> TypedWidget for Align<W> {
    type Visual = LayoutBox;

    fn layout(
        self,
        context: &mut LayoutCtx,
        previous_visual: Option<Box<LayoutBox>>,
        constraints: &BoxConstraints,
        env: Environment,
    ) -> (Box<LayoutBox>, Measurements) {
        let visual = previous_visual.unwrap_or_default();
        let (child_id, child_measurements) =
            context.emit_child(self.inner, &constraints.loosen(), env, None);
        let mut measurements = Measurements::new(constraints.constrain(child_measurements.size));
        let child_offset =
            layout::align_boxes(self.alignment, &mut measurements, child_measurements);
        context.set_child_offset(child_id, child_offset);
        (visual, measurements)
    }
}
