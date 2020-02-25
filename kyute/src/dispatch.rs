use std::cell::RefCell;
use std::rc::Rc;

pub struct Dispatcher<A> {
    /// Dispatch closure
    dispatch: RefCell<Box<dyn Fn(A)>>,
}

impl<A> Dispatcher<A> {
    /// Receives the actions from the child views and dispatches them to the given handler.
    pub fn new<F: Fn(A) + 'static>(dispatch: F) -> Rc<Dispatcher<A>> {
        Rc::new(Dispatcher {
            dispatch: RefCell::new(Box::new(dispatch)),
        })
    }

    pub fn dispatch(&self, action: A) {
        (self.dispatch.borrow_mut())(action)
    }
}

pub type DispatcherHandle<A> = Rc<Dispatcher<A>>;
