use crate::{
    core2::WindowPaintCtx,
    event::{PointerEvent, PointerEventKind},
    BoxConstraints, Environment, Event, EventCtx, GpuFrameCtx, LayoutCtx, Measurements, Offset,
    PaintCtx, Rect, Widget,
};
use kyute_shell::drawing::ToSkia;
use std::cell::Cell;

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
    fn debug_name(&self) -> &str {
        self.inner.debug_name()
    }

    fn event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        // Perform our own hit-test on the inner element.
        // This is basically the same logic than what is done in `WidgetPod::event`.
        //
        // NOTE: If we end up here before layout, the bounds may not be valid, so in theory the hit-test may fail,
        // But since the only events sent before layout should be non-pointer events, which always pass
        // the hit-test, that's not a problem.
        let bounds = self.measurements.get().bounds.translate(self.offset.get());
        let hit_test = ctx.hit_test(event, bounds);

        // send potential pointerover/pointerout events
        if let Some(relative_pointer_event) = hit_test.relative_pointer_event {
            if hit_test.pass {
                // Pointer hit-test pass: send pointerover; set flag that tells we're hovering the inner element.
                if !self.pointer_over.get() {
                    self.pointer_over.set(true);
                    self.do_event(
                        parent_ctx,
                        &mut Event::Pointer(PointerEvent {
                            kind: PointerEventKind::PointerOver,
                            ..relative_pointer_event
                        }),
                        env,
                    );
                }
            } else {
                // pointer hit-test fail; if we were hovering the element, send pointerout
                if self.pointer_over.get() {
                    self.pointer_over.set(false);
                    self.do_event(
                        parent_ctx,
                        &mut Event::Pointer(PointerEvent {
                            kind: PointerEventKind::PointerOut,
                            ..adjusted_pointer_event
                        }),
                        env,
                    );
                }
            }
        }

        // deliver event
        if hit_test.pass {
            if let Some(mut relative_pointer_event) = hit_test.relative_pointer_event {
                self.inner
                    .event(ctx, &mut Event::Pointer(relative_pointer_event), env);
            } else {
                self.inner.event(ctx, event, env);
            }
        }
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
