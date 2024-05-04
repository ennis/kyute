use crate::{context::ContextDataKey, utils::WidgetSet, ContextDataHandle, TreeCtx, WidgetId};
use std::{
    any::Any,
    cell::{Cell, RefCell},
    hash::{Hash, Hasher},
};

pub struct State<T: ?Sized> {
    /// The subtree of the UI that depends on this state, either for reading or writing.
    /// The tree is rooted at the UI root.
    dependents: RefCell<WidgetSet>,
    /// The state data
    pub data: T,
}

impl<T: Default> Default for State<T> {
    fn default() -> Self {
        State {
            dependents: Default::default(),
            data: Default::default(),
        }
    }
}

impl<T: 'static> State<T> {
    /// Creates a new state with the specified data.
    pub fn new(data: T) -> Self {
        State {
            dependents: Default::default(),
            data,
        }
    }

    pub fn set_dependency(&self, cx: &TreeCtx) {
        self.dependents.borrow_mut().insert(cx.current_path())
    }

    pub fn request_update(&self, cx: &TreeCtx) {
        cx.request_update(&self.dependents.borrow());
    }

    /// Modifies the state and notify the dependents.
    pub fn set(&mut self, cx: &mut TreeCtx, value: T) {
        self.data = value;
        self.request_update(cx);
    }

    /// Modifies the state and notify the dependents.
    pub fn modify(&mut self, cx: &mut TreeCtx, f: impl FnOnce(&mut T)) {
        f(&mut self.data);
        self.request_update(cx);
    }

    /// Returns the current value of the state.
    pub fn get(&self) -> &T {
        &self.data
    }
}

impl State<dyn Any> {
    pub fn downcast_ref<T: 'static>(&self) -> Option<&State<T>> {
        if self.data.is::<T>() {
            // SAFETY: the data is of the correct type
            Some(unsafe { &*(self as *const _ as *const State<T>) })
        } else {
            None
        }
    }

    pub fn downcast_mut<T: 'static>(&mut self) -> Option<&mut State<T>> {
        if self.data.is::<T>() {
            // SAFETY: the data is of the correct type
            Some(unsafe { &mut *(self as *mut _ as *mut State<T>) })
        } else {
            None
        }
    }
}

/// Ambient state key.
#[repr(transparent)]
pub struct AmbientKey<T>(ContextDataKey<State<T>>);

impl<T: 'static> AmbientKey<T> {
    pub const fn new(name: &'static str) -> AmbientKey<T> {
        AmbientKey(ContextDataKey::new(name))
    }

    pub fn get<'a>(self, cx: &'a TreeCtx) -> &'a T {
        let data = cx.keyed_data(self.0);
        data.set_dependency(cx);
        &data.data
    }
}

impl<T> Clone for AmbientKey<T> {
    fn clone(&self) -> Self {
        AmbientKey(self.0)
    }
}

impl<T> Copy for AmbientKey<T> {}

impl<T> PartialEq for AmbientKey<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T> Eq for AmbientKey<T> {}

impl<T> Hash for AmbientKey<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

pub fn with_ambient<T: 'static>(
    cx: &mut TreeCtx,
    ambient_key: AmbientKey<T>,
    state: &mut State<T>,
    f: impl FnOnce(&mut TreeCtx),
) {
    cx.with_keyed_data(ambient_key.0, state, |cx| {
        f(cx);
    });
}
