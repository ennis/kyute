use crate::{cache_cx, CacheVar};
use std::{fmt, rc::Rc};

/// Wrapper over a cache var.
/// TODO doc, why not use Rc<CacheVar<T>> directly
pub struct State<T>(Rc<CacheVar<T>>);

impl<T> Clone for State<T> {
    fn clone(&self) -> Self {
        State(self.0.clone())
    }
}

impl<T> fmt::Debug for State<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}

impl<T: Default + 'static> Default for State<T> {
    #[track_caller]
    fn default() -> Self {
        Self::new(Default::default)
    }
}

impl<T: 'static> State<T> {
    #[track_caller]
    pub fn new(init: impl FnOnce() -> T) -> State<T> {
        State(cache_cx::variable(init).0)
    }

    /// Returns the value of the cache entry and replaces it by the given value.
    ///
    /// Always invalidates.
    /// Can be called outside of recomposition.
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn replace(&self, new_value: T) -> T {
        self.0.set_dependency();
        self.0.replace(new_value, true)
        /*#[cfg(debug_assertions)]
        let result = self
            .0
            .replace(new_value, true, (Location::caller(), "state variable updated"));
        #[cfg(not(debug_assertions))]
        let result = self.0.replace(new_value, true);
        result*/
    }

    /// Returns the value of the cache entry and replaces it by the default value.
    ///
    /// Does not invalidate the dependent entries.
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn replace_without_invalidation(&self, new_value: T) -> T {
        self.0.set_dependency();
        self.0.replace(new_value, false)
        /*
        #[cfg(debug_assertions)]
        let result = self.0.replace(new_value, false, (Location::caller(), ""));
        #[cfg(not(debug_assertions))]
        let result = self.0.replace(new_value, false);*/
    }

    /// Sets the value of the state variable.
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn set(&self, new_value: T) {
        // TODO idea: log the call sites that invalidated the cache, for debugging
        // e.g. `state entry @ (call site) invalidated because of (state entries), because of manual invalidation @ (call site) OR invalidated externally`
        self.replace(new_value);
    }

    pub fn set_without_invalidation(&self, new_value: T) {
        // TODO idea: log the call sites that invalidated the cache, for debugging
        // e.g. `state entry @ (call site) invalidated because of (state entries), because of manual invalidation @ (call site) OR invalidated externally `
        self.replace_without_invalidation(new_value);
    }

    /*/// Sets the value of the state variable.
    pub(crate) fn set_with_cause(&self, new_value: T, location: &'static Location<'static>, cause: impl AsRef<str>) {
        #[cfg(debug_assertions)]
        self.0.replace(new_value, true, (location, cause.as_ref()));
        #[cfg(not(debug_assertions))]
        self.0.replace(new_value, true);
    }*/

    pub fn update_with(&self, f: impl FnOnce(&mut T) -> bool) {
        self.0.set_dependency();
        self.0.update_with(f);
    }
}

impl<T: Clone + 'static> State<T> {
    pub fn get(&self) -> T {
        self.0.set_dependency();
        self.0.get()
    }
}

impl<T: Default + 'static> State<T> {
    /// Returns the value of the cache entry and replaces it by the default value.
    pub fn take(&self) -> T {
        self.replace(T::default())
    }

    /// Returns the value of the cache entry and replaces it by the default value. Does not invalidate dependent entries.
    pub fn take_without_invalidation(&self) -> T {
        self.replace_without_invalidation(T::default())
    }
}

impl<T: PartialEq + 'static> State<T> {
    ///
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn update(&self, new_value: T) {
        self.update_with(|value| {
            if *value != new_value {
                *value = new_value;
                true
            } else {
                false
            }
        });
    }
}
