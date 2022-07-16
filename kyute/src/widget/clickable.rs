use crate::{event::PointerEventKind, widget::prelude::*, Signal};

#[derive(Clone)]
pub struct Clickable<Inner> {
    id: WidgetId,
    inner: Inner,
    clicked: Signal<()>,
}

impl<Inner: Widget + 'static> Clickable<Inner> {
    #[composable]
    pub fn new(inner: Inner) -> Clickable<Inner> {
        Clickable {
            id: WidgetId::here(),
            inner,
            clicked: Signal::new(),
        }
    }

    #[cfg_attr(debug_assertions, track_caller)]
    pub fn on_click(self, f: impl FnOnce()) -> Self {
        if self.clicked.signalled() {
            f();
        }
        self
    }

    /// Returns whether this button has been clicked.
    pub fn clicked(&self) -> bool {
        self.clicked.signalled()
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

impl<Inner: Widget + 'static> Widget for Clickable<Inner> {
    fn widget_id(&self) -> Option<WidgetId> {
        Some(self.id)
    }

    fn layout(&self, ctx: &mut LayoutCtx, constraints: &LayoutConstraints, env: &Environment) -> Layout {
        self.inner.layout(ctx, constraints, env)
    }

    fn event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        if let Event::Pointer(p) = event {
            if p.kind == PointerEventKind::PointerDown {
                self.clicked.signal(());
                ctx.request_focus();
                //ctx.request_redraw();
                ctx.set_handled();
            }
        }

        if !ctx.handled() {
            self.inner.route_event(ctx, event, env);
        }
    }

    fn paint(&self, ctx: &mut PaintCtx) {
        self.inner.paint(ctx)
    }
}
