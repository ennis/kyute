use crate::model::{Data, Lens, Revision};
use crate::util::Ptr;
use crate::view::{ActionCtx, View};
use bitflags::_core::marker::PhantomData;
use miniqt_sys::*;

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

    fn update(&mut self, s: &Revision<A>) {
        let inner = &mut self.inner;
        self.lens.focus(s, |s| inner.update(s));
    }

    fn mount(&mut self, actx: ActionCtx<V::Action>) {
        self.inner.mount(actx)
    }

    fn widget_ptr(&self) -> Option<Ptr<QWidget>> {
        self.inner.widget_ptr()
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
