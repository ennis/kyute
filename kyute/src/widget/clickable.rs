use crate::{event::PointerEventKind, widget::prelude::*, Signal};

#[derive(Clone)]
pub struct Clickable<Content> {
    id: WidgetId,
    content: Content,
    clicked: Signal<()>,
}

impl<Content: Widget + 'static> Clickable<Content> {
    #[composable]
    pub fn new(content: Content) -> Clickable<Content> {
        Clickable {
            id: WidgetId::here(),
            content,
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
    pub fn content(&self) -> &Content {
        &self.content
    }

    /// Returns a mutable reference to the inner widget.
    pub fn content_mut(&mut self) -> &mut Content {
        &mut self.content
    }
}

impl<Content: Widget + 'static> Widget for Clickable<Content> {
    fn widget_id(&self) -> Option<WidgetId> {
        Some(self.id)
    }

    fn layout(&self, ctx: &mut LayoutCtx, constraints: BoxConstraints, env: &Environment) -> Measurements {
        self.content.layout(ctx, constraints, env)
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
            self.content.route_event(ctx, event, env);
        }
    }

    fn paint(&self, ctx: &mut PaintCtx) {
        self.content.paint(ctx)
    }
}
