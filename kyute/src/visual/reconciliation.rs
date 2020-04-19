//! Reconciliation logic for list of Nodes.
use crate::state::NodeKey;
use crate::visual::{Node, Visual};
use crate::Layout;
use std::any::TypeId;

/// Trait implemented by Node containers that support reconciliation.
///
/// Lists: search for match by TypeId and key, return &mut Node or None
/// Option:
pub trait NodePlace {
    // Returns bool and not a `Option<&mut ref>` to work around a borrowck limitation which would
    // subsequently prevent calling `insert` if it returned None.
    fn gather(&mut self, ty: TypeId, key: Option<NodeKey>) -> bool;
    fn get(&mut self) -> Option<&mut Node>;
    fn insert(&mut self, node: Box<Node>) -> &mut Node;
}

impl<'a> dyn NodePlace + 'a {
    pub fn reconcile_key<V: Visual, F: FnOnce(Option<Node<V>>) -> Node<V>>(
        &mut self,
        key: Option<NodeKey>,
        f: F,
    ) -> &mut Node<V> {
        // two step find-get to work around a borrowck limitation
        let found = self.gather(TypeId::of::<V>(), key);

        if found {
            // we know downcast won't fail because of the invariants of find_and_move_in_place if found == true
            let node = self.get().unwrap().downcast_mut().unwrap();
            // sleight-of-hand to extract the node, pass it to layout(), and replace it with the
            // node returned by layout()
            replace_with::replace_with_or_abort(node, move |prev_node| f(Some(prev_node)));
            node
        } else {
            // not found, insert new
            let new_node = f(None);
            self.insert(Box::new(new_node)).downcast_mut().unwrap()
        }
    }

    pub fn reconcile<V: Visual, F: FnOnce(Option<Node<V>>) -> Node<V>>(
        &mut self,
        f: F,
    ) -> &mut Node<V> {
        self.reconcile_key(None, f)
    }

    pub fn get_or_insert_default<V: Visual + Default>(&mut self) -> &mut Node<V> {
        self.reconcile(|node| node.unwrap_or_default())
    }

    pub fn get_or_insert_with<V: Visual, F: FnOnce() -> Node<V>>(&mut self, f: F) -> &mut Node<V> {
        self.reconcile(|_| f())
    }
}

/// Handles reconciliation within a list of boxed, type-erased nodes.
pub struct NodeListReplacer<'a> {
    list: &'a mut Vec<Box<Node>>,
    index: usize,
}

impl<'a> NodeListReplacer<'a> {
    ///
    pub fn new(list: &'a mut Vec<Box<Node>>) -> NodeListReplacer<'a> {
        NodeListReplacer { list, index: 0 }
    }
}

impl<'a> NodePlace for NodeListReplacer<'a> {
    fn gather(&mut self, ty: TypeId, key: Option<u64>) -> bool {
        // look for matching node
        let pos = self.list[self.index..]
            .iter()
            .position(move |node| node.visual.type_id() == ty && node.key == key);
        let cur_pos = self.index;

        // match found
        if let Some(pos) = pos {
            if pos > cur_pos {
                // match found, but further in the list: rotate in place
                self.list[cur_pos..=pos].rotate_right(1);
                //trace!("[visual tree] rotate in place");
            }
            true
        } else {
            false
        }
    }

    fn get(&mut self) -> Option<&mut Node> {
        let i = self.index;
        self.index += 1;
        self.list.get_mut(i).map(|n| n.as_mut())
    }

    fn insert(&mut self, node: Box<Node>) -> &mut Node {
        let i = self.index;
        self.index += 1;
        self.list.insert(i, node);
        &mut self.list[i]
    }
}

impl<'a> Drop for NodeListReplacer<'a> {
    fn drop(&mut self) {
        self.list.drain(self.index..);
    }
}

impl NodePlace for Box<Node> {
    fn gather(&mut self, ty: TypeId, key: Option<u64>) -> bool {
        self.visual.type_id() == ty && self.key == key
    }

    fn get(&mut self) -> Option<&mut Node> {
        Some(self)
    }

    fn insert(&mut self, node: Box<Node>) -> &mut Node {
        // actually replace
        *self = node;
        &mut *self
    }
}
