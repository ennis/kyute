//! Alignment modifiers
use crate::{
    core::DebugNode,
    layout::Alignment,
    widget::{prelude::*, Modifier},
    LayerPaintCtx,
};
use kyute_common::RoundToPixel;

/// Horizontal aligmnent modifier.
pub struct HorizontalAlignment(pub Alignment);

/// Vertical alignment modifier.
pub struct VerticalAlignment(pub Alignment);

impl Modifier for HorizontalAlignment {
    fn layout<W: Widget>(
        &self,
        ctx: &mut LayoutCtx,
        widget: &W,
        constraints: &LayoutConstraints,
        env: &Environment,
    ) -> Layout {
        let sublayout = widget.layout(ctx, constraints, env);
        Layout {
            x_align: self.0,
            ..sublayout
        }
    }

    fn debug_node(&self) -> DebugNode {
        DebugNode::new(format!("horizontal alignment {:?}", self.0))
    }
}

impl Modifier for VerticalAlignment {
    fn layout<W: Widget>(
        &self,
        ctx: &mut LayoutCtx,
        widget: &W,
        constraints: &LayoutConstraints,
        env: &Environment,
    ) -> Layout {
        let sublayout = widget.layout(ctx, constraints, env);
        Layout {
            y_align: self.0,
            ..sublayout
        }
    }

    fn debug_node(&self) -> DebugNode {
        DebugNode::new(format!("vertical alignment {:?}", self.0))
    }
}
