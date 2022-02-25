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

    fn debug_name(&self) -> &str {
        std::any::type_name::<Self>()
    }

    fn event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        match event {
            Event::Pointer(p) => match p.kind {
                PointerEventKind::PointerDown => {
                    self.clicked.signal(ctx, ());
                    ctx.request_focus();
                    ctx.request_redraw();
                    ctx.set_handled();
                }
                _ => {}
            },
            _ => {}
        }

        if !ctx.handled() {
            self.content.event(ctx, event, env);
        }
    }

    fn layout(&self, ctx: &mut LayoutCtx, constraints: BoxConstraints, env: &Environment) -> Measurements {
        self.content.layout(ctx, constraints, env)
    }

    fn paint(&self, ctx: &mut PaintCtx, bounds: Rect, env: &Environment) {
        self.content.paint(ctx, bounds, env);
    }
}
