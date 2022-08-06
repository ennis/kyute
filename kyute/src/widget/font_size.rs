//! Font size modifier
use crate::widget::{prelude::*, Modifier};

/// Font size modifier.
pub struct FontSize(pub Length);

impl Modifier for FontSize {
    fn layout<W: Widget>(
        &self,
        ctx: &mut LayoutCtx,
        widget: &W,
        constraints: &LayoutParams,
        env: &Environment,
    ) -> BoxLayout {
        let subconstraints = LayoutParams {
            parent_font_size: self.0.compute(constraints),
            ..*constraints
        };
        widget.layout(ctx, &subconstraints, env)
    }

    fn debug_node(&self) -> DebugNode {
        DebugNode::new(format!("font size: {:?}", self.0))
    }
}
