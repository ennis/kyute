use crate::event::Event;
use crate::layout::BoxConstraints;
use crate::{Point, Widget, LayoutCtx, Visual, Measurements, Environment};
use generational_indextree::NodeId;
use std::marker::PhantomData;
use std::rc::Rc;
use std::any::TypeId;

/// Map one action to another.
pub struct Map<A, W, F> {
    inner: W,
    map: F,
    _phantom: PhantomData<A>,
}

impl<A, W, F> Map<A, W, F> {
    pub fn new(inner: W, map: F) -> Map<A, W, F> {
        Map {
            inner,
            map,
            _phantom: PhantomData,
        }
    }
}

impl<A: 'static, B: 'static, W: Widget<A>, F: Fn(A) -> B + 'static> Widget<B> for Map<A, W, F> {
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
    ) -> (Box<dyn Visual>, Measurements)
    {
        let mut ctx = context.map(self.map);
        self.inner
            .layout(&mut ctx, previous_visual, constraints, env)
    }
}
