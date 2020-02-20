//! terminology:
//! TODO not up to date
//!
//! # Object
//! an object
//!
//! # Aggregate
//! An object constituted of one or more sub-objects. This can be a structure, a collection, etc.
//!
//! # Lens
//! A lens is an object that represents a way to access a component of some complex aggregate type `U`.
//!
//! Concretely, a lens over a type `U`, given a reference to an aggregate of type `U`,
//! provides access to an object of type `V` stored within the aggregate
//! (and potentially deep within the structure of `U`).
//! `U` is called the _root type_ and `V` is called the _leaf type_.
//! For all intents and purposes, you can see lenses as a generic way to represent a sequence of
//! field accesses, indexing operations, and lookups that access an object
//! stored arbitrarily deep into an object of type `U`.
//! (e.g. `.field.collection[42].field2.map[21]`).
//!
//! The _lens_ term is borrowed from the concept of _functional lenses_ in some languages
//! (https://www.schoolofhaskell.com/school/to-infinity-and-beyond/pick-of-the-week/basic-lensing).
//! A possible synonym of lens could be "accessor".
//!
//!
//! # Lens path
//! A value that uniquely identifies a lens over a type `U`. If two lenses `K` and `L` over the same
//! type `U` have equal paths, then they represent access to the same component within `U`.
//! Lens paths can be decomposed into a sequence of _component indices_, with each index representing one
//! primitive component access operation (field access, indexing operation, or lookup).
//!
//! Component indices are u64 integers. Depending on the type of object that the component index
//! applies to, the index can represent either a structure field, an index in a linear collection,
//! or a key to access an element in an associative container.
//!
mod change;
mod data;
mod lens;
mod state;
pub mod update;

pub use change::Change;
pub use change::CollectionChanges;
pub use change::Revision;
pub use data::Collection;
pub use data::Data;
pub use data::Identifiable;
pub use data::IndexAddress;
pub use lens::IdentityLens;
pub use lens::Lens;
pub use lens::LensCompose;
pub use lens::LensExt;
pub use lens::LensIndexExt;
pub use lens::LensLookupExt;
pub use lens::UnitLens;
pub use state::State;
pub use state::Watcher;
