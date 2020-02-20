use std::fmt::Debug;
use std::hash::Hash;
use std::marker::PhantomData;

use crate::model::{Change, Data, Revision};

pub mod vec;
pub use vec::VecAddress;
pub use vec::VecLens;

/// Trait implemented by "lens" types, which act like a reified accessor for
/// some "child" part of type U of a "parent" object of type T.

// TODO We use closures to access the target value (B) because it might live only for the duration
// of the call to `with`. This is the case when the target value is not 'borrowable' within the
// source but is rather synthesized on-the-fly within the method.
//
// This works, but this "closure-passing" style has syntactical implications in methods:
// a lot of view classes store those lenses in a struct, and end up doing something like this:
// ```
// self.some_lens.with(s, |s| { self.other.do_thing() })
// ```
// With `do_thing()` needing a mut ref to `self.other`. This fails at borrowck since the whole
// self is borrowed within the closure. There is an RFC that would enable precise captures in
// closures, but has not seen progress for a while.
//
// In the meantime, this means that those methods must be rewritten as such:
// ```
// let mut other = &mut self.other;
// self.some_lens.with(s, |s| other.do_thing());
// ```
// Which is needlessly noisy.
// A better option would be to return a reference directly but that would not work with synthesized
// data.
//
// The debate here is whether we should make a difference between lenses that just borrow a part of
// the parent data structure and lenses that compute new values.

pub trait Lens<A: Data + ?Sized, B: Data + ?Sized> {
    // --- Accessors ---
    fn with<R, F: FnOnce(&B) -> R>(&self, data: &A, f: F) -> R;
    fn with_mut<R, F: FnOnce(&mut B) -> R>(&self, data: &mut A, f: F) -> R;
    fn try_with<R, F: FnOnce(&B) -> R>(&self, data: &A, f: F) -> Option<R>;
    fn try_with_mut<R, F: FnOnce(&mut B) -> R>(&self, data: &mut A, f: F) -> Option<R>;

    // --- Composition ---

    /// Returns the address of this lens within A:
    /// it's a value of the associated type `A::Address` that represents the path, within the parent
    /// object, to the part that the lens is watching (i.e. the path from &A to &B).
    /// Typically, this represents a sequence of field accesses and indexing operations on the
    /// parent to get to the part.
    fn address(&self) -> Option<A::Address>;

    /// Concatenate addresses.
    fn concat<K, C: Data>(&self, rhs: &K) -> Option<A::Address>
    where
        K: Lens<B, C>;

    /// Lens composition.
    fn compose<K, C: Data>(self, rhs: K) -> LensCompose<Self, K, B>
    where
        Self: Sized,
        K: Lens<B, C>,
    {
        LensCompose(self, rhs, PhantomData)
    }

    /// Removes the prefix specified by this lens to the address.
    fn unprefix(&self, addr: A::Address) -> Option<Option<B::Address>>;

    /// Transforms a revision in the source type to a revision in the target type.
    fn focus<R, F: FnOnce(&Revision<B>) -> R>(&self, src: &Revision<A>, f: F) -> Option<R> {
        // replace source -> replace destination
        if let Change::Replace = src.change {
            self.with(src.data, |data| {
                Some(f(&Revision {
                    change: src.change,
                    addr: None,
                    data,
                }))
            })
        } else {
            // change in source -> change in destination only if the address of the lens
            // correctly unprefixes the address of the revision (i.e. the revision happens in a part
            // of the object that the lens is watching).
            src.addr.clone().and_then(|addr| {
                self.unprefix(addr).map(|suffix| {
                    self.with(src.data, |data| {
                        f(&Revision {
                            addr: suffix,
                            change: src.change,
                            data,
                        })
                    })
                })
            })
        }
    }
}

pub trait LensExt<A: Data, B: Data>: Lens<A, B> {
    fn get_if_changed(&self, src: &Revision<A>) -> Option<B>
    where
        B: Clone,
    {
        self.focus(src, |s| s.data.clone())
    }

    fn get(&self, src: &A) -> B
    where
        B: Clone,
    {
        self.with(src, |x| x.clone())
    }
}

impl<A: Data, B: Data, L: Lens<A, B>> LensExt<A, B> for L {}

/// Indexing operations
pub trait LensIndexExt<A: Data, B: Data>: Lens<A, B> {
    type Output: Data;
    type Lens: Lens<A, Self::Output>;
    fn index(self, i: usize) -> Self::Lens;
}

/// Key lookup operations
pub trait LensLookupExt<A: Data, B: Data>: Lens<A, B> {
    type Key: Copy + Clone + Debug + Eq + PartialEq + Hash;
    type Output: Data;
    type Lens: Lens<A, Self::Output>;
    fn by_key(&self, key: Self::Key) -> Self::Lens;
}

