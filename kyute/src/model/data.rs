use crate::model::Lens;
use std::fmt::Debug;
use std::hash::Hash;

/// Issue: there is no blanket impl for Data, so every type that is used in the model
/// must implement Data. This is a problem, because due to orphan rules it's impossible for a user
/// of the veda crate to implement the Data trait for foreign data structure types.
/// (the user could only use the data structures/data types provided by veda)
///
/// Even so, a blanket impl forall T would not be very useful by itself, and would require specialization
/// to provide fine-grained addressability to vecs and other structures.
pub trait Data: Clone {
    /// the type of a value that can uniquely identify a component part of the data.
    type Address: Clone + Debug + Eq + PartialEq;
}

impl<T: Clone + 'static> Data for T {
    default type Address = ();
}

pub trait Identifiable: Data {
    type Id: Clone + Debug + Eq + PartialEq + Hash;
    fn id(&self) -> Self::Id;
}

/// Trait to handle different types of collections generically. Used in views.
// Note: putting a `Collection` bound is somehow not sufficient to imply the additional bounds on `Self::Address`,
// so you have to put those yourself. (see https://github.com/rust-lang/rust/issues/52662#issuecomment-473678551)
// This will supposedly be fixed by "implied bounds" (https://github.com/rust-lang/rust/issues/44491)
pub trait Collection: Data
where
    Self::Address: IndexAddress<Element = Self::Element, Index = Self::Index>,
{
    type Index: Clone + Debug + Eq + PartialEq;
    type Element: Data;
    type ElementLens: Lens<Self, Self::Element>;

    /// Creates a new lens that looks at the element at the given index.
    fn at_index(index: Self::Index) -> Self::ElementLens;

    /// Iterates over the collection.
    fn box_iter<'a>(&'a self) -> Box<dyn Iterator<Item = &'a Self::Element> + 'a>;

    fn get_at(&self, index: Self::Index) -> Option<&Self::Element> {
        unimplemented!()
        //Self::at_index(index).try_get(self)
    }
}

pub trait IndexAddress: Clone + Debug + Eq + PartialEq {
    type Element: Data;
    type Index;
    fn index(&self) -> Self::Index;
    fn rest(&self) -> &Option<<Self::Element as Data>::Address>;
}
