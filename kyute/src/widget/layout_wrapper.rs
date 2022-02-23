use crate::{
    core::{HitTestResult, WidgetIdentity, WindowPaintCtx},
    drawing::ToSkia,
    event::{PointerEvent, PointerEventKind},
    widget::prelude::*,
    GpuFrameCtx,
};
use std::cell::Cell;

#[derive(Clone)]
pub struct LayoutWrapper<W> {
    inner: W,
    offset: Cell<Offset>,
    measurements: Cell<Measurements>,
    /// Whether the inner element is hovered.
    // FIXME: this is destroyed between recomps, we don't want that
    // never mind that, this is invalidated on *relayouts*,
    pointer_over: Cell<bool>,
}

impl<W> LayoutWrapper<W> {
    pub fn new(inner: W) -> LayoutWrapper<W> {
        LayoutWrapper {
            inner,
            offset: Default::default(),
            measurements: Default::default(),
            pointer_over: Cell::new(false),
        }
    }

    pub fn set_offset(&self, offset: Offset) {
        self.offset.set(offset);
    }

    pub fn offset(&self) -> Offset {
        self.offset.get()
    }

    /// Returns a reference to the inner widget.
    pub fn inner(&self) -> &W {
        &self.inner
    }

    /// Returns a mutable reference to the inner widget.
    pub fn inner_mut(&mut self) -> &mut W {
        &mut self.inner
    }
}

impl<W: Widget> Widget for LayoutWrapper<W> {
    fn widget_identity(&self) -> Option<&WidgetIdentity> {
        // inherit the identity of the contents
        self.inner.widget_identity()
    }

    fn debug_name(&self) -> &str {
        self.inner.debug_name()
    }

    fn event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        // Perform our own hit-test on the inner element.
        // This is pretty much the same logic as in `WidgetPod::event`.
        //
        // NOTE: If we end up here before layout, the bounds may not be valid, so in theory the hit-test may fail,
        // But since the only events sent before layout should be non-pointer events, which always pass
        // the hit-test, that's not a problem.
        let bounds = self.measurements.get().bounds;

        event.with_local_coordinates(self.offset.get(), |event| match event {
            Event::Pointer(p) => match ctx.hit_test(p, bounds) {
                HitTestResult::Passed => {
                    if !self.pointer_over.get() {
                        self.pointer_over.set(true);
                        self.inner.event(
                            ctx,
                            &mut Event::Pointer(PointerEvent {
                                kind: PointerEventKind::PointerOver,
                                ..*p
                            }),
                            env,
                        );
                    }
                    self.inner.event(ctx, event, env);
                }
                HitTestResult::Failed => {
                    if self.pointer_over.get() {
                        self.pointer_over.set(false);
                        self.inner.event(
                            ctx,
                            &mut Event::Pointer(PointerEvent {
                                kind: PointerEventKind::PointerOut,
                                ..*p
                            }),
                            env,
                        );
                    }
                }
                HitTestResult::Skipped => {
                    self.inner.event(ctx, event, env);
                }
            },
            _ => {
                self.inner.event(ctx, event, env);
            }
        });
    }

    fn layout(&self, ctx: &mut LayoutCtx, constraints: BoxConstraints, env: &Environment) -> Measurements {
        let m = self.inner.layout(ctx, constraints, env);
        self.measurements.set(m);
        m
    }

    fn paint(&self, ctx: &mut PaintCtx, _bounds: Rect, env: &Environment) {
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
