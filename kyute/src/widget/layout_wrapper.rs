use crate::{
    cache::state,
    core::{HitTestResult, WindowPaintCtx},
    event::{PointerEvent, PointerEventKind},
    widget::prelude::*,
    GpuFrameCtx, State,
};
use std::cell::Cell;

#[derive(Clone)]
pub struct LayoutWrapper<W> {
    inner: W,
    offset: Cell<Offset>,
    measurements: Cell<Measurements>,
    /// Whether the inner element is hovered.
    pointer_over: State<bool>,
}

impl<W> LayoutWrapper<W> {
    #[composable]
    pub fn new(inner: W) -> LayoutWrapper<W> {
        LayoutWrapper {
            inner,
            offset: Default::default(),
            measurements: Default::default(),
            pointer_over: state(|| false),
        }
    }

    #[composable]
    pub fn with_offset(offset: impl Into<Offset>, inner: W) -> LayoutWrapper<W> {
        let mut w = Self::new(inner);
        w.set_offset(offset.into());
        w
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

    pub fn into_inner(self) -> W {
        self.inner
    }
}

impl<W: Widget> Widget for LayoutWrapper<W> {
    fn widget_id(&self) -> Option<WidgetId> {
        // inherit the identity of the contents
        self.inner.widget_id()
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

        ctx.with_local_transform(self.offset.get().to_transform(), event, |ctx, event| match event {
            Event::Pointer(p) => match ctx.hit_test(p, bounds) {
                HitTestResult::Passed => {
                    let was_over = self.pointer_over.replace(true);
                    if !was_over {
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
                    let was_over = self.pointer_over.replace(false);
                    if was_over {
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

    fn paint(&self, ctx: &mut PaintCtx, env: &Environment) {
        let m = self.measurements.get();
        let offset = self.offset.get();
        ctx.with_transform_and_clip(offset.to_transform(), m.bounds, m.clip_bounds, |ctx| {
            self.inner.paint(ctx, env);
        });
    }

    fn window_paint(&self, ctx: &mut WindowPaintCtx) {
        self.inner.window_paint(ctx);
    }

    fn gpu_frame<'a, 'b>(&'a self, ctx: &mut GpuFrameCtx<'a, 'b>) {
        self.inner.gpu_frame(ctx);
    }
}

/// A wrapper widget that makes the result of its layout available to the composition step.
#[derive(Clone)]
pub struct LayoutInspector<Content> {
    content: Content,
    size: Size,
    size_changed: Signal<Size>,
}

impl<Content: Widget + 'static> LayoutInspector<Content> {
    #[composable]
    pub fn new(content: Content) -> LayoutInspector<Content> {
        #[state]
        let mut size = Size::zero();
        let size_changed = Signal::new();
        if let Some(new_size) = size_changed.value() {
            size = new_size;
        }

        LayoutInspector {
            content,
            size,
            size_changed,
        }
    }

    /// Returns the current size of the thing.
    pub fn size(&self) -> Size {
        self.size
    }

    pub fn size_changed(&self) -> Option<Size> {
        self.size_changed.value()
    }

    pub fn on_size_changed(self, f: impl FnOnce(Size)) -> Self {
        self.size_changed.map(f);
        self
    }

    /// Returns a reference to the inner widget.
    pub fn contents(&self) -> &Content {
        &self.content
    }

    /// Returns a mutable reference to the inner widget.
    pub fn contents_mut(&mut self) -> &mut Content {
        &mut self.content
    }
}

impl<Content: Widget + 'static> Widget for LayoutInspector<Content> {
    fn widget_id(&self) -> Option<WidgetId> {
        self.content.widget_id()
    }

    fn event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        self.content.event(ctx, event, env)
    }

    fn layout(&self, ctx: &mut LayoutCtx, constraints: BoxConstraints, env: &Environment) -> Measurements {
        let measurements = self.content.layout(ctx, constraints, env);
        if measurements.bounds.size != self.size {
            self.size_changed.signal(measurements.bounds.size);
        }
        measurements
    }

    fn paint(&self, ctx: &mut PaintCtx, env: &Environment) {
        self.content.paint(ctx, env)
    }
}
