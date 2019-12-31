use crate::util::Ptr;
use crate::view::{ActionCtx, View};
use miniqt_sys::*;
use veda::lens::Lens;
use veda::Revision;

pub struct Lensed<L: Lens, V: View<L::Leaf>> {
    lens: L,
    inner: V,
}

impl<L: Lens, V: View<L::Leaf>> View<L::Root> for Lensed<L, V> {
    type Action = V::Action;

    fn update(&mut self, rev: Revision<L::Root>) {
        rev.focus(self.lens.clone())
            .map(|rev| self.inner.update(rev));
    }

    fn mount(&mut self, actx: ActionCtx<V::Action>) {
        self.inner.mount(actx)
    }

    fn widget_ptr(&self) -> Option<Ptr<QWidget>> {
        self.inner.widget_ptr()
    }
}

impl<L: Lens, V: View<L::Leaf>> Lensed<L, V> {
    pub fn new(lens: L, inner: V) -> Lensed<L, V> {
        Lensed { lens, inner }
    }
}
