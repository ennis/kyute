//! Reconciliation logic for list of Nodes.
use crate::state::NodeKey;
use crate::visual::{NodeArena, NodeData, Visual};
use crate::Layout;
use generational_indextree::NodeId;
use log::trace;
use std::any;
use std::any::TypeId;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum NodeCursor {
    /// Place as a child of the specified node.
    Child(NodeId),
    /// Place after the specified node.
    After(NodeId),
}

impl NodeCursor {
    pub fn first_child(parent: NodeId) -> NodeCursor {
        NodeCursor::Child(parent)
    }

    pub fn insert(&mut self, id: NodeId, arena: &mut NodeArena) {
        match *self {
            NodeCursor::Child(parent) => {
                if arena[parent].first_child() != Some(id) {
                    parent.prepend(id, arena);
                }
                *self = NodeCursor::After(id);
            }
            NodeCursor::After(before) => {
                if id != before {
                    before.insert_after(id, arena);
                    *self = NodeCursor::After(id);
                }
            }
        }
    }
}

pub(crate) fn find_by_typeid_and_key(
    nodes: &mut NodeArena,
    cursor: NodeCursor,
    ty: TypeId,
    key: Option<NodeKey>,
) -> Option<NodeId> {
    match cursor {
        NodeCursor::Child(parent) => parent.children(nodes).find(|&id| {
            let node = nodes.get(id).unwrap();
            node.get().visual.type_id() == ty && node.get().key == key
        }),
        NodeCursor::After(sibling) => sibling.following_siblings(nodes).skip(1).find(|&id| {
            let node = nodes.get(id).unwrap();
            node.get().visual.type_id() == ty && node.get().key == key
        }),
        _ => unimplemented!(),
    }
}

impl NodeCursor {
    pub fn reconcile_with_key<V: Visual, F: FnOnce(Option<NodeData<V>>) -> NodeData<V>>(
        &mut self,
        nodes: &mut NodeArena,
        key: Option<NodeKey>,
        f: F,
    ) -> NodeId {
        // find a matching node and move it in place if necessary
        // otherwise, insert a new node
        if let Some(id) = find_by_typeid_and_key(nodes, *self, TypeId::of::<V>(), key) {
            self.insert(id, nodes);
            trace!("reconciled {}({:?})", any::type_name::<V>(), key);
            // we know downcast won't fail because of the invariants of find_by_typeid_and_key
            let node = nodes.get_mut(id).unwrap().get_mut().downcast_mut().unwrap();
            // sleight-of-hand to extract the node, pass it to the closure(), and replace it with the
            // returned node
            replace_with::replace_with_or_abort(node, move |prev_node| f(Some(prev_node)));
            id
        } else {
            // insert
            trace!("not found {}({:?})", any::type_name::<V>(), key);
            let id = nodes.new_node(Box::new(f(None)));
            self.insert(id, nodes);
            id
        }
    }

    pub fn reconcile<V: Visual, F: FnOnce(Option<NodeData<V>>) -> NodeData<V>>(
        &mut self,
        nodes: &mut NodeArena,
        f: F,
    ) -> NodeId {
        self.reconcile_with_key(nodes, None, f)
    }

    pub fn get_or_insert_default<V: Visual + Default>(&mut self, nodes: &mut NodeArena) -> NodeId {
        self.reconcile::<V, _>(nodes, |node| node.unwrap_or_default())
    }

    pub fn get_or_insert_with<V: Visual, F: FnOnce() -> NodeData<V>>(
        &mut self,
        nodes: &mut NodeArena,
        f: F,
    ) -> NodeId {
        self.reconcile(nodes, |node| {
            //dbg!(node.is_some());
            node.unwrap_or_else(f)
        })
    }

    pub fn remove_after(&mut self, nodes: &mut NodeArena) {
        match self {
            NodeCursor::Child(id) => {
                // nothing
            }
            NodeCursor::After(before) => {
                for id in before.following_siblings(nodes).skip(1) {
                    //unimplemented!()
                }
            }
        }
    }
}
