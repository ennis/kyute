use crate::{cache, event::PointerEventKind, widget::prelude::*, Signal, State};
use std::cell::Cell;

#[derive(Clone)]
pub struct Thumb<Content> {
    id: WidgetId,
    content: Content,
    gesture_started: Signal<Point>,
    gesture_ended: Signal<Point>,
    position_changed: Signal<Point>,
}

impl<Content: Widget + 'static> Thumb<Content> {
    #[composable]
    pub fn draggable(content: Content, offset: Offset) -> Thumb<Content> {
        #[state]
        let mut anchor_offset: Option<Offset> = None;
        #[state]
        let mut offset = Offset::zero();

        //let anchor_offset = State::new(|| None);
        let offset = State::new(|| Offset::zero());
        let thumb = Thumb::new(content, offset.get());

        /*if let Some(at) = thumb.gesture_started() {
            // on gesture started, set anchor offset
            anchor_offset.set(Some(at));
        }*/

        thumb
    }

    #[composable]
    pub fn new(content: Content, offset: Offset) -> Thumb<Content> {
        Thumb {
            id: WidgetId::here(),
            content,
            gesture_started: Signal::new(),
            gesture_ended: Signal::new(),
            position_changed: Signal::new(),
        }
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

// dragging behavior:
// - on mouse down -> record current pos as anchor pos, set delta to zero
// - on pointer move

impl<Content: Widget + 'static> Widget for Thumb<Content> {
    fn widget_id(&self) -> Option<WidgetId> {
        Some(self.id)
    }

    fn event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        match event {
            Event::Pointer(p) => match p.kind {
                PointerEventKind::PointerDown => {
                    // start drag gesture
                    //ctx.set_state(self.anchor_pos, Some(p.position));
                    ctx.capture_pointer();
                    ctx.request_redraw();
                    ctx.set_handled();
                }
                PointerEventKind::PointerMove => {}
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
