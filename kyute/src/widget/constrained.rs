use crate::layout::BoxConstraints;
use crate::{Environment, LayoutCtx, Measurements, Visual, Widget};
use generational_indextree::NodeId;
use std::any::TypeId;

/// Expands the child widget to fill all its available space.
pub struct ConstrainedBox<W> {
    constraints: BoxConstraints,
    inner: W,
}

impl<W> ConstrainedBox<W> {
    pub fn new(constraints: BoxConstraints, inner: W) -> ConstrainedBox<W> {
        ConstrainedBox { constraints, inner }
    }
}

impl<W: Widget> Widget for ConstrainedBox<W> {
    fn key(&self) -> Option<u64> {
        self.inner.key()
    }

    fn visual_type_id(&self) -> TypeId {
        self.inner.visual_type_id()
    }

    fn layout(
        self,
        context: &mut LayoutCtx,
        previous_visual: Option<Box<dyn Visual>>,
        constraints: &BoxConstraints,
        env: Environment,
    ) -> (Box<dyn Visual>, Measurements) {
        let constraints = constraints.enforce(&self.constraints);
        self.inner
            .layout(context, previous_visual, &constraints, env)
    }
}
