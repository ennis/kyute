use crate::model::{Data, Revision};
use crate::util::Ptr;
use crate::view::{Action, ActionCtx, ActionSink, View};
use miniqt_sys::QWidget;
use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;

pub struct ActionTransformer<A: Action, B: Action, F: Fn(A) -> B> {
    parent: RefCell<Option<ActionCtx<B>>>,
    transform: F,
    _phantom: PhantomData<*const A>,
}

impl<A: Action, B: Action, F: Fn(A) -> B> ActionTransformer<A, B, F> {
    pub fn new(transform: F) -> Rc<ActionTransformer<A, B, F>> {
        Rc::new(ActionTransformer {
            parent: RefCell::new(None),
            transform,
            _phantom: PhantomData,
        })
    }

    pub fn set_parent(&self, parent: ActionCtx<B>) {
        self.parent.replace(Some(parent));
    }
}

impl<A: Action, B: Action, F: Fn(A) -> B> ActionSink<A> for ActionTransformer<A, B, F> {
    fn emit(&self, action: A) {
        self.parent
            .borrow()
            .as_ref()
            .map(|x| x.emit((&self.transform)(action)));
    }
}

pub struct Map<S: Data, A: Action, V: View<S>, F: Fn(V::Action) -> A + 'static> {
    inner: V,
    actx: Rc<ActionTransformer<V::Action, A, F>>,
}

impl<S: Data, A: Action, V: View<S>, F: Fn(V::Action) -> A + 'static> View<S> for Map<S, A, V, F> {
    type Action = A;

    fn update(&mut self, rev: &Revision<S>) {
        self.inner.update(rev)
    }

    fn mount(&mut self, actx: ActionCtx<A>) {
        self.actx.set_parent(actx.clone());
        self.inner.mount(self.actx.clone());
    }

    fn widget_ptr(&self) -> Option<Ptr<QWidget>> {
        self.inner.widget_ptr()
    }
}

impl<S: Data, A: Action, V: View<S>, F: Fn(V::Action) -> A + 'static> Map<S, A, V, F> {
    pub fn new(mut inner: V, transform: F) -> Map<S, A, V, F> {
        let actx = ActionTransformer::new(transform);
        Map { inner, actx }
    }
}
