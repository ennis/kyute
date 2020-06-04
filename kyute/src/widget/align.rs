use crate::layout::BoxConstraints;
use crate::{Alignment, Measurements, Widget, LayoutCtx, TypedWidget, LayoutBox, layout};
use generational_indextree::NodeId;
use crate::Environment;

pub struct Align<W> {
    alignment: Alignment,
    inner: W,
}

impl<W> Align<W> {
    pub fn new(alignment: Alignment, inner: W) -> Align<W> {
        Align { alignment, inner }
    }
}


impl<A: 'static, W> TypedWidget<A> for Align<W>
where
    W: Widget<A>,
{
    type Visual = LayoutBox;

    fn layout(
        self,
        context: &mut LayoutCtx<A>,
        previous_visual: Option<Box<LayoutBox>>,
        constraints: &BoxConstraints,
        env: Environment,
    ) -> (Box<LayoutBox>, Measurements)
    {
        let visual = previous_visual.unwrap_or_default();
        let (child_id, child_measurements) = context.emit_child(self.inner, &constraints.loosen(), env);
        let mut measurements = Measurements::new(constraints.constrain(child_measurements.size));
        let child_offset = layout::align_boxes(self.alignment, &mut measurements, child_measurements);
        context.set_child_offset(child_id, child_offset);
        (visual, measurements)
    }
}
