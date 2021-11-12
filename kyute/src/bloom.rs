//! Bloom filter implementation.
//! To optimize delivery of events to a specific node in the hierarchy, identified by ID.
//! Inspired by druid's approach (https://github.com/linebender/druid/blob/a08ea03389a38d9f1024267153491b6070cab97c/druid/src/bloom.rs)

// Contains portions of code adapted from the druid GUI framework:
// Copyright 2020 The Druid Authors.
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

use fnv::FnvHasher;
use std::{
    fmt,
    hash::{Hash, Hasher},
    marker::PhantomData,
};

#[derive(Copy, Clone)]
pub(crate) struct Bloom<T> {
    bits: u64,
    _phantom: PhantomData<*const T>,
}

// the 'offset_basis' for the fnv-1a hash algorithm.
// see http://www.isthe.com/chongo/tech/comp/fnv/index.html#FNV-param
//
// The first of these is the one described in the algorithm, the second is random.
const OFFSET_ONE: u64 = 0xcbf2_9ce4_8422_2325;
const OFFSET_TWO: u64 = 0xe10_3ad8_2dad_8028;
const NUM_BITS: u64 = 64;

fn bloom_hash<T: Hash>(item: &T, seed: u64) -> u64 {
    let mut h = FnvHasher::with_key(seed);
    item.hash(&mut h);
    h.finish()
}

fn bloom_bitmask<T: Hash>(item: &T) -> u64 {
    let h1 = bloom_hash(item, OFFSET_ONE);
    let h2 = bloom_hash(item, OFFSET_TWO);
    (1 << (h1 % NUM_BITS)) | (1 << (h2 % NUM_BITS))
}

impl<T: Hash> Bloom<T> {
    pub fn new() -> Bloom<T> {
        Bloom {
            bits: 0,
            _phantom: PhantomData,
        }
    }

    /// Adds an item to the filter.
    pub fn add(&mut self, item: &T) {
        self.bits |= bloom_bitmask(item);
    }

    /// Combines the items of the filter with another.
    pub fn extend(&mut self, other: &Bloom<T>) {
        self.bits |= other.bits
    }

    /// Removes all entries from the filter.
    pub fn clear(&mut self) {
        self.bits = 0;
    }

    /// Returns whether the specified item may have been added to the filter.
    pub fn may_contain(&self, item: &T) -> bool {
        let mask = bloom_bitmask(item);
        self.bits & mask == mask
    }
}

impl<T> Default for Bloom<T> {
    fn default() -> Self {
        Bloom {
            bits: 0,
            _phantom: PhantomData,
        }
    }
}

impl<T> fmt::Debug for Bloom<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:064b}", self.bits)
    }
}
