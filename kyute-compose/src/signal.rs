use crate::{cache_cx, CacheVar};
use std::rc::Rc;

/// A primitive that can be used to signal value changes that affect the UI.
///
/// # Usage
///
/// In the memoized function:
///
/// ```
/// fn button() -> Button {
///     // Create a new signal (with no value). It has an associated cache variable that can be read.
///     let clicked = Signal::<()>::new();
///     //  
///     if clicked.signalled() {
///     }
/// }
/// ```
#[derive(Clone, Debug)]
pub struct Signal<T> {
    cache_var: Rc<CacheVar<Option<T>>>,
    value: Option<T>,
}

impl<T: Clone + 'static> Signal<T> {
    #[track_caller]
    pub fn new() -> Signal<T> {
        let (cache_var, _new) = cache_cx::variable(|| None);
        let value = cache_var.replace(None, false);
        Signal { cache_var, value }
    }

    #[cfg_attr(debug_assertions, track_caller)]
    pub fn signalled(&self) -> bool {
        self.cache_var.set_dependency();
        self.value.is_some()
    }

    #[cfg_attr(debug_assertions, track_caller)]
    pub fn value(&self) -> Option<T> {
        self.cache_var.set_dependency();
        self.value.clone()
    }

    #[cfg_attr(debug_assertions, track_caller)]
    pub fn map<U>(&self, f: impl FnOnce(T) -> U) -> Option<U> {
        self.cache_var.set_dependency();
        self.value.clone().map(f)
    }

    #[cfg_attr(debug_assertions, track_caller)]
    pub fn signal(&self, value: T) {
        self.cache_var.replace(Some(value), true);
        /*#[cfg(debug_assertions)]
        self.key
            .set_with_cause(Some(value), Location::caller(), "value signalled");
        #[cfg(not(debug_assertions))]
        self.key.set(Some(value));*/
    }
}
