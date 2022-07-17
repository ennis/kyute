use crate::{event::PointerEventKind, widget::prelude::*, Signal};

#[derive(Clone)]
pub struct Clickable<Inner> {
    id: WidgetId,
    inner: Inner,
    clicked: Signal<()>,
    active: Signal<bool>,
    hovered: Signal<bool>,
}

impl<Inner: Widget + 'static> Clickable<Inner> {
    #[composable]
    pub fn new(inner: Inner) -> Clickable<Inner> {
        Clickable {
            id: WidgetId::here(),
            inner,
            clicked: Signal::new(),
            active: Signal::new(),
            hovered: Signal::new(),
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

    /// Returns whether this button is active (holding the mouse button over it).
    pub fn activated(&self) -> bool {
        self.active.value() == Some(true)
    }

    /// Returns whether this button is active (holding the mouse button over it).
    pub fn deactivated(&self) -> bool {
        self.active.value() == Some(false)
    }

    pub fn on_activated(self, f: impl FnOnce()) -> Self {
        if self.activated() {
            f();
        }
        self
    }

    pub fn on_deactivated(self, f: impl FnOnce()) -> Self {
        if self.deactivated() {
            f();
        }
        self
    }

    /// Returns whether the pointer entered the clickable area.
    pub fn pointer_entered(&self) -> bool {
        self.hovered.value() == Some(true)
    }

    /// Returns whether the pointer exited the clickable area.
    pub fn pointer_exited(&self) -> bool {
        self.hovered.value() == Some(false)
    }

    pub fn on_pointer_entered(self, f: impl FnOnce()) -> Self {
        if self.pointer_entered() {
            f();
        }
        self
    }

    pub fn on_pointer_exited(self, f: impl FnOnce()) -> Self {
        if self.pointer_exited() {
            f();
        }
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

impl<Inner: Widget + 'static> Widget for Clickable<Inner> {
    fn widget_id(&self) -> Option<WidgetId> {
        Some(self.id)
    }

    fn layout(&self, ctx: &mut LayoutCtx, constraints: &LayoutConstraints, env: &Environment) -> Layout {
        self.inner.layout(ctx, constraints, env)
    }

    fn event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        if let Event::Pointer(p) = event {
            match p.kind {
                PointerEventKind::PointerDown => {
                    ctx.request_focus();
                    ctx.set_handled();
                    ctx.capture_pointer();
                    self.active.signal(true);
                }
                PointerEventKind::PointerUp => {
                    self.active.signal(false);
                    self.clicked.signal(());
                }
                PointerEventKind::PointerOver => {
                    eprintln!("clickable PointerOver");
                    self.hovered.signal(true);
                }
                PointerEventKind::PointerOut => {
                    eprintln!("clickable PointerOut");
                    self.hovered.signal(false);
                }
                _ => {}
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
