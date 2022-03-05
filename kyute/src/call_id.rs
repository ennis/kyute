use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    fmt,
    hash::{Hash, Hasher},
    panic::Location,
    rc::Rc,
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
        /*#[cfg(debug_assertions)]
        {
            if f.alternate() {
                writeln!("CallKey({:016X})", self.id);
                let mut info = Some(&self.info);
                let mut depth = 0;
                while let Some(current_location) = location {
                    writeln!(f, "\t[{:2}]: {}", depth, current_location.location)?;
                    depth += 1;
                    location = current_location.parent;
                }
                Ok(())
            } else {
                f.debug_tuple("CallKey")
                    .field(&format_args!("{:016X}", self.0))
                    .finish()
            }
        }

        #[cfg(not(debug_assertions))]*/

        f.debug_tuple("CallKey")
            .field(&format_args!("{:016X}", self.0))
            .finish()
    }
}

#[derive(Clone)]
pub struct CallNode {
    id: CallId,
    parent: Option<Rc<CallNode>>,
    pub(crate) location: &'static Location<'static>,
    pub(crate) index: usize, // or `iteration`, `count`
}

impl fmt::Debug for CallNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if f.alternate() {
            writeln!(f, "CallNode({:016X})", self.id.0)?;
            let mut node = Some(self);
            //let mut depth = 0;
            while let Some(current_node) = node {
                writeln!(f, "\t --> {} (index {})", current_node.location, current_node.index)?;
                //depth += 1;
                node = current_node.parent.as_deref();
            }
            Ok(())
        } else {
            f.debug_tuple("CallNode")
                .field(&format_args!("{:016X}", self.id.0))
                .finish()
        }
    }
}

pub(crate) struct CallIdStack {
    id_stack: Vec<u64>,
    nodes: HashMap<CallId, Rc<CallNode>>,
    current_node: Option<Rc<CallNode>>,
}

impl CallIdStack {
    /// Creates a new empty CallIdStack.
    pub fn new() -> CallIdStack {
        CallIdStack {
            id_stack: vec![],
            nodes: Default::default(),
            current_node: None,
        }
    }

    fn chain_hash<H: Hash>(&self, s: &H) -> u64 {
        let stacklen = self.id_stack.len();
        let key1 = if stacklen >= 2 { self.id_stack[stacklen - 2] } else { 0 };
        let key0 = if stacklen >= 1 { self.id_stack[stacklen - 1] } else { 0 };
        let mut hasher = DefaultHasher::new();
        key0.hash(&mut hasher);
        key1.hash(&mut hasher);
        s.hash(&mut hasher);
        hasher.finish()
    }

    /// Enters a scope in the call graph.
    pub fn enter(&mut self, location: &'static Location<'static>, index: usize) -> CallId {
        let id = self.chain_hash(&(location, index));
        self.id_stack.push(id);
        let id = CallId(id);
        let node = Rc::new(CallNode {
            id,
            parent: self.current_node.clone(),
            location,
            index,
        });
        self.nodes.insert(id, node.clone());
        self.current_node = Some(node);
        id
    }

    /// Exits a scope previously entered with `enter`.
    pub fn exit(&mut self) {
        self.id_stack.pop();
        self.current_node = self.current_node.take().unwrap().parent.clone();
    }

    /// Returns the `CallId` of the current scope.
    pub fn current(&self) -> CallId {
        CallId(*self.id_stack.last().unwrap())
    }

    /// Returns the current node in the call tree.
    pub fn current_call_node(&self) -> Option<Rc<CallNode>> {
        self.current_node.clone()
    }

    /// Returns the call node corresponding to the specified CallId.
    pub fn call_node(&self, id: CallId) -> Option<Rc<CallNode>> {
        self.nodes.get(&id).cloned()
    }

    /// Returns whether the stack is empty.
    ///
    /// The stack is empty just after creation, and when `enter` and `exit` calls are balanced.
    pub fn is_empty(&self) -> bool {
        self.id_stack.is_empty()
    }
}
