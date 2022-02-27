use crate::{cache, composable, EventCtx};
use std::cell::{Cell, RefCell};

/// FIXME: verify that the automatic clone impl doesn't have sketchy implications w.r.t. cache invalidation
#[derive(Clone, Debug)]
pub struct Signal<T> {
    fetched: Cell<bool>,
    value: RefCell<Option<T>>,
    key: cache::Key<Option<T>>,
}

impl<T: Clone + 'static> Signal<T> {
    #[composable]
    pub fn new() -> Signal<T> {
        let key = cache::state(|| None);
        Signal {
            fetched: Cell::new(false),
            value: RefCell::new(None),
            key,
        }
    }

    fn fetch_value(&self) {
        if !self.fetched.get() {
            let value = self.key.get();
            if value.is_some() {
                self.key.set(None);
            }
            self.value.replace(value);
            self.fetched.set(true);
        }
    }

    pub fn set(&self, value: T) {
        self.value.replace(Some(value));
        self.fetched.set(true);
    }

    pub fn signal(&self, ctx: &mut EventCtx, value: T) {
        ctx.set_state(self.key, Some(value));
    }

    pub fn signalled(&self) -> bool {
        self.fetch_value();
        self.value.borrow().is_some()
    }

    pub fn value(&self) -> Option<T> {
        self.fetch_value();
        self.value.borrow().clone()
    }
}

#[derive(Clone)]
pub struct State<T> {
    key: cache::Key<T>,
}

impl<T: Clone + 'static> State<T> {
    #[composable]
    pub fn new(init: impl FnOnce() -> T) -> State<T> {
        let key = cache::state(init);
        State { key }
    }

    pub fn get(&self) -> T {
        self.key.get()
    }

    pub fn update(&self, value: Option<T>) {
        if let Some(value) = value {
            self.key.set(value)
        }
    }

    pub fn set(&self, value: T) {
        self.key.set(value)
    }
}