/// Identity lens.
#[derive(Copy, Clone, Debug)]
pub struct IdentityLens;

impl<A: Data> Lens<A, A> for IdentityLens {
    // need to convert everything into closure-passing style
    fn with<R, F: FnOnce(&A) -> R>(&self, data: &A, f: F) -> R {
        f(data)
    }

    fn with_mut<R, F: FnOnce(&mut A) -> R>(&self, data: &mut A, f: F) -> R {
        f(data)
    }

    fn try_with<R, F: FnOnce(&A) -> R>(&self, data: &A, f: F) -> Option<R> {
        Some(f(data))
    }

    fn try_with_mut<R, F: FnOnce(&mut A) -> R>(&self, data: &mut A, f: F) -> Option<R> {
        Some(f(data))
    }

    fn address(&self) -> Option<A::Address> {
        None
    }

    fn concat<K, C: Data>(&self, rhs: &K) -> Option<A::Address>
    where
        K: Lens<A, C>,
    {
        rhs.address()
    }

    fn unprefix(&self, addr: A::Address) -> Option<Option<A::Address>> {
        Some(Some(addr))
    }
}

/// Lens composition by reference: combines `&Lens<U,V>` and `&Lens<V,W>` to `Lens<U,W>`.
#[derive(Debug)]
pub struct RefLensCompose<'a, K, L, B>(pub &'a K, pub &'a L, pub PhantomData<B>);

// #26925
impl<'a, K: Clone, L: Clone, B> Clone for RefLensCompose<'a, K, L, B> {
    fn clone(&self) -> Self {
        RefLensCompose(self.0, self.1, PhantomData)
    }
}

impl<'a, K, L, A: Data, B: Data, C: Data> Lens<A, C> for RefLensCompose<'a, K, L, B>
where
    K: Lens<A, B>,
    L: Lens<B, C>,
{
    fn address(&self) -> Option<A::Address> {
        self.0.concat(self.1)
    }

    fn concat<M, D: Data>(&self, rhs: &M) -> Option<A::Address>
    where
        M: Lens<C, D>,
    {
        // XXX what's the complexity of this?
        self.0.concat(&RefLensCompose(self.1, rhs, PhantomData))
    }

    fn with<R, F: FnOnce(&C) -> R>(&self, data: &A, f: F) -> R {
        self.0.with(data, |data| self.1.with(data, |data| f(data)))
    }

    fn with_mut<R, F: FnOnce(&mut C) -> R>(&self, data: &mut A, f: F) -> R {
        self.0
            .with_mut(data, |data| self.1.with_mut(data, |data| f(data)))
    }

    fn try_with<R, F: FnOnce(&C) -> R>(&self, data: &A, f: F) -> Option<R> {
        self.0
            .try_with(data, |data| self.1.try_with(data, |data| f(data)))
            .flatten()
    }

    fn try_with_mut<R, F: FnOnce(&mut C) -> R>(&self, data: &mut A, f: F) -> Option<R> {
        self.0
            .try_with_mut(data, |data| self.1.try_with_mut(data, |data| f(data)))
            .flatten()
    }

    fn compose<M, D: Data>(self, rhs: M) -> LensCompose<Self, M, C>
    where
        M: Lens<C, D>,
    {
        LensCompose(self, rhs, PhantomData)
    }

    fn unprefix(&self, addr: A::Address) -> Option<Option<C::Address>> {
        self.0
            .unprefix(addr)
            .and_then(|addr| addr.and_then(|addr| self.1.unprefix(addr)))
    }
}

/// Lens composition: combines `Lens<U,V>` and `Lens<V,W>` to `Lens<U,W>`.
///
/// Equivalent to applying two lenses in succession.
#[derive(Debug)]
pub struct LensCompose<K, L, B: ?Sized>(pub K, pub L, pub PhantomData<B>);

// #26925
impl<K: Clone, L: Clone, B: ?Sized> Clone for LensCompose<K, L, B> {
    fn clone(&self) -> Self {
        LensCompose(self.0.clone(), self.1.clone(), PhantomData)
    }
}

// LensCompose<K, VecLens<T>, std::vec::Vec<T>>
// K: Lens<A, Vec<T>>,
// VecLens<T>:

