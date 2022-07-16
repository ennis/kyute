use crate::{
    composable, core::DebugNode, EnvKey, EnvValue, Environment, Event, EventCtx, Layout, LayoutConstraints, LayoutCtx,
    PaintCtx, Widget, WidgetId,
};

pub struct EnvOverride<W> {
    inner: W,
    env: Environment,
}

impl<W: Widget> EnvOverride<W> {
    #[composable]
    pub fn new(inner: W) -> EnvOverride<W> {
        EnvOverride {
            inner,
            env: Environment::new(),
        }
    }

    #[must_use]
    pub fn with<T: EnvValue>(mut self, key: EnvKey<T>, value: T) -> EnvOverride<W> {
        self.env.set(key, value);
        self
    }
}

impl<W: Widget> Widget for EnvOverride<W> {
    fn widget_id(&self) -> Option<WidgetId> {
        self.inner.widget_id()
    }

    fn layout(&self, ctx: &mut LayoutCtx, constraints: &LayoutConstraints, env: &Environment) -> Layout {
        let merged_env = env.merged(self.env.clone());
        self.inner.layout(ctx, constraints, &merged_env)
    }

    fn event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        self.inner.event(ctx, event, env)
    }

    fn paint(&self, ctx: &mut PaintCtx) {
        self.inner.paint(ctx)
    }

    fn debug_node(&self) -> DebugNode {
        DebugNode::new(format!("{:?}", self.env))
    }
}
