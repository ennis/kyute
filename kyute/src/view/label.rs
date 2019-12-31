use crate::util::Ptr;
use crate::view::{Action, ActionCtx, View};
use miniqt_sys::*;
use std::marker::PhantomData;
use veda::Revision;

pub struct Label<A: Action> {
    label: Option<Ptr<QLabel>>,
    _phantom: PhantomData<*const A>,
}

impl<A: Action> Label<A> {
    pub fn new() -> Self {
        Label {
            label: None,
            _phantom: PhantomData,
        }
    }
}

impl<A: Action> View<String> for Label<A> {
    type Action = A;

    fn update(&mut self, rev: Revision<String>) {
        eprintln!("Label update {:?} {}", rev.address(), rev.data());

        assert!(self.label.is_some(), "not mounted");

        unsafe { QLabel_setText(self.label.unwrap().as_ptr(), &rev.data().into()) }
    }

    fn mount(&mut self, actx: ActionCtx<A>) {
        let label = Ptr::new(unsafe { QLabel_new() });
        self.label.replace(label);
    }

    fn widget_ptr(&self) -> Option<Ptr<QWidget>> {
        self.label.map(Ptr::upcast)
    }
}
