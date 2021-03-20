use crate::{
    layout::{BoxConstraints, Measurements},
    Environment, LayoutBox, LayoutCtx, Offset, SideOffsets, Size, TypedWidget, Widget,
};

/// Padding.
pub struct Padding<W> {
    inner: W,
    insets: SideOffsets,
}

impl<W> Padding<W> {
    pub fn new(insets: SideOffsets, inner: W) -> Padding<W> {
        Padding { inner, insets }
    }
}

impl<W: Widget> TypedWidget for Padding<W> {
    type Visual = LayoutBox;

    fn key(&self) -> Option<u64> {
        None
    }

    fn layout(
        self,
        context: &mut LayoutCtx,
        previous_visual: Option<Box<LayoutBox>>,
        constraints: &BoxConstraints,
        env: Environment,
    ) -> (Box<LayoutBox>, Measurements) {
        let Padding { inner, insets } = self;
        let visual = previous_visual.unwrap_or_default();

        let (child_id, child_measurements) =
            context.emit_child(inner, &constraints.deflate(&insets), env, None);
        context.set_child_offset(child_id, Offset::new(insets.left, insets.top));

        let measurements = Measurements {
            size: Size::new(
                child_measurements.size.width + insets.left + insets.right,
                child_measurements.size.height + insets.top + insets.bottom,
            ),
            baseline: child_measurements.baseline.map(|b| b + insets.top),
        };

        (visual, measurements)
    }
}
