//! Atomic counters
use std::{
    num::NonZeroU64,
    sync::atomic::{AtomicU64, Ordering},
};

// Adapted from druid: see license info
/// An incrementing counter for generating unique ids.
///
/// This can be used safely from multiple threads.
///
/// The counter will overflow if `next()` is called 2^64 - 2 times.
/// If this is possible for your application, and reuse would be undesirable,
/// use something else.
pub struct Counter(AtomicU64);

impl Counter {
    /// Create a new counter.
    pub const fn new() -> Counter {
        Counter(AtomicU64::new(1))
    }

    /// Creates a new counter with a given starting value.
    ///
    /// # Safety
    ///
    /// The value must not be zero.
    pub const unsafe fn new_unchecked(init: u64) -> Counter {
        Counter(AtomicU64::new(init))
    }

    /// Return the next value.
    pub fn next(&self) -> u64 {
        self.0.fetch_add(1, Ordering::Relaxed)
    }

    /// Return the next value, as a `NonZeroU64`.
    pub fn next_nonzero(&self) -> NonZeroU64 {
        // safe because our initial value is 1 and can only be incremented.
        unsafe { NonZeroU64::new_unchecked(self.0.fetch_add(1, Ordering::Relaxed)) }
    }
}
