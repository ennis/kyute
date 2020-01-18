use crate::util::Ptr;
use crate::view::{Action, ActionCtx, View};
use miniqt_sys::*;
use std::marker::PhantomData;
use veda::Revision;
use crate::view::Property;

pub struct Label<A: Action> {
    text: String,
    label: Option<Ptr<QLabel>>,
    _phantom: PhantomData<*const A>,
}

impl<A: Action> Label<A> {
    pub fn new() -> Self {
        Label {
            label: None,
            text: "".into(),
            _phantom: PhantomData,
        }
    }

    pub fn text<'a>(&'a mut self) -> impl Property<Value=String> + 'a {
        simple_property!{
            self: self,
            get: |this| this.text.clone(),
            update: |this, revision: Revision<String>| {
                this.text = revision.data().clone();
                this.update_text_internal();
            }
        }
    }

    fn update_text_internal(&mut self) {
        if let Some(label) = self.label {
            unsafe { QLabel_setText(label.as_ptr(), &(&self.text).into()) }
        }
    }
}

impl<A: Action> View for Label<A> {
    type Action = A;

    /*fn update(&mut self, rev: Revision<String>) {
        eprintln!("Label update {:?} {}", rev.address(), rev.data());

        assert!(self.label.is_some(), "not mounted");

        unsafe { QLabel_setText(self.label.unwrap().as_ptr(), &rev.data().into()) }
    }*/

    fn mount(&mut self, actx: ActionCtx<A>) {
        let label = Ptr::new(unsafe { QLabel_new() });
        self.label.replace(label);
        self.update_text_internal();
    }

    fn widget_ptr(&self) -> Option<Ptr<QWidget>> {
        self.label.map(Ptr::upcast)
    }
}
