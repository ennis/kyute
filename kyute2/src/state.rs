use crate::{
    context::ContextDataKey,
    utils::WidgetSet,
    widget::{TreeCtx, WeakWidgetPtr, WidgetPtr},
    ContextDataHandle, WidgetId,
};
use std::{
    any::Any,
    cell::{Cell, Ref, RefCell},
    collections::HashSet,
    hash::{Hash, Hasher},
    rc::Rc,
};
use weak_table::PtrWeakHashSet;

struct StateInner<T: ?Sized> {
    dependents: RefCell<PtrWeakHashSet<WeakWidgetPtr>>,
    data: RefCell<T>,
}

pub struct State<T: ?Sized>(Rc<StateInner<T>>);

impl<T: Default + 'static> Default for State<T> {
    fn default() -> Self {
        State::new(T::default())
    }
}

impl<T> Clone for State<T> {
    fn clone(&self) -> Self {
        State(Rc::clone(&self.0))
    }
}

impl<T: 'static> State<T> {
    /// Creates a new state with the specified data.
    pub fn new(data: T) -> Self {
        State(Rc::new(StateInner {
            dependents: Default::default(),
            data: RefCell::new(data),
        }))
    }

    pub fn set_dependency(&self, cx: &TreeCtx) {
        self.0.dependents.borrow_mut().insert(cx.current());
    }

    fn notify(&self, cx: &TreeCtx) {
        for dep in self.0.dependents.borrow().iter() {
            cx.mark_needs_update(dep);
        }
    }

    /// Modifies the state and notify the dependents.
    pub fn set(&self, cx: &mut TreeCtx, value: T) {
        self.0.data.replace(value);
        self.notify(cx);
    }

    /// Modifies the state and notify the dependents.
    pub fn update<R>(&self, cx: &mut TreeCtx, f: impl FnOnce(&mut T) -> R) -> R {
        let mut data = self.0.data.borrow_mut();
        let r = f(&mut *data);
        self.notify(cx);
        r
    }

    /// Returns the current value of the state, setting a dependency on the current widget.
    pub fn get(&self, cx: &mut TreeCtx) -> Ref<T> {
        self.set_dependency(cx);
        self.0.data.borrow()
    }

    pub fn get_untracked(&self) -> Ref<T> {
        self.0.data.borrow()
    }
}

impl State<dyn Any> {
    pub fn downcast_ref<T: 'static>(&self) -> Option<&State<T>> {
        if self.0.data.borrow().is::<T>() {
            // SAFETY: the data is of the correct type
            Some(unsafe { &*(self as *const _ as *const State<T>) })
        } else {
            None
        }
    }

    pub fn downcast_mut<T: 'static>(&mut self) -> Option<&mut State<T>> {
        if self.0.data.borrow().is::<T>() {
            // SAFETY: the data is of the correct type
            Some(unsafe { &mut *(self as *mut _ as *mut State<T>) })
        } else {
            None
        }
    }
}

/*
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

    pub fn get_mut_untracked<'a>(self, cx: &'a mut TreeCtx) -> &'a mut T {
        let data = cx.keyed_data(self.0);
        &mut data.data
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
*/
