use crate::{
    composable, core::DebugNode, EnvKey, EnvValue, Environment, Event, EventCtx, Geometry, LayoutCtx, LayoutParams,
    PaintCtx, Widget, WidgetId,
};
use bitflags::bitflags;

/// Assigns a debug name to a widget.
pub struct DebugName<W> {
    inner: W,
    name: String,
}

impl<W: Widget> DebugName<W> {
    pub fn new(inner: W, name: String) -> DebugName<W> {
        DebugName { inner, name }
    }
}

impl<W: Widget> Widget for DebugName<W> {
    fn widget_id(&self) -> Option<WidgetId> {
        self.inner.widget_id()
    }

    fn layout(&self, ctx: &mut LayoutCtx, constraints: &LayoutParams, env: &Environment) -> Geometry {
        self.inner.layout(ctx, constraints, env)
    }

    fn event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        self.inner.event(ctx, event, env)
    }

    fn paint(&self, ctx: &mut PaintCtx) {
        self.inner.paint(ctx)
    }

    fn debug_name(&self) -> &str {
        &self.name
    }
}

bitflags! {
    pub struct DebugFlags: u32 {
        /// Dump the widget geometry returned from `Widget::layout`.
        const DUMP_GEOMETRY = 0b00000001;
        /// Dump constraints passed to `Widget::layout`.
        const DUMP_CONSTRAINTS = 0b00000010;
        /// Dump received events.
        const DUMP_EVENTS = 0b00000100;
    }
}

/// Dumps debugging information for the wrapped widget.
pub struct Debug<W> {
    inner: W,
    flags: DebugFlags,
}

impl<W: Widget> Debug<W> {
    pub fn new(inner: W, flags: DebugFlags) -> Debug<W> {
        Debug { inner, flags }
    }
}

impl<W: Widget> Widget for Debug<W> {
    fn widget_id(&self) -> Option<WidgetId> {
        self.inner.widget_id()
    }

    fn layout(&self, ctx: &mut LayoutCtx, constraints: &LayoutParams, env: &Environment) -> Geometry {
        let debug_name = self.inner.debug_name();
        if self.flags.contains(DebugFlags::DUMP_CONSTRAINTS) {
            eprintln!("[{debug_name}] constraints: {constraints:?}");
        }
        let geometry = self.inner.layout(ctx, constraints, env);
        if self.flags.contains(DebugFlags::DUMP_GEOMETRY) {
            eprintln!("[{debug_name}] geometry: {geometry:?}");
        }
        geometry
    }

    fn event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        let debug_name = self.inner.debug_name();
        if self.flags.contains(DebugFlags::DUMP_EVENTS) {
            eprintln!("[{debug_name}] event: {event:?}");
        }
        self.inner.event(ctx, event, env)
    }

    fn paint(&self, ctx: &mut PaintCtx) {
        self.inner.paint(ctx)
    }
}
