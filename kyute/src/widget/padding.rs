use crate::{
    core::DebugNode,
    widget::{prelude::*, Modifier},
    Length, SideOffsets,
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
        constraints: &LayoutParams,
        env: &Environment,
    ) -> BoxLayout {
        let top = self.top.compute(constraints, env);
        let right = self.right.compute(constraints, env);
        let bottom = self.bottom.compute(constraints, env);
        let left = self.left.compute(constraints, env);
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
