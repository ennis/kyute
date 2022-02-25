// Copyright 2019 The Druid Authors.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Traits for handling value types.

use crate::{Offset, Point, Rect, SideOffsets};
use std::{ptr, rc::Rc, sync::Arc};
//use crate::style::StyleSet;

pub trait Data: Clone + 'static {
    fn same(&self, other: &Self) -> bool;
}

/// An impl of `Data` suitable for simple types.
///
/// The `same` method is implemented with equality, so the type should
/// implement `Eq` at least.
macro_rules! impl_data_simple {
    ($t:ty) => {
        impl Data for $t {
            fn same(&self, other: &Self) -> bool {
                self == other
            }
        }
    };
}

// Standard library impls
impl_data_simple!(i8);
impl_data_simple!(i16);
impl_data_simple!(i32);
impl_data_simple!(i64);
impl_data_simple!(i128);
impl_data_simple!(isize);
impl_data_simple!(u8);
impl_data_simple!(u16);
impl_data_simple!(u32);
impl_data_simple!(u64);
impl_data_simple!(u128);
impl_data_simple!(usize);
impl_data_simple!(char);
impl_data_simple!(bool);
impl_data_simple!(std::path::PathBuf);
impl_data_simple!(std::num::NonZeroI8);
impl_data_simple!(std::num::NonZeroI16);
impl_data_simple!(std::num::NonZeroI32);
impl_data_simple!(std::num::NonZeroI64);
impl_data_simple!(std::num::NonZeroI128);
impl_data_simple!(std::num::NonZeroIsize);
impl_data_simple!(std::num::NonZeroU8);
impl_data_simple!(std::num::NonZeroU16);
impl_data_simple!(std::num::NonZeroU32);
impl_data_simple!(std::num::NonZeroU64);
impl_data_simple!(std::num::NonZeroU128);
impl_data_simple!(std::num::NonZeroUsize);
impl_data_simple!(std::time::SystemTime);
impl_data_simple!(std::time::Instant);
impl_data_simple!(std::time::Duration);
impl_data_simple!(std::io::ErrorKind);
impl_data_simple!(std::net::Ipv4Addr);
impl_data_simple!(std::net::Ipv6Addr);
impl_data_simple!(std::net::SocketAddrV4);
impl_data_simple!(std::net::SocketAddrV6);
impl_data_simple!(std::net::IpAddr);
impl_data_simple!(std::net::SocketAddr);
impl_data_simple!(std::ops::RangeFull);
//impl_data_simple!(Color);
impl_data_simple!(SideOffsets);
impl_data_simple!(Rect);
impl_data_simple!(Point);
impl_data_simple!(Offset);

//TODO: remove me!?
impl_data_simple!(String);

impl Data for &'static str {
    fn same(&self, other: &Self) -> bool {
        ptr::eq(*self, *other)
    }
}

impl Data for f32 {
    fn same(&self, other: &Self) -> bool {
        self.to_bits() == other.to_bits()
    }
}

impl Data for f64 {
    fn same(&self, other: &Self) -> bool {
        self.to_bits() == other.to_bits()
    }
}

impl<T: ?Sized + 'static> Data for Arc<T> {
    fn same(&self, other: &Self) -> bool {
        Arc::ptr_eq(self, other)
    }
}

impl<T: ?Sized + 'static> Data for std::sync::Weak<T> {
    fn same(&self, other: &Self) -> bool {
        std::sync::Weak::ptr_eq(self, other)
    }
}

impl<T: ?Sized + 'static> Data for Rc<T> {
    fn same(&self, other: &Self) -> bool {
        Rc::ptr_eq(self, other)
    }
}

impl<T: ?Sized + 'static> Data for std::rc::Weak<T> {
    fn same(&self, other: &Self) -> bool {
        std::rc::Weak::ptr_eq(self, other)
    }
}

impl<T: Data> Data for Option<T> {
    fn same(&self, other: &Self) -> bool {
        match (self, other) {
            (Some(a), Some(b)) => a.same(b),
            (None, None) => true,
            _ => false,
        }
    }
}

impl<T: Data, U: Data> Data for Result<T, U> {
    fn same(&self, other: &Self) -> bool {
        match (self, other) {
            (Ok(a), Ok(b)) => a.same(b),
            (Err(a), Err(b)) => a.same(b),
            _ => false,
        }
    }
}

impl Data for () {
    fn same(&self, _other: &Self) -> bool {
        true
    }
}

impl<T0: Data> Data for (T0,) {
    fn same(&self, other: &Self) -> bool {
        self.0.same(&other.0)
    }
}

