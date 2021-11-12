mod paint;

use crate::{key::Key, layout::Measurements, widget::Widget, Offset, Point};
pub use paint::PaintCtx;
use std::{cell::Cell, panic::Location};


pub struct NodeRef<'a> {
    pub tree: &'a NodeTree,
    pub id: NodeId,
}

pub struct NodeRefMut<'a> {
    pub tree: &'a mut NodeTree,
    pub id: NodeId,
}

struct Links {
    parent: Option<NodeId>,
    previous_sibling: Option<NodeId>,
    next_sibling: Option<NodeId>,
    first_child: Option<NodeId>,
    last_child: Option<NodeId>,
}

impl Links {
    fn root() -> Links {
        Links {
            parent: None,
            previous_sibling: None,
            next_sibling: None,
            first_child: None,
            last_child: None,
        }
    }
}

/// A node within the visual tree.
///
/// It contains the bounds of the visual, and an instance of [`Visual`] that defines its behavior:
/// painting, hit-testing, and how it responds to events that target the visual.
pub struct Node<ElementType: Widget + ?Sized = dyn Widget> {
    links: Links,
    /// Offset of the node relative to the parent widget
    pub(crate) offset: Offset,
    /// Layout of the node (size and baseline).
    pub(crate) measurements: Measurements,
    /// Position of the node in window coordinates.
    pub(crate) window_pos: Cell<Point>,
    pub(crate) widget: Option<Box<ElementType>>,
    /// Key associated to the node.
    pub(crate) key: Option<Key>,
}

impl<T: Widget + ?Sized> Node<T> {
    pub fn parent(&self) -> Option<NodeId> {
        self.links.parent
    }
    pub fn previous_sibling(&self) -> Option<NodeId> {
        self.links.previous_sibling
    }
    pub fn next_sibling(&self) -> Option<NodeId> {
        self.links.next_sibling
    }
    pub fn first_child(&self) -> Option<NodeId> {
        self.links.first_child
    }
    pub fn last_child(&self) -> Option<NodeId> {
        self.links.last_child
    }
}

/// A position between nodes in the node tree.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum NodeCursor {
    /// Points before the specified node.
    Before(NodeId),
    /// Points before the first child of the specified node.
    BeforeChild(NodeId),
    /// Points after the specified node.
    After(NodeId),
}

/// A tree of visual nodes representing the user interface elements shown in a window.
///
/// Contrary to the widget tree, those nodes are retained (as much as possible) across data updates
/// and relayouts. It is incrementally updated by [widgets](crate::widget::Widget) during layout.
///
/// See also: [`Widget::layout`](crate::widget::Widget::layout).
pub struct NodeTree {
    nodes: slotmap::SlotMap<NodeId, Node>,
    root: NodeId,
    window_origin: Point,
}

pub struct NodeIter<'a> {
    tree: &'a NodeTree,
    current: Option<NodeId>,
}

impl<'a> Iterator for NodeIter<'a> {
    type Item = NodeId;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            if let Some(current) = self.current {
                let node = self.tree.nodes.get_unchecked(current);
                self.current = node.links.next_sibling;
                Some(current)
            } else {
                None
            }
        }
    }
}

impl NodeTree {
    /// Creates a new node tree containing a single root node.
    pub fn new() -> NodeTree {
        let mut nodes = slotmap::SlotMap::with_key();

        // create the root node
        let root = nodes.insert(Node {
            links: Links::root(),
            offset: Default::default(),
            measurements: Default::default(),
            window_pos: Cell::new(Default::default()),
            widget: None,
            key: None,
        });

        NodeTree {
            nodes,
            root,
            window_origin: Point::origin(),
        }
    }

    /// Returns the ID of the root node.
    pub fn root(&self) -> NodeId {
        self.root
    }

    /// Returns a reference to the node with the given ID.
    pub fn get(&self, id: NodeId) -> Option<&Node> {
        self.nodes.get(id)
    }

    /// Returns a reference to the node with the given ID.
    pub unsafe fn get_unchecked(&self, id: NodeId) -> &Node {
        self.nodes.get_unchecked(id)
    }

    /// Returns a mutable reference to the node with the given ID.
    pub fn get_mut(&mut self, id: NodeId) -> Option<&mut Node> {
        self.nodes.get_mut(id)
    }

    /// Returns an iterator over the children of the specified node.
    pub fn children(&self, id: NodeId) -> NodeIter {
        NodeIter {
            tree: self,
            current: self.nodes.get(id).unwrap().first_child(),
        }
    }

