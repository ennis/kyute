use crate::{
    align_boxes, cache, composable,
    core2::{EventCtx, LayoutCtx, PaintCtx},
    event::PointerEventKind,
    widget::Text,
    Alignment, BoxConstraints, Cache, Environment, Event, Key, Measurements, Rect, SideOffsets,
    Size, Widget, WidgetPod,
};
use tracing::trace;
use crate::state::Signal;

#[derive(Clone)]
pub struct Clickable<Content> {
    content: WidgetPod<Content>,
    clicked: Signal<()>,
}

impl<Content: Widget + 'static> Clickable<Content> {
    #[composable(uncached)]
    pub fn new(content: Content) -> Clickable<Content> {
        Clickable {
            content: WidgetPod::new(content),
            clicked: Signal::new(),
        }
    }

    /// Returns whether this button has been clicked.
    pub fn clicked(&self) -> bool {
        self.clicked.signalled()
    }
}

impl<Content: Widget + 'static> Widget for Clickable<Content> {
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

    fn layout(
        &self,
        ctx: &mut LayoutCtx,
        constraints: BoxConstraints,
        env: &Environment,
    ) -> Measurements {
        self.content.layout(ctx, constraints, env)
    }

    fn paint(&self, ctx: &mut PaintCtx, bounds: Rect, env: &Environment) {
        self.content.paint(ctx, bounds, env);
    }
}
