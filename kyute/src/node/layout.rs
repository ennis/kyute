//! Layout and reconciliation pass.
use super::NodeData;
use crate::application::WindowCtx;
use crate::layout::Size;
use crate::layout::{BoxConstraints, Offset};
use crate::node::{NodeArena, NodeTree};
use crate::state::NodeKey;
use crate::widget::BoxedWidget;
use crate::{env, node, Environment, Measurements, Point, Widget};
use generational_indextree::{Node, NodeEdge, NodeId};
use kyute_shell::platform::Platform;
use std::any::TypeId;
use std::rc::Rc;

/// A position between nodes in the node tree.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum NodeCursor {
    Before(NodeId),
    /// Place as a child of the specified node.
    Child(NodeId),
    /// Place after the specified node.
    After(NodeId),
}

impl NodeCursor {
    pub fn first_child(parent: NodeId) -> NodeCursor {
        NodeCursor::Child(parent)
    }

    /*pub fn insert(&mut self, id: NodeId, arena: &mut NodeArena) {
        match *self {
            NodeCursor::Before(root) => unimplemented!(),
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
    }*/
}

/// Context passed to a widget during the layout pass.
///
/// See [`Widget::layout`].
pub struct LayoutCtx<'a, 'ctx> {
    pub(crate) win_ctx: &'a mut WindowCtx<'ctx>,
    arena: &'a mut NodeArena,
    // Current (parent) node, None if pre-root
    node: Option<NodeId>,
    // reconciliation cursor
    cursor: &'a mut NodeCursor,
}

impl<'a, 'ctx> LayoutCtx<'a, 'ctx> {
    /// Returns the global platform object.
    pub fn platform(&self) -> &'ctx Platform {
        self.win_ctx.platform
    }

    /// Emits a child widget.
    ///
    /// Returns the ID of the node associated to the widget, and its measurements.
    pub fn emit_child(
        &mut self,
        widget: impl Widget,
        constraints: &BoxConstraints,
        env: Environment,
    ) -> (NodeId, Measurements) {
        // Reconciliation
        let widget_key = widget.key();
        let widget_visual_type_id = widget.visual_type_id();
        let matching_node_id: Option<NodeId> = match self.cursor {
            NodeCursor::Before(sibling) => sibling.following_siblings(self.arena).find(|&id| {
                let node = self.arena.get(id).unwrap();
                node.get().visual_type_id() == Some(widget_visual_type_id)
                    && node.get().key == widget_key
            }),
            NodeCursor::Child(parent) => parent.children(self.arena).find(|&id| {
                let node = self.arena.get(id).unwrap();
                node.get().visual_type_id() == Some(widget_visual_type_id)
                    && node.get().key == widget_key
            }),
            NodeCursor::After(sibling) => {
                sibling.following_siblings(self.arena).skip(1).find(|&id| {
                    let node = self.arena.get(id).unwrap();
                    node.get().visual_type_id() == Some(widget_visual_type_id)
                        && node.get().key == widget_key
                })
            }
        };

        let (id, prev_visual) = if let Some(id) = matching_node_id {
            // reconciliation found a matching existing node: extract previous visual
            let prev_visual = self.arena.get_mut(id).unwrap().get_mut().visual.take();
            (id, prev_visual)
        } else {
            // no match, create a new node
            let id = self.arena.new_node(NodeData::new(widget_key, env.clone()));
            (id, None)
        };

        // move the node in place and advance the cursor
        match *self.cursor {
            NodeCursor::Before(after) => {
                if id != after {
                    after.insert_before(id, self.arena);
                }
            }
            NodeCursor::Child(parent) => {
                // don't prepend if it's already in place
                if self.arena[parent].first_child() != Some(id) {
                    parent.prepend(id, self.arena);
                }
                *self.cursor = NodeCursor::After(id);
            }
            NodeCursor::After(before) => {
                //assert(id != i)
                //if id != before {
                before.insert_after(id, self.arena);
                *self.cursor = NodeCursor::After(id);
                //}
            }
        }

        // run layout on the child
        // -> build layout context for the child
        // reconciliation starts at the beginning of the child list
        let mut child_cursor = NodeCursor::Child(id);
        let mut child_ctx = LayoutCtx {
            win_ctx: self.win_ctx,
            arena: self.arena,
            node: Some(id),
            cursor: &mut child_cursor,
        };
        // -> recursive call of layout
        let (visual, measurements) = widget.layout(&mut child_ctx, prev_visual, constraints, env);
        // update the measurements of the node
        self.arena[id].get_mut().measurements = measurements;
        self.arena[id].get_mut().visual = Some(visual);

        // remove all remaining nodes after the child cursor:
        // they did not match with any widget, so it means that they should be removed from the GUI
        let mut next_to_remove = match child_cursor {
            NodeCursor::Before(id) => Some(id),
            NodeCursor::Child(id) => self.arena[id].first_child(),
            NodeCursor::After(id) => self.arena[id].next_sibling(),
        };
        while let Some(to_remove) = next_to_remove {
            next_to_remove = self.arena[to_remove].next_sibling();
            to_remove.remove(self.arena);
        }

        (id, measurements)
    }

    /// Returns the ID of the node associated to the widget.
    pub fn node_id(&self) -> NodeId {
        self.node.expect("node_id called outside `Widget::layout`")
    }

    /// Sets the offset of a node relative to its parent.
    ///
    /// This is meant to be called during `Widget::layout`, on children of the node associated to the widget.
    pub fn set_child_offset(&mut self, child: NodeId, offset: Offset) {
        assert_eq!(
            self.arena[child].parent(),
            self.node,
            "set_child_position must be called on children of the current node"
        );
        self.arena[child].get_mut().offset = offset;
    }
}

impl NodeTree {
    /// Runs the layout and update passes on this tree.
    pub(crate) fn layout(
        &mut self,
        widget: BoxedWidget,
        size: Size,
        root_constraints: &BoxConstraints,
        env: Environment,
        win_ctx: &mut WindowCtx,
    ) {
        let mut cursor = NodeCursor::Before(self.root);
        let mut layout_ctx = LayoutCtx {
            win_ctx,
            arena: &mut self.arena,
            // no parent => inserting into the root list
            node: None,
            cursor: &mut cursor,
        };
        let (root, root_measurements) = layout_ctx.emit_child(widget, root_constraints, env);
        // update root (it might not be the same node)
        self.root = root;
        self.calculate_window_positions(Point::origin());
    }

    /// Recursively compute window positions of the nodes.
    fn calculate_window_positions(&mut self, origin: Point) {
        let mut stack = Vec::new();
        let mut current_origin = origin;
        for edge in self.root.traverse(&self.arena) {
            match edge {
                NodeEdge::Start(id) => {
                    stack.push(current_origin);
                    let node = self.arena[id].get();
                    current_origin += node.offset;
                    node.window_pos.set(current_origin);
                }
                NodeEdge::End(id) => {
                    current_origin = stack.pop().expect("unbalanced traversal");
                }
            }
        }
    }
}
