use crate::{
    cache,
    event::{PointerButton, PointerButtons, PointerEventKind},
    widget::prelude::*,
    Signal, State,
};

/// Widget that provides feedback on the pointer position and status when the pointer is over it.
#[derive(Clone)]
pub struct Thumb<Content> {
    id: WidgetId,
    content: Content,
    pointer_buttons: PointerButtons,
    pointer_down: Signal<(Point, Transform)>,
    pointer_move: Signal<Point>,
    pointer_up: Signal<Point>,
    scrolled: Signal<Offset>,
}

impl<Content: Widget + 'static> Thumb<Content> {
    /// Creates a new Thumb widget over the specified content.
    #[composable]
    pub fn new(content: Content) -> Thumb<Content> {
        Thumb {
            id: WidgetId::here(),
            content,
            pointer_buttons: PointerButtons::ALL,
            pointer_down: Signal::new(),
            pointer_move: Signal::new(),
            pointer_up: Signal::new(),
            scrolled: Signal::new(),
        }
    }

    pub fn pointer_button_filter(mut self, buttons: PointerButtons) -> Self {
        self.pointer_buttons = buttons;
        self
    }

    pub fn pointer_down(&self) -> Option<(Point, Transform)> {
        self.pointer_down.value()
    }

    pub fn on_pointer_down(self, f: impl FnOnce((Point, Transform))) -> Self {
        self.pointer_down.map(f);
        self
    }

    pub fn pointer_moved(&self) -> Option<Point> {
        self.pointer_move.value()
    }

    pub fn on_pointer_moved(self, f: impl FnOnce(Point)) -> Self {
        self.pointer_move.map(f);
        self
    }

    pub fn pointer_up(&self) -> Option<Point> {
        self.pointer_up.value()
    }

    pub fn on_pointer_up(self, f: impl FnOnce(Point)) -> Self {
        self.pointer_up.map(f);
        self
    }

    pub fn scrolled(&self) -> Option<Offset> {
        self.scrolled.value()
    }

    pub fn on_scrolled(self, f: impl FnOnce(Offset)) -> Self {
        self.scrolled.map(f);
        self
    }

    /// Returns a reference to the inner widget.
    pub fn content(&self) -> &Content {
        &self.content
    }

    /// Returns a mutable reference to the inner widgets.
    pub fn content_mut(&mut self) -> &mut Content {
        &mut self.content
    }
}

impl<Content: Widget + 'static> Widget for Thumb<Content> {
    fn widget_id(&self) -> Option<WidgetId> {
        Some(self.id)
    }

    fn layout(&self, ctx: &mut LayoutCtx, constraints: &LayoutParams, env: &Environment) -> Geometry {
        self.content.layout(ctx, constraints, env)
    }

    fn event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        self.content.route_event(ctx, event, env);

        if !ctx.handled {
            match event {
                Event::Pointer(p) => match p.kind {
                    PointerEventKind::PointerDown => {
                        if self.pointer_buttons.test(p.button.unwrap()) {
                            self.pointer_down.signal((p.window_position, *ctx.window_transform()));
                            ctx.capture_pointer();
                            ctx.set_handled();
                        }
                    }
                    PointerEventKind::PointerMove => {
                        self.pointer_move.signal(p.window_position);
                        ctx.set_handled();
                    }
                    PointerEventKind::PointerUp => {
                        if self.pointer_buttons.test(p.button.unwrap()) {
                            self.pointer_up.signal(p.window_position);
                            ctx.set_handled();
                        }
                    }
                    _ => {}
                },
                Event::Wheel(wheel) => self.scrolled.signal(Offset::new(wheel.delta_x, wheel.delta_y)),
                _ => {}
            }
        }
    }

    fn paint(&self, ctx: &mut PaintCtx) {
        self.content.paint(ctx)
    }
}

// alternate design for the dragcontroller:
//
// - being_drag(start_value)
// - dragging(impl FnOnce(start_value, offset))

pub struct DragController<T, Content> {
    content: Thumb<Content>,
    started: bool,
    delta: Option<Offset>,
    completed: bool,
    start_value: T,
}

impl<T: Clone + 'static, Content: Widget + 'static> DragController<T, Content> {
    #[composable]
    pub fn new(value: T, content: Content) -> DragController<T, Content> {
        #[state]
        let mut anchor: Option<(Point, Transform)> = None;

        let mut start_value = cache::state(|| value.clone());

        let mut delta = None;
        let mut started = false;
        let mut completed = false;

        let thumb = Thumb::new(content).pointer_button_filter(PointerButtons::new().with(PointerButton::LEFT));

        if let Some(p) = thumb.pointer_down() {
            anchor = Some(p);
            started = true;
            start_value.set_without_invalidation(value);
        }

        if let Some(p) = thumb.pointer_moved() {
            if let Some((anchor_point, anchor_transform)) = anchor {
                delta = Some(anchor_transform.transform_vector(p - anchor_point));
            }
        }

        if let Some(_p) = thumb.pointer_up() {
            anchor = None;
            completed = true;
        }

        DragController {
            start_value: start_value.get(),
            content: thumb,
            started,
            delta,
            completed,
        }
    }

    pub fn started(&self) -> bool {
        self.started
    }

    pub fn on_started(self, f: impl FnOnce()) -> Self {
        if self.started {
            f()
        }
        self
    }

    pub fn delta(&self) -> Option<Offset> {
        self.delta
    }

    pub fn on_delta(self, f: impl FnOnce(T, Offset)) -> Self {
        if let Some(delta) = self.delta {
            f(self.start_value.clone(), delta)
        }
        self
    }

    pub fn completed(&self) -> bool {
        self.completed
    }

    pub fn on_completed(self, f: impl FnOnce()) -> Self {
        if self.completed {
            f()
        }
        self
    }

    pub fn content(&self) -> &Content {
        self.content.content()
    }

    pub fn content_mut(&mut self) -> &mut Content {
        self.content.content_mut()
    }
}

impl<T, Content: Widget + 'static> Widget for DragController<T, Content> {
    fn widget_id(&self) -> Option<WidgetId> {
        self.content.widget_id()
    }

    fn layout(&self, ctx: &mut LayoutCtx, constraints: &LayoutParams, env: &Environment) -> Geometry {
        self.content.layout(ctx, constraints, env)
    }

    fn event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        self.content.route_event(ctx, event, env);
    }

    fn paint(&self, ctx: &mut PaintCtx) {
        self.content.paint(ctx)
    }
}
