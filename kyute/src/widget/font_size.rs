//! Font size modifier
use crate::{
    theme,
    widget::{prelude::*, Modifier},
};

/// Font size modifier.
pub struct FontSize(pub Length);

impl Modifier for FontSize {
    fn layout<W: Widget>(
        &self,
        ctx: &mut LayoutCtx,
        widget: &W,
        constraints: &LayoutParams,
        env: &Environment,
    ) -> Geometry {
        let font_size = self.0.compute(constraints, env);
        widget.layout(ctx, &constraints, &env.clone().add(theme::FONT_SIZE, font_size))
    }

    fn debug_node(&self) -> DebugNode {
        DebugNode::new(format!("font size: {:?}", self.0))
    }
}
