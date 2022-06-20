use crate::{
    core::DebugNode,
    widget::{prelude::*, Modifier},
    LayoutConstraints, Length,
};

/// Minimum-width constraint.
pub struct MinWidth(pub Length);
/// Maximum-width constraint.
pub struct MaxWidth(pub Length);
/// Minimum-height constraint.
pub struct MinHeight(pub Length);
/// Maximum-height constraint.
pub struct MaxHeight(pub Length);

macro_rules! impl_size_constraint {
    ($t:ident; $body:expr; $debug:literal) => {
        impl Modifier for $t {
            fn layout<W: Widget>(
                &self,
                ctx: &mut LayoutCtx,
                widget: &W,
                constraints: &LayoutConstraints,
                env: &Environment,
            ) -> Layout {
                let mut subconstraints = *constraints;
                ($body)(constraints, &mut subconstraints, self.0);
                widget.layout(ctx, &subconstraints, env)
            }

            fn debug_node(&self) -> DebugNode {
                DebugNode::new(format!(std::concat!($debug, ": {:?}"), self.0))
            }
        }
    };
}

impl_size_constraint!(MinWidth;
    |constraints, sub, value| *sub.min.width = sub.min.width.max(constraints.resolve_width(value));
    "minimum width"
);
impl_size_constraint!(MinHeight;
    |constraints, sub, value| *sub.min.height = sub.min.height.max(constraints.resolve_height(value));
    "minimum height"
);
impl_size_constraint!(MaxWidth;
    |constraints, sub, value| *sub.max.width = sub.max.width.min(constraints.resolve_width(value));
    "minimum width"
);
impl_size_constraint!(MaxHeight;
    |constraints, sub, value| *sub.max.height = sub.max.height.min(constraints.resolve_height(value));
    "minimum height"
);

pub struct Fill;

impl Modifier for Fill {
    fn layout<W: Widget>(
        &self,
        ctx: &mut LayoutCtx,
        widget: &W,
        constraints: &LayoutConstraints,
        env: &Environment,
    ) -> Layout {
        let mut subconstraints = *constraints;
        subconstraints.min = subconstraints.max;
        widget.layout(ctx, &subconstraints, env)
    }

    fn debug_node(&self) -> DebugNode {
        DebugNode::new("fill")
    }
}
