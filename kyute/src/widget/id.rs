use crate::{layout::BoxConstraints, Environment, LayoutCtx, Measurements, Visual, Widget};
use std::{any::TypeId, hash::Hash};

/// Identifies a widget.
pub struct Id<W> {
    inner: W,
}

impl<W> Id<W> {
    pub fn new(_id: impl Hash, inner: W) -> Id<W> {
        Id { inner }
    }
}

impl<W: Widget> Widget for Id<W> {
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
        // TODO ID?
        self.inner
            .layout(context, previous_visual, constraints, env)
    }
}
