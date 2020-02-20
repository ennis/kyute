//! # Database
//! The "source of truth", contains the state.
//!
//! A _revision_ is a snapshot of the database state at some point in time.
//! Revisions are identified by _revision numbers_.
//!
//! Note that there is only one revision available at a time (the latest one).
//! However, it is possible to rollback the database to the state of a previous revision with
//! undo operations. Note that this does not remove revisions: instead, it creates new revisions
//! that revert the changes (like git revert).
//!

use crate::model::update::{Append, Replace, Update};
use crate::model::Data;
use crate::model::Lens;
use crate::model::Revision;

use std::cell::RefCell;
use std::rc::{Rc, Weak};

//--------------------------------------------------------------------------------------------------

/// A view over a database.
pub trait Watcher<Root: Data> {
    /// Called by the database when something has changed.
    fn on_change(&self, revision: &Revision<Root>);
}

/// 'Database' wrapper for a data model that keeps track of changes in the model.
pub struct State<M: Data> {
    data: RefCell<M>,
    log: RefCell<Vec<Box<dyn Update<M>>>>,
    watchers: RefCell<Vec<Weak<dyn Watcher<M>>>>,
}

impl<M: Data> State<M> {
    /// Creates a new database wrapping an existing data model instance.
    pub fn new(data: M) -> State<M> {
        State {
            data: RefCell::new(data),
            log: RefCell::new(Vec::new()),
            watchers: RefCell::new(Vec::new()),
        }
    }

    pub fn update(&self, mut u: impl Update<M> + 'static) {
        let mut data = self.data.borrow_mut();
        // apply the update
        let change = u.apply(&mut *data);

        eprintln!("database update {:?}", u.address());

        // update all views
        let addr = u.address();
        let rev = Revision {
            change,
            data: &*data,
            addr: addr.clone(),
        };
        for w in self.watchers.borrow().iter() {
            if let Some(w) = w.upgrade() {
                w.on_change(&rev);
            }
        }

        // and record it in the log
        self.log.borrow_mut().push(Box::new(u));
    }

    /// Adds a watcher that will be called back immediately and whenever the state changes.
    pub fn add_watcher(&mut self, w: Rc<dyn Watcher<M>>) {
        let data = self.data.borrow();
        w.on_change(&((&*data).into()));
        self.watchers.borrow_mut().push(Rc::downgrade(&w))
    }

    pub fn with<R, F: FnOnce(&M) -> R>(&self, f: F) -> R {
        let data = self.data.borrow();
        f(&*data)
    }
}

impl<S: Data> State<S> {
    pub fn append<T: Data + Clone, A, K>(&mut self, lens: K, element: A)
    where
        K: Lens<S, T>,
        Append<A, K>: Update<S> + 'static,
    {
        self.update(Append::new(lens, element))
    }

    pub fn replace<T: Data + Clone, K: Lens<S, T>>(&mut self, lens: K, element: T)
    where
        Replace<K, T>: 'static,
    {
        self.update(Replace::new(lens, element))
    }
}
