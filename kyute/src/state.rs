//! Stack of widget IDs.
use std::{
    any::Any,
    collections::{hash_map::DefaultHasher, HashMap},
    hash::{Hash, Hasher},
};

pub type NodeKey = u64;

/// The ID stack. Each level corresponds to a parent ItemNode.
struct KeyStack(pub(super) Vec<NodeKey>);

impl KeyStack {
    /// Creates a new IdStack and push the specified ID onto it.
    pub fn new(root_key: NodeKey) -> KeyStack {
        KeyStack(vec![root_key])
    }

    fn chain_hash<H: Hash>(&self, s: &H) -> NodeKey {
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

    /// Hashes the given data, initializing the hasher with the items currently on the stack.
    /// Pushes the result on the stack and returns it.
    /// This is used to generate a unique ID per item path in the hierarchy.
    pub fn push_key<H: Hash>(&mut self, s: &H) -> NodeKey {
        let id = self.chain_hash(s);
        //let parent_id = *self.0.last().unwrap();
        self.0.push(id);
        id
    }

    /// Pops the ID at the top of the stack.
    pub fn pop_key(&mut self) {
        self.0.pop();
    }
}

pub struct StateCtx {
    stack: KeyStack,
    state: HashMap<NodeKey, Box<dyn Any>>,
}

impl StateCtx {
    pub(super) fn new() -> StateCtx {
        StateCtx {
            stack: KeyStack::new(0),
            state: HashMap::new(),
        }
    }

    pub fn with_id<H: Hash, R, F: FnOnce(&mut StateCtx) -> R>(&mut self, s: &H, f: F) -> R {
        self.stack.push_key(s);
        let r = f(self);
        self.stack.pop_key();
        r
    }
}

// widget
// visual
// cache
//
// scene -> cached layout stuff + state
// stage -> where to render the scene, create using a
// painter ->
//
// stage:
//      - owns PlatformWindow,
//      - implements WindowHandler,
//           - receives events, translate them, and feed them to the UI, collect actions
//           - dispatch actions to handler
//      - provide an "application" trait that generates the widget tree
//          - own state, or reference application state through RC
//          -> long-lived "borrow"
//      - some widgets (context menu), when repainted, check if the window is already opened
// Child dialogs / popup windows:
//  - another WindowHandler
//  - send actions to parent window
//
