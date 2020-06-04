use crate::layout::BoxConstraints;
use crate::{Widget, LayoutCtx, Visual, Environment, Measurements};
use generational_indextree::NodeId;
use std::hash::Hash;
use std::any::TypeId;

/// Identifies a widget.
pub struct Id<W> {
    inner: W,
}

impl<W> Id<W> {
    pub fn new(_id: impl Hash, inner: W) -> Id<W> {
        Id { inner }
    }
}

impl<A: 'static, W: Widget<A>> Widget<A> for Id<W> {
    fn key(&self) -> Option<u64> {
        self.inner.key()
    }

    fn visual_type_id(&self) -> TypeId {
        self.inner.visual_type_id()
    }

    fn layout(
        self,
        context: &mut LayoutCtx<A>,
        previous_visual: Option<Box<dyn Visual>>,
        constraints: &BoxConstraints,
        env: Environment,
    ) -> (Box<dyn Visual>, Measurements) {
        // TODO ID?
        self.inner.layout(context, previous_visual, constraints, env)
    }
}
