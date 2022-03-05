use crate::{
    cache,
    event::PointerEventKind,
    widget::{prelude::*, LayoutWrapper},
    Signal, State,
};
use std::cell::Cell;

#[derive(Clone)]
pub struct Thumb<Content> {
    id: WidgetId,
    content: Content,
    drag_started: Signal<Point>,
    drag_delta: Signal<Point>,
    drag_completed: Signal<Point>,
}

impl<Content: Widget + 'static> Thumb<Content> {
    #[composable]
    pub fn new(content: Content) -> Thumb<Content> {
        /*#[state]
        let mut position = Point::zero();
        #[state]
        let mut offset: Option<Offset> = None;

        let actual_position = if let Some(offset) = offset {
            position + offset
        } else {
            position
        };*/

        /*if let Some(at) = thumb.gesture_started() {
            position = at;
            offset = Some(Offset::zero());
        }

        if let Some(ref mut offset) = offset {
            if let Some(at) = thumb.gesture_moved() {
                *offset = at - position;
            }
        }

        if let Some(at) = thumb.gesture_ended() {
            position = at;
            offset = None;
        }*/

        Thumb {
            id: WidgetId::here(),
            content,
            drag_started: Signal::new(),
            drag_delta: Signal::new(),
            drag_completed: Signal::new(),
        }
    }

    pub fn drag_started(&self) -> Option<Point> {
        self.drag_started.value()
    }

    pub fn drag_delta(&self) -> Option<Point> {
        self.drag_delta.value()
    }

    pub fn drag_completed(&self) -> Option<Point> {
        self.drag_completed.value()
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

impl<Content: Widget + 'static> Widget for Thumb<Content> {
    fn widget_id(&self) -> Option<WidgetId> {
        Some(self.id)
    }

    fn event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        match event {
            Event::Pointer(p) => match p.kind {
                PointerEventKind::PointerDown => {
                    // start drag gesture
                    self.drag_started.signal(ctx, p.position);
                    ctx.capture_pointer();
                    ctx.request_redraw();
                    ctx.set_handled();
                }
                PointerEventKind::PointerMove => {
                    self.drag_delta.signal(ctx, p.position);
                    ctx.request_redraw();
                    ctx.set_handled();
                }
                PointerEventKind::PointerUp => {
                    self.drag_completed.signal(ctx, p.position);
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

    fn paint(&self, ctx: &mut PaintCtx, bounds: Rect, transform: Transform, env: &Environment) {
        self.content.paint(ctx, bounds, transform, env);
    }
}
