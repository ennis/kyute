use crate::composable;
use kyute_compose::{cache_cx, CallId};
use std::fmt;

pub struct WidgetIdDebug(Option<WidgetId>);
impl fmt::Debug for WidgetIdDebug {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.0 {
            None => write!(f, "(anonymous)"),
            Some(id) => {
                write!(f, "{:?}", id)
            }
        }
    }
}

/// ID of a node in the tree.
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
#[repr(transparent)]
pub struct WidgetId(CallId);

impl WidgetId {
    pub(crate) fn from_call_id(call_id: CallId) -> WidgetId {
        WidgetId(call_id)
    }

    #[composable]
    pub fn here() -> WidgetId {
        WidgetId(cache_cx::current_call_id())
    }

    /// Returns a debug proxy for an `Option<Widget>` (more compact than the default impl for `Option<WidgetId>`).
    pub fn dbg_option(id: Option<WidgetId>) -> WidgetIdDebug {
        WidgetIdDebug(id)
    }
}

impl fmt::Debug for WidgetId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:04X}", self.0.to_u64())
    }
}