impl<K, L, A: Data, B: Data, C: Data> Lens<A, C> for LensCompose<K, L, B>
where
    K: Lens<A, B>,
    L: Lens<B, C>,
{
    fn address(&self) -> Option<A::Address> {
        self.0.concat(&self.1)
    }

    fn concat<M, D: Data>(&self, rhs: &M) -> Option<A::Address>
    where
        M: Lens<C, D>,
    {
        // XXX what's the complexity of this?
        self.0.concat(&RefLensCompose(&self.1, rhs, PhantomData))
    }

    fn with<R, F: FnOnce(&C) -> R>(&self, data: &A, f: F) -> R {
        self.0.with(data, |data| self.1.with(data, |data| f(data)))
    }

    fn with_mut<R, F: FnOnce(&mut C) -> R>(&self, data: &mut A, f: F) -> R {
        self.0
            .with_mut(data, |data| self.1.with_mut(data, |data| f(data)))
    }

    fn try_with<R, F: FnOnce(&C) -> R>(&self, data: &A, f: F) -> Option<R> {
        self.0
            .try_with(data, |data| self.1.try_with(data, |data| f(data)))
            .flatten()
    }

    fn try_with_mut<R, F: FnOnce(&mut C) -> R>(&self, data: &mut A, f: F) -> Option<R> {
        self.0
            .try_with_mut(data, |data| self.1.try_with_mut(data, |data| f(data)))
            .flatten()
    }

    fn compose<M, D: Data>(self, rhs: M) -> LensCompose<Self, M, C>
    where
        M: Lens<C, D>,
    {
        LensCompose(self, rhs, PhantomData)
    }

    fn unprefix(&self, addr: A::Address) -> Option<Option<C::Address>> {
        self.0
            .unprefix(addr)
            .and_then(|addr| addr.and_then(|addr| self.1.unprefix(addr)))
    }
}

/// Unit lens: returns () always
#[derive(Copy, Clone, Debug)]
pub struct UnitLens;

impl<T: Data> Lens<T, ()> for UnitLens {
    fn address(&self) -> Option<T::Address> {
        None
    }

    fn concat<K, C>(&self, _rhs: &K) -> Option<T::Address>
    where
        K: Lens<(), C>,
        C: Data,
    {
        None
    }

    fn with<R, F: FnOnce(&()) -> R>(&self, _data: &T, f: F) -> R {
        f(&())
    }

    fn with_mut<R, F: FnOnce(&mut ()) -> R>(&self, _data: &mut T, f: F) -> R {
        f(&mut ())
    }

    fn try_with<R, F: FnOnce(&()) -> R>(&self, _data: &T, f: F) -> Option<R> {
        Some(f(&()))
    }

    fn try_with_mut<R, F: FnOnce(&mut ()) -> R>(&self, _data: &mut T, f: F) -> Option<R> {
        Some(f(&mut ()))
    }

    fn unprefix(&self, _addr: T::Address) -> Option<Option<<() as Data>::Address>> {
        // never unprefixed by anything
        None
    }
}

/*
/// Composition
pub trait LensExt: Lens {
    fn compose<K>(&self, other: K) -> LensCompose<Self, K>
    where
        K: Lens<Root = <Self as Lens>::Leaf>,
    {
        LensCompose(self.clone(), other)
    }
}

impl<L: Lens> LensExt for L {}*/

/*
/// Macro for implementing a lens type that accesses a field of a struct.
#[macro_export]
macro_rules! impl_field_lens {
    ($v:vis $lens:ident [ $t:ty => $u:ty ] [ $f:ident ( $index:expr ) ]) => {
        #[derive(Copy,Clone,Debug)]
        $v struct $lens;
        impl $crate::lens::Lens for $lens {
            type Root = $t;
            type Leaf = $u;

            fn path(&self) -> $crate::lens::Path<$t, $u> {
                $crate::lens::Path::field($index)
            }

            fn get<'a>(&self, data: &'a $t) -> &'a $u {
                &data.$f
            }

            fn get_mut<'a>(&self, data: &'a mut $t) -> &'a mut $u {
                &mut data.$f
            }
        }
    };
}
*/

/// Value-synthesizing lens
impl<A: Data, B: Data, G> Lens<A, B> for G
where
    G: for<'a> Fn(&'a A) -> B,
{
    fn address(&self) -> Option<A::Address> {
        None
    }

    fn concat<K, C>(&self, _rhs: &K) -> Option<A::Address>
    where
        K: Lens<B, C>,
        C: Data,
    {
        None
    }

    fn with<R, F: FnOnce(&B) -> R>(&self, data: &A, f: F) -> R {
        f(&(self)(data))
    }

    fn with_mut<R, F: FnOnce(&mut B) -> R>(&self, data: &mut A, f: F) -> R {
        f(&mut (self)(data))
    }

    fn try_with<R, F: FnOnce(&B) -> R>(&self, data: &A, f: F) -> Option<R> {
        Some(f(&(self)(data)))
    }

    fn try_with_mut<R, F: FnOnce(&mut B) -> R>(&self, data: &mut A, f: F) -> Option<R> {
        Some(f(&mut (self)(data)))
    }

    fn unprefix(&self, _addr: A::Address) -> Option<Option<B::Address>> {
        Some(None)
    }
}
