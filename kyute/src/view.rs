#[macro_use]
mod property;

mod binding;
mod button;
mod checkbox;
mod label;
//mod lensed;
//mod list;
//mod map;
mod root;
mod vbox;
mod tuple;

use crate::util::Ptr;
use miniqt_sys::QWidget;
use std::cell::RefCell;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::rc::Rc;
use veda::{Data, Revision};

pub use button::Button;
pub use button::ButtonAction;
pub use checkbox::Checkbox;
pub use checkbox::CheckboxState;
pub use label::Label;
//pub use lensed::Lensed;
//pub use list::List;
//pub use map::Map;
pub use root::Root;
pub use vbox::VBox;
pub use property::Property;
pub use property::SimpleProperty;
//pub use binding::Binding;


pub trait Action: Clone + Debug + 'static {}
impl<T: Clone + Debug + 'static> Action for T {}

pub trait View
{
    type Action: Action;
    fn mount(&mut self, actx: ActionCtx<Self::Action>);
    fn widget_ptr(&self) -> Option<Ptr<QWidget>>;
}

pub trait ViewCollection {
    type Action: Action;

    fn mount(&mut self, actx: ActionCtx<Self::Action>);
    fn widgets(&self) -> Vec<Ptr<QWidget>>;
}

/// Receives an action
pub trait ActionSink<A: Action> {
    fn emit(&self, action: A);
}

pub type ActionCtx<A> = Rc<dyn ActionSink<A>>;

pub struct ActionRoot<A: Action> {
    received: RefCell<Vec<A>>,
}

impl<A: Action> ActionRoot<A> {
    pub fn new() -> Rc<ActionRoot<A>> {
        Rc::new(ActionRoot {
            received: RefCell::new(Vec::new()),
        })
    }

    pub fn collect_actions(&self) -> Vec<A> {
        self.received.replace(Vec::new())
    }
}

impl<A: Action> ActionSink<A> for ActionRoot<A> {
    fn emit(&self, action: A) {
        self.received.borrow_mut().push(action)
    }
}

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

/*pub trait ViewExt<S: Data>: View<S> {
    fn map<A, F>(self, closure: F) -> Map<S, A, Self, F>
    where
        Self: Sized,
        A: Action,
        F: Fn(Self::Action) -> A;
}

impl<S: Data, V: View<S>> ViewExt<S> for V {
    fn map<A, F>(self, closure: F) -> Map<S, A, Self, F>
    where
        Self: Sized,
        A: Action,
        F: Fn(Self::Action) -> A,
    {
        Map::new(self, closure)
    }
}
*/