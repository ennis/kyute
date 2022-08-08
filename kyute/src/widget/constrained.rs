//! Size constraint modifiers
use crate::{
    widget::{prelude::*, Modifier},
    LayoutParams, Length,
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
                params: &LayoutParams,
                env: &Environment,
            ) -> BoxLayout {
                let mut subconstraints = *params;
                ($body)(params, &mut subconstraints, env, self.0);
                widget.layout(ctx, &subconstraints, env)
            }

            fn debug_node(&self) -> DebugNode {
                DebugNode::new(format!(std::concat!($debug, ": {:?}"), self.0))
            }
        }
    };
}

impl_size_constraint!(MinWidth;
    |constraints: &LayoutParams, sub: &mut LayoutParams, env: &Environment, value: Length| sub.min.width = sub.min.width.max(value.compute(constraints, env));
    "minimum width"
);
impl_size_constraint!(MinHeight;
    |constraints: &LayoutParams, sub: &mut LayoutParams, env: &Environment, value: Length| sub.min.height = sub.min.height.max(value.compute(constraints, env));
    "minimum height"
);
impl_size_constraint!(MaxWidth;
    |constraints: &LayoutParams, sub: &mut LayoutParams, env: &Environment, value: Length| sub.max.width = sub.max.width.min(value.compute(constraints, env));
    "minimum width"
);
impl_size_constraint!(MaxHeight;
    |constraints: &LayoutParams, sub: &mut LayoutParams, env: &Environment, value: Length| sub.max.height = sub.max.height.min(value.compute(constraints, env));
    "minimum height"
);
impl_size_constraint!(FixedWidth;
    |constraints: &LayoutParams, sub: &mut LayoutParams, env: &Environment, value: Length| {
        let value = value.compute(constraints, env);
        sub.min.width = sub.min.width.max(value);
        sub.max.width = sub.max.width.min(value);
    };
    "fixed width"
);
impl_size_constraint!(FixedHeight;
    |constraints: &LayoutParams, sub: &mut LayoutParams, env: &Environment, value: Length| {
        let value = value.compute(constraints, env);
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
        constraints: &LayoutParams,
        env: &Environment,
    ) -> BoxLayout {
        let mut subconstraints = *constraints;
        if subconstraints.max.width.is_finite() {
            subconstraints.min.width = subconstraints.max.width;
        }
        if subconstraints.max.height.is_finite() {
            subconstraints.min.height = subconstraints.max.height;
        }
        widget.layout(ctx, &subconstraints, env)
    }

    fn debug_node(&self) -> DebugNode {
        DebugNode::new("fill")
    }
}
