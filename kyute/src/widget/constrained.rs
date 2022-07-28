use crate::{
    core::DebugNode,
    widget::{prelude::*, Modifier},
    LayoutConstraints, Length,
};

/// Minimum-width constraint.
#[derive(Copy, Clone, Debug)]
pub struct MinWidth(pub Length);
/// Maximum-width constraint.
#[derive(Copy, Clone, Debug)]
pub struct MaxWidth(pub Length);
/// Minimum-height constraint.
#[derive(Copy, Clone, Debug)]
pub struct MinHeight(pub Length);
/// Maximum-height constraint.
#[derive(Copy, Clone, Debug)]
pub struct MaxHeight(pub Length);
/// Fixed-width constraint.
#[derive(Copy, Clone, Debug)]
pub struct FixedWidth(pub Length);
/// Fixed-height constraint.
#[derive(Copy, Clone, Debug)]
pub struct FixedHeight(pub Length);

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
    |constraints: &LayoutConstraints, sub: &mut LayoutConstraints, value: Length| sub.min.width = sub.min.width.max(value.compute(constraints));
    "minimum width"
);
impl_size_constraint!(MinHeight;
    |constraints: &LayoutConstraints, sub: &mut LayoutConstraints, value: Length| sub.min.height = sub.min.height.max(value.compute(constraints));
    "minimum height"
);
impl_size_constraint!(MaxWidth;
    |constraints: &LayoutConstraints, sub: &mut LayoutConstraints, value: Length| sub.max.width = sub.max.width.min(value.compute(constraints));
    "minimum width"
);
impl_size_constraint!(MaxHeight;
    |constraints: &LayoutConstraints, sub: &mut LayoutConstraints, value: Length| sub.max.height = sub.max.height.min(value.compute(constraints));
    "minimum height"
);
impl_size_constraint!(FixedWidth;
    |constraints: &LayoutConstraints, sub: &mut LayoutConstraints, value: Length| {
        let value = value.compute(constraints);
        sub.min.width = sub.min.width.max(value);
        sub.max.width = sub.max.width.min(value);
    };
    "fixed width"
);
impl_size_constraint!(FixedHeight;
    |constraints: &LayoutConstraints, sub: &mut LayoutConstraints, value: Length| {
        let value = value.compute(constraints);
        sub.min.height = sub.min.height.max(value);
        sub.max.height = sub.max.height.min(value);
    };
    "fixed height"
);

#[derive(Copy, Clone, Debug)]
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