    /// Returns an iterator over the elements "to the right" of the specified cursor.
    ///
    /// Panics if the cursor isn't valid.
    pub fn cursor_iter(&self, cursor: NodeCursor) -> NodeIter {
        match cursor {
            NodeCursor::Before(sibling) => {
                self.nodes.get(sibling).expect("invalid cursor");
                NodeIter {
                    tree: self,
                    current: Some(sibling),
                }
            }
            NodeCursor::BeforeChild(parent) => {
                let node = self.nodes.get(parent).expect("invalid cursor");
                NodeIter {
                    tree: self,
                    current: node.first_child(),
                }
            }
            NodeCursor::After(sibling) => {
                let node = self.nodes.get(sibling).expect("invalid cursor");
                NodeIter {
                    tree: self,
                    current: node.next_sibling(),
                }
            }
        }
    }

    /// Returns the ID of the node just after the cursor.
    pub fn node_after(&self, cursor: NodeCursor) -> Option<NodeId> {
        self.cursor_iter(cursor).next()
    }

    /// Returns an iterator over the following siblings of the specified node.
    pub fn following_siblings(&self, id: NodeId) -> NodeIter {
        NodeIter {
            tree: self,
            current: self.nodes.get(id).unwrap().next_sibling(),
        }
    }

    /// Creates a new, unattached node.
    pub fn create(&mut self, element: Box<dyn Widget>) -> NodeId {
        self.nodes.insert(Node {
            links: Links {
                parent: None,
                previous_sibling: None,
                next_sibling: None,
                first_child: None,
                last_child: None,
            },
            offset: Default::default(),
            measurements: Default::default(),
            window_pos: Cell::new(Default::default()),
            widget: Some(element),
            key: None,
        })
    }

    /// Inserts a node at a specified location.
    pub fn insert(&mut self, id: NodeId, at: NodeCursor) {
        self.detach(id);
        unsafe {
            match at {
                NodeCursor::Before(after_id) => {
                    let [this, after] = self.nodes.get_disjoint_unchecked_mut([id, after_id]);
                    let prev_id = after.links.previous_sibling;
                    let parent_id = after.links.parent;

                    this.links.next_sibling = Some(after_id);
                    this.links.previous_sibling = prev_id;
                    this.links.parent = parent_id;
                    after.links.previous_sibling = Some(id);

                    if let Some(prev_id) = prev_id {
                        let prev = self.nodes.get_unchecked_mut(prev_id);
                        prev.links.next_sibling = Some(id);
                    } else if let Some(parent_id) = parent_id {
                        let parent = self.nodes.get_unchecked_mut(parent_id);
                        parent.links.first_child = Some(id);
                    }
                }

                NodeCursor::After(before_id) => {
                    let [this, before] = self.nodes.get_disjoint_unchecked_mut([id, before_id]);
                    let next_id = before.links.next_sibling;
                    let parent_id = before.links.parent;

                    this.links.previous_sibling = Some(before_id);
                    this.links.next_sibling = next_id;
                    this.links.parent = parent_id;
                    before.links.next_sibling = Some(id);

                    if let Some(next_id) = next_id {
                        let next = self.nodes.get_unchecked_mut(next_id);
                        next.links.previous_sibling = Some(id);
                    } else if let Some(parent_id) = parent_id {
                        let parent = self.nodes.get_unchecked_mut(parent_id);
                        parent.links.last_child = Some(id);
                    }
                }

                NodeCursor::BeforeChild(parent_id) => {
                    let [this, parent] = self.nodes.get_disjoint_unchecked_mut([id, parent_id]);
                    let last_child = parent.links.last_child;

                    this.links.parent = Some(parent_id);
                    this.links.previous_sibling = last_child;
                    parent.links.last_child = Some(id);

                    if let Some(last_child) = last_child {
                        let child = self.nodes.get_unchecked_mut(last_child);
                        child.links.next_sibling = Some(id);
                    } else {
                        parent.links.first_child = Some(id);
                    }
                }
            }
        }
    }

