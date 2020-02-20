use crate::model::Data;
use crate::model::{Change, Lens};

/// Represents an edit on a value of type T (i.e. a modification of part of the object).
pub trait Update<T: Data> {
    fn apply(&mut self, model: &mut T) -> Change;
    fn address(&self) -> Option<T::Address>;
}

/// Represents a partial change to an aggregate model.
pub struct Replace<K, T> {
    lens: K,
    value: T,
}

impl<K, T> Replace<K, T> {
    pub fn new(lens: K, value: T) -> Replace<K, T> {
        Replace { lens, value }
    }
}

impl<S, T, K> Update<S> for Replace<K, T>
where
    S: Data,
    T: Data + Clone,
    K: Lens<S, T>,
{
    fn apply(&mut self, data: &mut S) -> Change {
        let value = self.value.clone();
        self.lens.with_mut(data, |v| *v = value);
        Change::replacement()
    }

    fn address(&self) -> Option<S::Address> {
        self.lens.address()
    }
}

/// Append operation.
pub struct Append<A, K> {
    lens: K,
    element: A,
}

impl<A, K> Append<A, K> {
    pub fn new(into: K, element: A) -> Append<A, K> {
        Append {
            lens: into,
            element,
        }
    }
}

// Append to Vec<A>
impl<S: Data, A: Data + Clone, K: Lens<S, Vec<A>>> Update<S> for Append<A, K> {
    fn apply(&mut self, data: &mut S) -> Change {
        let elem = self.element.clone();
        self.lens.with_mut(data, |v| v.push(elem));
        Change::replacement() // TODO more precise description ?
    }

    fn address(&self) -> Option<S::Address> {
        self.lens.address()
    }
}

//--------------------------------------------------------------------------------------------------
/// Insert operation
pub struct Insert<A: Data, K, I: Clone> {
    lens: K,
    index: I,
    element: A,
}

impl<A: Data, K, I: Clone> Insert<A, K, I> {
    pub fn new(into: K, index: I, element: A) -> Insert<A, K, I> {
        Insert {
            lens: into,
            index,
            element,
        }
    }
}

// Insert into Vec<A>
impl<S: Data, A: Data + Clone, K: Lens<S, Vec<A>>> Update<S> for Insert<A, K, usize> {
    fn apply(&mut self, data: &mut S) -> Change {
        let index = self.index;
        let elem = self.element.clone();
        self.lens.with_mut(data, |v| v.insert(index, elem));
        Change::replacement()
    }

    fn address(&self) -> Option<S::Address> {
        self.lens.address()
    }
}
