use crate::widget::prelude::*;

/*#[derive(Clone)]
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
        self.layer().set_offset(offset);
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
        let bounds = self.measurements.get().local_bounds();

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

    /*fn paint(&self, ctx: &mut PaintCtx, env: &Environment) {
        let m = self.measurements.get();
        let offset = self.offset.get();
        ctx.with_transform_and_clip(offset.to_transform(), m.local_bounds(), m.clip_bounds, |ctx| {
            self.inner.paint(ctx, env);
        });
    }*/

    fn window_paint(&self, ctx: &mut WindowPaintCtx) {
        self.inner.window_paint(ctx);
    }

    fn gpu_frame<'a, 'b>(&'a self, ctx: &mut GpuFrameCtx<'a, 'b>) {
        self.inner.gpu_frame(ctx);
    }
}*/

/// A wrapper around a widget that makes its layout available to the composition step.
#[derive(Clone)]
pub struct LayoutInspector<Inner> {
    inner: Inner,
    size: Size,
    size_changed: Signal<Size>,
}

impl<Inner: Widget + 'static> LayoutInspector<Inner> {
    #[composable]
    pub fn new(inner: Inner) -> LayoutInspector<Inner> {
        #[state]
        let mut size = Size::zero();
        let size_changed = Signal::new();
        if let Some(new_size) = size_changed.value() {
            size = new_size;
        }

        LayoutInspector {
            inner,
            size,
            size_changed,
        }
    }

    /// Returns the current size of the inner widget.
    pub fn size(&self) -> Size {
        self.size
    }

    /// Returns whether the current size of the inner widget has changed since the last composition.
    pub fn size_changed(&self) -> Option<Size> {
        self.size_changed.value()
    }

    /// Calls the given closure if the current size of the inner widget has changed since the last composition.
    pub fn on_size_changed(self, f: impl FnOnce(Size)) -> Self {
        self.size_changed.map(f);
        self
    }

    /// Returns a reference to the inner widget.
    pub fn inner(&self) -> &Inner {
        &self.inner
    }

    /// Returns a mutable reference to the inner widget.
    pub fn inner_mut(&mut self) -> &mut Inner {
        &mut self.inner
    }
}

impl<Inner: Widget + 'static> Widget for LayoutInspector<Inner> {
    fn widget_id(&self) -> Option<WidgetId> {
        self.inner.widget_id()
    }

    fn layout(&self, ctx: &mut LayoutCtx, constraints: &LayoutParams, env: &Environment) -> Layout {
        let layout = self.inner.layout(ctx, constraints, env);
        if layout.measurements.size != self.size {
            self.size_changed.signal(layout.measurements.size);
        }
        layout
    }

    fn event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        self.inner.route_event(ctx, event, env)
    }

    fn paint(&self, ctx: &mut PaintCtx) {
        self.inner.paint(ctx)
    }
}
