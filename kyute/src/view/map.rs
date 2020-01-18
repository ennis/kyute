use crate::util::Ptr;
use crate::view::{Action, ActionCtx, ActionTransformer, View};
use miniqt_sys::QWidget;
use std::rc::Rc;
use veda::{Data, Revision};

pub struct Map<S: Data, A: Action, V: View<S>, F: Fn(V::Action) -> A + 'static> {
    inner: V,
    actx: Rc<ActionTransformer<V::Action, A, F>>,
}

impl<S: Data, A: Action, V: View<S>, F: Fn(V::Action) -> A + 'static> View<S> for Map<S, A, V, F> {
    type Action = A;

    /*fn update(&mut self, rev: Revision<S>) {
        self.inner.update(rev)
    }*/

    fn mount(&mut self, actx: ActionCtx<A>) {
        self.actx.set_parent(actx.clone())
    }

    fn widget_ptr(&self) -> Option<Ptr<QWidget>> {
        self.inner.widget_ptr()
    }
}

impl<S: Data, A: Action, V: View<S>, F: Fn(V::Action) -> A + 'static> Map<S, A, V, F> {
    pub fn new(mut inner: V, transform: F) -> Map<S, A, V, F> {
        let actx = ActionTransformer::new(transform);
        inner.mount(actx.clone());
        Map { inner, actx }
    }
}