    /// Detaches the specified node and its descendants from its parent, but does not remove the
    /// node itself.
    pub fn detach(&mut self, id: NodeId) {
        let node = self.nodes.get_mut(id).unwrap();
        let prev_id = node.links.previous_sibling;
        let next_id = node.links.next_sibling;
        let parent_id = node.links.parent;

        node.links.parent = None;
        node.links.previous_sibling = None;
        node.links.next_sibling = None;

        unsafe {
            if let Some(prev_id) = prev_id {
                let prev = self.nodes.get_unchecked_mut(prev_id);
                prev.links.next_sibling = next_id;
            } else if let Some(parent_id) = parent_id {
                let parent = self.nodes.get_unchecked_mut(parent_id);
                parent.links.first_child = next_id;
            }

            if let Some(next_id) = next_id {
                let next = self.nodes.get_unchecked_mut(next_id);
                next.links.previous_sibling = prev_id;
            } else if let Some(parent_id) = parent_id {
                let parent = self.nodes.get_unchecked_mut(parent_id);
                parent.links.last_child = prev_id;
            }
        }
    }

    pub fn remove(&mut self, id: NodeId) {
        self.detach(id);
        self.nodes.remove(id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::widget::Dummy;

    #[test]
    fn test_insertion() {
        let mut node_tree = NodeTree::new();
        let root = node_tree.root();
        {
            let root_node = node_tree.get(root).unwrap();
            assert_eq!(root_node.first_child(), None);
            assert_eq!(root_node.last_child(), None);
            assert_eq!(root_node.previous_sibling(), None);
            assert_eq!(root_node.next_sibling(), None);
            assert_eq!(root_node.parent(), None);
        }

        // insert child: R(A)
        let a = node_tree.create(Box::new(Dummy));
        node_tree.insert(a, NodeCursor::BeforeChild(root));
        {
            let a_node = node_tree.get(a).unwrap();
            let root_node = node_tree.get(root).unwrap();
            assert_eq!(a_node.parent(), Some(root));
            assert_eq!(a_node.previous_sibling(), None);
            assert_eq!(a_node.next_sibling(), None);
            assert_eq!(root_node.first_child(), Some(a));
            assert_eq!(root_node.last_child(), Some(a));
        }

        // insert child: R(A,B)
        let b = node_tree.create(Box::new(Dummy));
        node_tree.insert(b, NodeCursor::BeforeChild(root));
        {
            let a_node = node_tree.get(a).unwrap();
            let b_node = node_tree.get(b).unwrap();
            let root_node = node_tree.get(root).unwrap();

            assert_eq!(b_node.parent(), Some(root));
            assert_eq!(a_node.previous_sibling(), None);
            assert_eq!(a_node.next_sibling(), Some(b));
            assert_eq!(b_node.previous_sibling(), Some(a));
            assert_eq!(b_node.next_sibling(), None);
            assert_eq!(root_node.first_child(), Some(a));
            assert_eq!(root_node.last_child(), Some(b));
        }

        // insert before (begin): R(C,A,B)
        let c = node_tree.create(Box::new(Dummy));
        node_tree.insert(c, NodeCursor::Before(a));
        {
            let a_node = node_tree.get(a).unwrap();
            let c_node = node_tree.get(c).unwrap();
            let root_node = node_tree.get(root).unwrap();

            assert_eq!(c_node.parent(), Some(root));
            assert_eq!(a_node.previous_sibling(), Some(c));
            assert_eq!(a_node.next_sibling(), Some(b));
            assert_eq!(c_node.previous_sibling(), None);
            assert_eq!(c_node.next_sibling(), Some(a));
            assert_eq!(root_node.first_child(), Some(c));
            assert_eq!(root_node.last_child(), Some(b));
        }

        // insert before (end): R(C,A,D,B)
        let d = node_tree.create(Box::new(Dummy));
        node_tree.insert(d, NodeCursor::Before(b));
        {
            let a_node = node_tree.get(a).unwrap();
            let d_node = node_tree.get(d).unwrap();
            let b_node = node_tree.get(b).unwrap();
            assert_eq!(d_node.parent(), Some(root));
            assert_eq!(a_node.next_sibling(), Some(d));
            assert_eq!(d_node.previous_sibling(), Some(a));
            assert_eq!(d_node.next_sibling(), Some(b));
            assert_eq!(b_node.previous_sibling(), Some(d));
        }

        // insert before (middle): R(C,A,E,D,B)
        let e = node_tree.create(Box::new(Dummy));
        node_tree.insert(e, NodeCursor::Before(d));
        {
            let a_node = node_tree.get(a).unwrap();
            let e_node = node_tree.get(e).unwrap();
            let d_node = node_tree.get(d).unwrap();
            assert_eq!(e_node.parent(), Some(root));
            assert_eq!(a_node.next_sibling(), Some(e));
            assert_eq!(e_node.previous_sibling(), Some(a));
            assert_eq!(e_node.next_sibling(), Some(d));
            assert_eq!(d_node.previous_sibling(), Some(e));
        }

        // insert after (middle): R(C,A,E,F,D,B)
        let f = node_tree.create(Box::new(Dummy));
        node_tree.insert(f, NodeCursor::After(e));
        {
            let e_node = node_tree.get(e).unwrap();
            let f_node = node_tree.get(f).unwrap();
            let d_node = node_tree.get(d).unwrap();
            assert_eq!(f_node.parent(), Some(root));
            assert_eq!(e_node.next_sibling(), Some(f));
            assert_eq!(f_node.previous_sibling(), Some(e));
            assert_eq!(f_node.next_sibling(), Some(d));
            assert_eq!(d_node.previous_sibling(), Some(f));
        }

        // insert after (end): R(C,A,E,F,D,B,G)
        let g = node_tree.create(Box::new(Dummy));
        node_tree.insert(g, NodeCursor::After(b));
        {
            let b_node = node_tree.get(b).unwrap();
            let g_node = node_tree.get(g).unwrap();
            let root_node = node_tree.get(root).unwrap();
            assert_eq!(g_node.parent(), Some(root));
            assert_eq!(b_node.next_sibling(), Some(g));
            assert_eq!(g_node.previous_sibling(), Some(b));
            assert_eq!(g_node.next_sibling(), None);
            assert_eq!(root_node.last_child(), Some(g));
        }
    }

    #[test]
    fn test_removal() {
        let mut node_tree = NodeTree::new();
        let root = node_tree.root();
        // R(A,B,C,D)
        let a = node_tree.create(Box::new(Dummy));
        node_tree.insert(a, NodeCursor::BeforeChild(root));
        let b = node_tree.create(Box::new(Dummy));
        node_tree.insert(b, NodeCursor::After(a));
        let c = node_tree.create(Box::new(Dummy));
        node_tree.insert(c, NodeCursor::After(b));
        let d = node_tree.create(Box::new(Dummy));
        node_tree.insert(d, NodeCursor::After(c));

        // R(B,C,D)
        node_tree.detach(a);
        {
            let root_node = node_tree.get(root).unwrap();
            let b_node = node_tree.get(b).unwrap();
            assert_eq!(root_node.first_child(), Some(b));
            assert_eq!(root_node.last_child(), Some(d));
        }

        // R(B,D)
        node_tree.detach(c);
        {
            let root_node = node_tree.get(root).unwrap();
            let b_node = node_tree.get(b).unwrap();
            let d_node = node_tree.get(d).unwrap();
            assert_eq!(b_node.next_sibling(), Some(d));
            assert_eq!(d_node.previous_sibling(), Some(b));
            assert_eq!(root_node.first_child(), Some(b));
            assert_eq!(root_node.last_child(), Some(d));
        }

        // R(D)
        node_tree.detach(b);
        {
            let root_node = node_tree.get(root).unwrap();
            let d_node = node_tree.get(d).unwrap();
            assert_eq!(d_node.next_sibling(), None);
            assert_eq!(d_node.previous_sibling(), None);
            assert_eq!(root_node.first_child(), Some(d));
            assert_eq!(root_node.last_child(), Some(d));
        }

        // R
        node_tree.detach(d);
        {
            let root_node = node_tree.get(root).unwrap();
            assert_eq!(root_node.first_child(), None);
            assert_eq!(root_node.last_child(), None);
        }
    }

    #[test]
    fn test_iter() {
        let mut node_tree = NodeTree::new();
        let root = node_tree.root();
        // R(A,B,C,D)
        let a = node_tree.create(Box::new(Dummy));
        node_tree.insert(a, NodeCursor::BeforeChild(root));
        let b = node_tree.create(Box::new(Dummy));
        node_tree.insert(b, NodeCursor::After(a));
        let c = node_tree.create(Box::new(Dummy));
        node_tree.insert(c, NodeCursor::After(b));
        let d = node_tree.create(Box::new(Dummy));
        node_tree.insert(d, NodeCursor::After(c));

        let root_node = node_tree.get(root).unwrap();
        let children: Vec<_> = node_tree
            .children(root)
            .map(|x| node_tree.get(x).unwrap() as *const Node)
            .collect();
        assert_eq!(
            &children[..],
            &[
                node_tree.get(a).unwrap() as *const _,
                node_tree.get(b).unwrap() as *const _,
                node_tree.get(c).unwrap() as *const _,
                node_tree.get(d).unwrap() as *const _,
            ]
        );
    }
}
