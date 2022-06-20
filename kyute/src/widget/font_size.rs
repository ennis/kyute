//! Alignment modifiers
use crate::{
    core::DebugNode,
    widget::{prelude::*, Modifier},
};

/// Font size modifier.
pub struct FontSize(pub Length);

impl Modifier for FontSize {
    fn layout<W: Widget>(
        &self,
        ctx: &mut LayoutCtx,
        widget: &W,
        constraints: &LayoutConstraints,
        env: &Environment,
    ) -> Layout {
        let subconstraints = LayoutConstraints {
            parent_font_size: constraints.resolve_height(self.0),
            ..*constraints
        };
        widget.layout(ctx, &subconstraints, env)
    }

    fn debug_node(&self) -> DebugNode {
        DebugNode::new(format!("font size: {:?}", self.0))
    }
}
