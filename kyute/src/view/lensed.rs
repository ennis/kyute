use crate::event::Event;
use crate::model::{Data, Lens, Revision};
use crate::paint::RenderContext;
use crate::view::{EventCtx, View};
use std::marker::PhantomData;

pub struct Lensed<B, L, V> {
    lens: L,
    inner: V,
    _phantom: PhantomData<B>,
}

impl<A, B, L, V> View<A> for Lensed<B, L, V>
where
    A: Data,
    B: Data,
    L: Lens<A, B>,
    V: View<B>,
{
    type Action = V::Action;

    fn event(&mut self, e: &Event, a: &mut EventCtx<Self::Action>) {
        self.inner.event(e, a)
    }

    fn update(&mut self, state: &Revision<A>) {
        let inner = &mut self.inner;
        self.lens.focus(state, |state| inner.update(state));
    }

    fn paint(&mut self, state: &A, ctx: &mut RenderContext) -> bool {
        let inner = &mut self.inner;
        self.lens.with(state, |state| inner.paint(state, ctx))
    }
}

impl<B, L, V> Lensed<B, L, V> {
    pub fn new(lens: L, inner: V) -> Lensed<B, L, V> {
        Lensed {
            lens,
            inner,
            _phantom: PhantomData,
        }
    }
}
