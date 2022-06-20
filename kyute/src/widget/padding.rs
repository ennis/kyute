use crate::{
    core::DebugNode,
    widget::{prelude::*, Modifier},
    LayerPaintCtx, Length, SideOffsets,
};

/// Applies padding to the inner widget.
pub struct Padding {
    top: Length,
    right: Length,
    bottom: Length,
    left: Length,
}

impl Padding {
    pub fn new(
        top: impl Into<Length>,
        right: impl Into<Length>,
        bottom: impl Into<Length>,
        left: impl Into<Length>,
    ) -> Padding {
        Padding {
            top: top.into(),
            right: right.into(),
            bottom: bottom.into(),
            left: left.into(),
        }
    }
}

impl Modifier for Padding {
    fn layout<W: Widget>(
        &self,
        ctx: &mut LayoutCtx,
        widget: &W,
        constraints: &LayoutConstraints,
        env: &Environment,
    ) -> Layout {
        let top = constraints.resolve_height(self.top);
        let right = constraints.resolve_width(self.right);
        let bottom = constraints.resolve_height(self.bottom);
        let left = constraints.resolve_width(self.left);
        let subconstraints = constraints.deflate(SideOffsets::new(top, right, bottom, left));
        let mut layout = widget.layout(ctx, &subconstraints, env);
        layout.padding_top += top;
        layout.padding_right += right;
        layout.padding_bottom += bottom;
        layout.padding_left += left;
        layout
    }

    fn debug_node(&self) -> DebugNode {
        DebugNode::new(format!(
            "padding {:?},{:?},{:?},{:?}",
            self.top, self.right, self.bottom, self.left
        ))
    }
}