impl<T0: Data, T1: Data> Data for (T0, T1) {
    fn same(&self, other: &Self) -> bool {
        self.0.same(&other.0) && self.1.same(&other.1)
    }
}

impl<T0: Data, T1: Data, T2: Data> Data for (T0, T1, T2) {
    fn same(&self, other: &Self) -> bool {
        self.0.same(&other.0) && self.1.same(&other.1) && self.2.same(&other.2)
    }
}

impl<T0: Data, T1: Data, T2: Data, T3: Data> Data for (T0, T1, T2, T3) {
    fn same(&self, other: &Self) -> bool {
        self.0.same(&other.0) && self.1.same(&other.1) && self.2.same(&other.2) && self.3.same(&other.3)
    }
}

impl<T0: Data, T1: Data, T2: Data, T3: Data, T4: Data> Data for (T0, T1, T2, T3, T4) {
    fn same(&self, other: &Self) -> bool {
        self.0.same(&other.0)
            && self.1.same(&other.1)
            && self.2.same(&other.2)
            && self.3.same(&other.3)
            && self.4.same(&other.4)
    }
}

impl<T0: Data, T1: Data, T2: Data, T3: Data, T4: Data, T5: Data> Data for (T0, T1, T2, T3, T4, T5) {
    fn same(&self, other: &Self) -> bool {
        self.0.same(&other.0)
            && self.1.same(&other.1)
            && self.2.same(&other.2)
            && self.3.same(&other.3)
            && self.4.same(&other.4)
            && self.5.same(&other.5)
    }
}

impl<T: 'static + ?Sized> Data for std::marker::PhantomData<T> {
    fn same(&self, _other: &Self) -> bool {
        // zero-sized types
        true
    }
}

impl<T: 'static> Data for std::mem::Discriminant<T> {
    fn same(&self, other: &Self) -> bool {
        *self == *other
    }
}

impl<T: 'static + ?Sized + Data> Data for std::mem::ManuallyDrop<T> {
    fn same(&self, other: &Self) -> bool {
        (&**self).same(&**other)
    }
}

impl<T: Data> Data for std::num::Wrapping<T> {
    fn same(&self, other: &Self) -> bool {
        self.0.same(&other.0)
    }
}

impl<T: Data> Data for std::ops::Range<T> {
    fn same(&self, other: &Self) -> bool {
        self.start.same(&other.start) && self.end.same(&other.end)
    }
}

impl<T: Data> Data for std::ops::RangeFrom<T> {
    fn same(&self, other: &Self) -> bool {
        self.start.same(&other.start)
    }
}

impl<T: Data> Data for std::ops::RangeInclusive<T> {
    fn same(&self, other: &Self) -> bool {
        self.start().same(other.start()) && self.end().same(other.end())
    }
}

impl<T: Data> Data for std::ops::RangeTo<T> {
    fn same(&self, other: &Self) -> bool {
        self.end.same(&other.end)
    }
}

impl<T: Data> Data for std::ops::RangeToInclusive<T> {
    fn same(&self, other: &Self) -> bool {
        self.end.same(&other.end)
    }
}

impl<T: Data> Data for std::ops::Bound<T> {
    fn same(&self, other: &Self) -> bool {
        use std::ops::Bound::*;
        match (self, other) {
            (Included(t1), Included(t2)) if t1.same(t2) => true,
            (Excluded(t1), Excluded(t2)) if t1.same(t2) => true,
            (Unbounded, Unbounded) => true,
            _ => false,
        }
    }
}

impl<T: Data, const N: usize> Data for [T; N] {
    fn same(&self, other: &Self) -> bool {
        self.iter().zip(other.iter()).all(|(a, b)| a.same(b))
    }
}

/*impl Data for TextFormat {
    fn same(&self, other: &Self) -> bool {
        self.as_raw().eq(other.as_raw())
    }
}

impl Data for TextLayout {
    fn same(&self, other: &Self) -> bool {
        self.as_raw().eq(other.as_raw())
    }
}*/

/*
#[cfg(test)]
mod test {
    use super::Data;
    use test_env_log::test;

    #[test]
    fn array_data() {
        let input = [1u8, 0, 0, 1, 0];
        assert!(input.same(&[1u8, 0, 0, 1, 0]));
        assert!(!input.same(&[1u8, 1, 0, 1, 0]));
    }

    #[test]
    fn static_strings() {
        let first = "test";
        let same = "test";
        let second = "test2";
        assert!(!Data::same(&first, &second));
        assert!(Data::same(&first, &first));
        // although these are different, the compiler will notice that the string "test" is common,
        // intern it, and reuse it for all "text" `&'static str`s.
        assert!(Data::same(&first, &same));
    }
}*/
