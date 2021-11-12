use std::{
    collections::hash_map::DefaultHasher,
    fmt,
    fmt::Formatter,
    hash::{Hash, Hasher},
    panic::Location,
};

/// Identifies a particular call site in a call tree.
///
/// TODO more docs
#[derive(Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct CallId(pub u64);

impl CallId {
    pub fn to_u64(&self) -> u64 {
        self.0
    }
}

impl fmt::Debug for CallId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("CallKey")
            .field(&format_args!("{:016X}", self.0))
            .finish()
    }
}


pub(crate) struct CallIdStack(Vec<u64>);

impl CallIdStack {
    /// Creates a new empty CallIdStack.
    pub fn new() -> CallIdStack {
        CallIdStack(vec![])
    }

    fn chain_hash<H: Hash>(&self, s: &H) -> u64 {
        let stacklen = self.0.len();
        let key1 = if stacklen >= 2 {
            self.0[stacklen - 2]
        } else {
            0
        };
        let key0 = if stacklen >= 1 {
            self.0[stacklen - 1]
        } else {
            0
        };
        let mut hasher = DefaultHasher::new();
        key0.hash(&mut hasher);
        key1.hash(&mut hasher);
        s.hash(&mut hasher);
        hasher.finish()
    }

    /// Enters a scope in the call graph.
    pub fn enter(&mut self, location: &Location, index: usize) -> CallId {
        let id = self.chain_hash(&(location, index));
        self.0.push(id);
        CallId(id)
    }

    /// Exits a scope previously entered with `enter`.
    pub fn exit(&mut self) {
        self.0.pop();
    }

    /// Returns the `CallId` of the current scope.
    pub fn current(&self) -> CallId {
        CallId(*self.0.last().unwrap())
    }

    /// Returns whether the stack is empty.
    ///
    /// The stack is empty just after creation, and when `enter` and `exit` calls are balanced.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}
