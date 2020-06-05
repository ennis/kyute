use crate::event::InputState;
use crate::layout::Offset;
use crate::node::event::FocusState;
use crate::node::NodeTree;
use crate::{env, Bounds, Point, Size};
use generational_indextree::NodeId;
use kyute_shell::drawing::DrawContext;
use kyute_shell::platform::Platform;
use std::ops::{Deref, DerefMut};

/// Context passed to [`Visual::paint`].
pub struct PaintCtx<'a> {
    platform: &'a Platform,
    pub(crate) draw_ctx: &'a mut DrawContext,
    window_bounds: Bounds,
    node_id: NodeId,
    focus_state: &'a FocusState,
    input_state: &'a InputState,
    hover: bool,
    focus: bool,
}

impl<'a> PaintCtx<'a> {
    pub fn platform(&self) -> &Platform {
        self.platform
    }

    /// Returns the window bounds of the node
    pub fn window_bounds(&self) -> Bounds {
        self.window_bounds
    }

    /// Returns the bounds of the node.
    pub fn bounds(&self) -> Bounds {
        Bounds::new(Point::origin(), self.window_bounds.size)
    }

    /// Returns the size of the node.
    pub fn size(&self) -> Size {
        self.window_bounds.size
    }

    pub fn is_hovering(&self) -> bool {
        self.hover
    }

    pub fn is_focused(&self) -> bool {
        self.focus
    }

    pub fn is_capturing_pointer(&self) -> bool {
        self.focus_state.pointer_grab == Some(self.node_id)
    }
}

// PaintCtx auto-derefs to a DrawContext
impl<'a> Deref for PaintCtx<'a> {
    type Target = DrawContext;

    fn deref(&self) -> &Self::Target {
        self.draw_ctx
    }
}

impl<'a> DerefMut for PaintCtx<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.draw_ctx
    }
}

impl NodeTree {
    /// Painting.
    pub fn paint(
        &mut self,
        platform: &Platform,
        draw_context: &mut DrawContext,
        input_state: &InputState,
    ) {
        self.paint_node(
            platform,
            draw_context,
            Offset::zero(),
            input_state,
            self.root,
        )
    }

    /// Draws the node in the specified drawing context.
    ///
    /// Effectively, it applies the transform of the node (which, right now, is only an offset relative to the parent),
    /// and calls [`Visual::paint`] on `self.visual`.
    fn paint_node(
        &mut self,
        platform: &Platform,
        draw_context: &mut DrawContext,
        offset: Offset,
        input_state: &InputState,
        node_id: NodeId,
    ) {
        let mut node = self.arena[node_id].get_mut();
        let node_offset = node.offset;
        let node_size = node.measurements.size;
        let window_bounds = Bounds::new(
            Point::origin() + offset + node_offset,
            node_size,
        );

        let hover = input_state
            .pointers
            .iter()
            .any(|(_, state)| window_bounds.contains(state.position));
        dbg!(hover);

        draw_context.save();
        draw_context.transform(&node_offset.to_transform());

        {
            let mut ctx = PaintCtx {
                platform,
                draw_ctx: draw_context,
                window_bounds,
                node_id,
                focus_state: &self.focus,
                input_state,
                hover,
                focus: self.focus.focus == Some(node_id),
            };
            let env = &node.env;
            node.visual.as_mut().map(|v| v.paint(&mut ctx, &env));
        }

        // paint children
        let mut child_id = self.arena[node_id].first_child();
        while let Some(id) = child_id {
            self.paint_node(
                platform,
                draw_context,
                offset + node_offset,
                input_state,
                id,
            );
            child_id = self.arena[id].next_sibling();
        }

        draw_context.restore();
    }
}
