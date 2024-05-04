use crate::counter::Counter;
use std::{fmt, num::NonZeroU32};

/// Widget ID.
///
/// Identifies a widget among its siblings.
#[derive(Clone, Copy, Hash, PartialEq, Eq, Ord, PartialOrd)]
#[repr(transparent)]
pub struct WidgetId(NonZeroU32);

const WIDGET_ID_COUNTER: Counter = Counter::new();

impl Default for WidgetId {
    fn default() -> Self {
        Self::ANONYMOUS
    }
}

impl WidgetId {
    /// ID used for anonymous elements.
    ///
    /// IDs are only needed by elements that need to receive events.
    /// If the element doesn't need to receive events, it can use this anonymous ID instead of
    /// generating a unique ID.
    pub const ANONYMOUS: WidgetId = WidgetId(NonZeroU32::MAX);

    /// Returns whether the ID is the anonymous ID.
    pub fn is_anonymous(self) -> bool {
        self == Self::ANONYMOUS
    }

    /// Converts the ID to a `u32` value.
    pub fn to_u32(self) -> u32 {
        self.0.get()
    }

    pub fn next() -> WidgetId {
        WidgetId(NonZeroU32::new(WIDGET_ID_COUNTER.next() as u32).unwrap())
    }
}

impl fmt::Debug for WidgetId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:08X}", self.to_u32())
    }
}
