//! Reconciliation logic for list of Nodes.
use crate::state::NodeKey;
use crate::visual::{Node, Visual};
use std::any::TypeId;

/// Handles reconciliation within a list of boxed, type-erased nodes.
pub struct NodeReplacer<'a> {
    list: &'a mut Vec<Box<Node<dyn Visual>>>,
    index: usize,
}

impl<'a> NodeReplacer<'a> {
    ///
    pub fn new(list: &'a mut Vec<Box<Node<dyn Visual>>>) -> NodeReplacer<'a> {
        NodeReplacer { list, index: 0 }
    }

    /// Finds a node that matches the given visual type and key, starting from the cursor to the end
    /// of the list.
    fn find_and_move_in_place(&mut self, ty: TypeId, key: Option<NodeKey>) -> bool {
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

    /// This function searches for a node that matches the provided type T and key in the list of
    /// nodes, starting from the cursor location.
    /// If a matching node is found, returns a ref-counted reference to the node, and mark as "dead"
    /// all nodes that were skipped during the search.
    ///
    /// If no matching node is found, a new one is created by invoking the provided closure, and
    /// inserted at the current cursor position. The current cursor position is then advanced to
    /// the next node.
    pub fn replace_or_create_with<V: Visual, F: FnOnce(Option<Node<V>>) -> Node<V>>(
        &mut self,
        key: Option<NodeKey>,
        f: F,
    ) {
        let found = self.find_and_move_in_place(TypeId::of::<V>(), key);
        let index = self.index;

        if found {
            // we know downcast won't fail because of the invariants of find_and_move_in_place if found == true
            let mut node = self.list[index].downcast_mut().unwrap();

            // sleight-of-hand to extract the node, pass it to layout(), and replace it with the node returned by
            // layout()
            replace_with::replace_with_or_abort(node, move |prev_node| f(Some(prev_node)));
        } else {
            // not found, insert new node
            let new_node = f(None);
            self.list.insert(index, Box::new(new_node));
        }

        self.index += 1;
    }
}

impl<'a> Drop for NodeReplacer<'a> {
    fn drop(&mut self) {
        self.list.drain(self.index..);
    }
}

pub fn replace_or_create_with<V: Visual, F: FnOnce(Option<Node<V>>) -> Node<V>>(
    node: &mut Box<Node<dyn Visual>>,
    key: Option<NodeKey>,
    f: F,
) {
    if let Some(node) = node.downcast_mut() {
        replace_with::replace_with_or_abort(node, move |node| f(Some(node)))
    } else {
        let new_node = f(None);
        *node = Box::new(new_node);
    }
}
