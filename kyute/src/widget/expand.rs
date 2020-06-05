use crate::layout::BoxConstraints;
use crate::{Environment, LayoutCtx, Measurements, Visual, Widget};
use std::any::TypeId;

/// A widget that forces its contents to fill the available layout space.
pub struct Expand<W>(pub W);

impl<W: Widget> Widget for Expand<W> {
    fn key(&self) -> Option<u64> {
        self.0.key()
    }

    fn visual_type_id(&self) -> TypeId {
        self.0.visual_type_id()
    }

    fn layout(
        self,
        context: &mut LayoutCtx,
        previous_visual: Option<Box<dyn Visual>>,
        constraints: &BoxConstraints,
        env: Environment,
    ) -> (Box<dyn Visual>, Measurements) {
        self.0.layout(
            context,
            previous_visual,
            &BoxConstraints::tight(constraints.biggest()),
            env,
        )
    }
}
