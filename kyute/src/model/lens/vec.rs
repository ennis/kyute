//! `Vec<T>` lenses
use crate::model::{Collection, Data, IndexAddress};
use crate::model::{Lens, LensCompose, LensIndexExt};
use std::fmt;
use std::fmt::Debug;
use std::marker::PhantomData;

/// A lens that looks at a particular item in a vector.
///
/// It implements `Lens<Vec<T>,T>`.
#[derive(Debug)]
pub struct VecLens<T> {
    index: usize,
    _phantom: PhantomData<T>,
}

// #26925
impl<T> Clone for VecLens<T> {
    fn clone(&self) -> Self {
        VecLens {
            index: self.index,
            _phantom: PhantomData,
        }
    }
}

impl<T> VecLens<T> {
    pub fn new(index: usize) -> VecLens<T> {
        VecLens {
            index,
            _phantom: PhantomData,
        }
    }
}

pub struct VecAddress<T: Data>(usize, Option<T::Address>);

// #26925 impl (when is this going to be fixed?)
impl<T: Data> Clone for VecAddress<T> {
    fn clone(&self) -> Self {
        VecAddress(self.0, self.1.clone())
    }
}

impl<T: Data> fmt::Debug for VecAddress<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{}]", self.0)?;
        if let Some(addr) = &self.1 {
            write!(f, ".{:?}", addr)?;
        }
        Ok(())
    }
}

impl<T: Data> PartialEq for VecAddress<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0 && self.1 == other.1
    }
}

impl<T: Data> Eq for VecAddress<T> {}

impl<T: Data> IndexAddress for VecAddress<T> {
    type Element = T;
    type Index = usize;

    fn index(&self) -> Self::Index {
        self.0
    }

    fn rest(&self) -> &Option<T::Address> {
        &self.1
    }
}

impl<T: Data> Data for Vec<T> {
    type Address = VecAddress<T>;
}

impl<T: Data> Collection for Vec<T> {
    type Index = usize;
    type Element = T;
    type ElementLens = VecLens<T>;

    fn at_index(index: usize) -> VecLens<T> {
        VecLens {
            index,
            _phantom: PhantomData,
        }
    }

    fn get_at(&self, index: Self::Index) -> Option<&Self::Element> {
        self.get(index)
    }

    fn box_iter<'a>(&'a self) -> Box<dyn Iterator<Item = &'a Self::Element> + 'a> {
        Box::new(self.iter())
    }
}

impl<T: Data> Lens<Vec<T>, T> for VecLens<T> {
    fn address(&self) -> Option<VecAddress<T>>
    where
        Vec<T>: Data,
    {
        Some(VecAddress(self.index, None))
    }

    /// Concatenate addresses.
    fn concat<K, C: Data>(&self, rhs: &K) -> Option<VecAddress<T>>
    where
        K: Lens<T, C>,
        Vec<T>: Data,
    {
        Some(VecAddress(self.index, rhs.address()))
    }

    fn with<R, F: FnOnce(&T) -> R>(&self, data: &Vec<T>, f: F) -> R {
        f(&data[self.index])
    }

    fn with_mut<R, F: FnOnce(&mut T) -> R>(&self, data: &mut Vec<T>, f: F) -> R {
        f(&mut data[self.index])
    }

    fn try_with<R, F: FnOnce(&T) -> R>(&self, data: &Vec<T>, f: F) -> Option<R> {
        data.get(self.index).map(f)
    }

    fn try_with_mut<R, F: FnOnce(&mut T) -> R>(&self, data: &mut Vec<T>, f: F) -> Option<R> {
        data.get_mut(self.index).map(f)
    }

    fn unprefix(&self, addr: <Vec<T> as Data>::Address) -> Option<Option<T::Address>>
    where
        T: Data,
    {
        if self.index == addr.0 {
            Some(addr.1)
        } else {
            None
        }
    }
}

impl<A, T, K> LensIndexExt<A, Vec<T>> for K
where
    A: Data,
    T: Data,
    K: Clone + Lens<A, Vec<T>>,
{
    type Output = T;
    type Lens = LensCompose<K, VecLens<T>, Vec<T>>;

    fn index(self, i: usize) -> Self::Lens {
        LensCompose(self, VecLens::new(i), PhantomData)
    }
}

/*
/// Lookup by sorted key (linear search) in a sorted vector.
#[derive(Copy,Clone,Debug)]
pub struct LinearSearchVecLookupLens<T: Entity>(T::Key);

impl<T: Entity> Lens for LinearSearchVecLookupLens<T> {
    type Root = Vec<T>;
    type Leaf = T;

    fn address(&self) -> _ {
        unimplemented!()
    }

    fn concat<L, U>(&self, rhs: L) -> Vec where L: Lens<Root=Self::Leaf, Leaf=U>, U: Data {
        unimplemented!()
    }

    fn get<'a>(&self, data: &'a Self::Root) -> &'a Self::Leaf {
        data.iter().find(|&item| item.key() == self.0).unwrap()
    }

    fn get_mut<'a>(&self, data: &'a mut Self::Root) -> &'a mut Self::Leaf {
        data.iter_mut().find(|item| item.key() == self.0).unwrap()
    }

    fn try_get<'a>(&self, data: &'a Self::Root) -> Option<&'a Self::Leaf> {
        data.iter().find(|&item| item.key() == self.0)
    }

    fn try_get_mut<'a>(&self, data: &'a mut Self::Root) -> Option<&'a mut Self::Leaf> {
        data.iter_mut().find(|item| item.key() == self.0)
    }
}

impl<T, L> LensLookupExt for L where
    T: Entity,
    L: Lens<Leaf=Vec<T>>,
{
    type Key = T::Key;
    type Output = LinearSearchVecLookupLens<T>;

    fn by_key(&self, key: Self::Key) -> LinearSearchVecLookupLens<T> {
        LinearSearchVecLookupLens(key)
    }
}
*/
