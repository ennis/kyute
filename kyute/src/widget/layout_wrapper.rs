use crate::{
    core2::WindowPaintCtx, BoxConstraints, Environment, Event, EventCtx, GpuFrameCtx, LayoutCtx,
    Measurements, Offset, PaintCtx, Rect, Widget,
};
use kyute_shell::drawing::ToSkia;
use std::cell::Cell;

pub struct LayoutWrapper<W> {
    inner: W,
    offset: Cell<Offset>,
    measurements: Cell<Measurements>,
}

impl<W> LayoutWrapper<W> {
    pub fn new(inner: W) -> LayoutWrapper<W> {
        LayoutWrapper {
            inner,
            offset: Default::default(),
            measurements: Default::default(),
        }
    }

    pub fn set_offset(&self, offset: Offset) {
        self.offset.set(offset);
    }
}

impl<W: Widget> Widget for LayoutWrapper<W> {
    fn debug_name(&self) -> &str {
        self.inner.debug_name()
    }

    fn event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        // translate pointer event to local coords
        event.apply_offset(-self.offset.get());
        // FIXME: need to reject the event if not in bounds of the inner widget
        // ALSO: pointerin, pointerout, etc.
        // TODO: maybe just use a widgetpod for that? the issue with widgetpods are potential overhead
        // (one arc allocation per layout wrapper)
        self.inner.event(ctx, event, env);
        event.apply_offset(self.offset.get());
    }

    fn layout(
        &self,
        ctx: &mut LayoutCtx,
        constraints: BoxConstraints,
        env: &Environment,
    ) -> Measurements {
        let m = self.inner.layout(ctx, constraints, env);
        self.measurements.set(m);
        m
    }

    fn paint(&self, ctx: &mut PaintCtx, bounds: Rect, env: &Environment) {
        // TODO cleanup duplication: ctx.measurements() and bounds
        ctx.canvas.save();
        ctx.canvas.translate(self.offset.get().to_skia());
        ctx.measurements = self.measurements.get();
        self.inner.paint(ctx, self.measurements.get().bounds, env);
        ctx.canvas.restore();
    }

    fn window_paint(&self, ctx: &mut WindowPaintCtx) {
        self.inner.window_paint(ctx);
    }

    fn gpu_frame<'a, 'b>(&'a self, ctx: &mut GpuFrameCtx<'a, 'b>) {
        self.inner.gpu_frame(ctx);
    }
}
