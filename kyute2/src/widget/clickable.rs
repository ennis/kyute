//! Clickable widget wrapper
use crate::widget::prelude::*;
use keyboard_types::{Key, KeyState};
use std::mem;
use tracing::trace;

type DefaultClickHandler = fn(&mut TreeCtx);

/// Wraps an inner widget and allows the user to respond to clicks on it.
#[derive(Clone)]
pub struct Clickable<T, OnClicked = DefaultClickHandler> {
    inner: T,
    on_clicked: OnClicked, // default value?
}

impl<T> Clickable<T, DefaultClickHandler> {
    pub fn new(inner: T) -> Clickable<T, DefaultClickHandler> {
        Clickable {
            inner,
            on_clicked: |cx| {
                trace!("Clickable::on_clicked");
            },
        }
    }

    #[must_use]
    pub fn on_clicked<OnClicked>(self, on_clicked: OnClicked) -> Clickable<T, OnClicked>
    where
        OnClicked: FnOnce(&mut TreeCtx),
    {
        Clickable {
            inner: self.inner,
            on_clicked,
        }
    }
}

impl<T, OnClicked> Widget for Clickable<T, OnClicked>
where
    T: Widget,
    OnClicked: FnOnce(&mut TreeCtx),
{
    type Element = ClickableElement<T::Element>;

    fn build(self, cx: &mut TreeCtx, id: ElementId) -> Self::Element {
        eprintln!("clickable rebuilt");
        ClickableElement {
            id,
            inner: cx.build(self.inner),
            state: Default::default(),
            events: Default::default(),
            hovered: false,
        }
    }

    fn update(self, cx: &mut TreeCtx, element: &mut Self::Element) -> ChangeFlags {
        eprintln!("clickable update");
        let events = mem::take(&mut element.events);
        if events.clicked {
            (self.on_clicked)(cx);
        }
        self.inner.update(cx, &mut element.inner)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Default)]
struct ClickableState {
    active: bool,
    focus: bool,
    hovered: bool,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Default)]
struct ClickableEvents {
    focused: Option<bool>,
    clicked: bool,
    activated: Option<bool>,
}

pub struct ClickableElement<Inner> {
    id: ElementId,
    inner: Inner,
    state: ClickableState,
    events: ClickableEvents,
    hovered: bool,
}

impl<Inner: Element> Element for ClickableElement<Inner> {
    fn id(&self) -> ElementId {
        self.id
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, params: &LayoutParams) -> Geometry {
        self.inner.layout(ctx, params)
    }

    fn event(&mut self, ctx: &mut EventCtx, event: &mut Event) -> ChangeFlags {
        let mut change_flags = ChangeFlags::empty();

        // We don't capture anything, so forward to children first.
        if let Some(target) = event.next_target() {
            assert_eq!(self.inner.id(), target);
            change_flags |= self.inner.event(ctx, event);
        }

        if event.handled {
            return change_flags;
        }

        match event.kind {
            EventKind::PointerDown(ref p) => {
                eprintln!("clickable PointerDown");
                ctx.request_focus(self.id);
                ctx.request_pointer_capture(self.id);
                event.handled = true;

                self.state.active = true;
                self.events.activated = Some(true);

                // TODO: for now return "PAINT" because we make the assumption that the only
                // thing affected by widget state is the painting. Notably, we assume that layout
                // doesn't depend on the widget state.
                ChangeFlags::PAINT
            }
            EventKind::PointerUp(ref p) => {
                event.handled = true;
                self.state.active = false;
                self.events.activated = Some(false);
                self.events.clicked = true;
                ChangeFlags::PAINT
            }
            EventKind::PointerOver(ref p) => {
                //eprintln!("clickable PointerOver");
                self.state.hovered = true;
                ChangeFlags::NONE
                //ChangeFlags::PAINT
            }
            EventKind::PointerOut(ref p) => {
                //eprintln!("clickable PointerOut");
                self.state.hovered = false;
                ChangeFlags::NONE
                //ChangeFlags::PAINT
            }
            EventKind::Keyboard(ref key) => {
                match key.state {
                    KeyState::Down => {
                        let press = match key.key {
                            Key::Enter => true,
                            Key::Character(ref s) if s == " " => true,
                            _ => false,
                        };

                        if press {
                            event.handled = true;
                            self.state.active = true;
                            self.events.activated = Some(true);
                            self.events.clicked = true;
                        }

                        /*if key.key == Key::Tab {
                            if key.modifiers.contains(Modifiers::SHIFT) {
                                ctx.focus_prev();
                            } else {
                                ctx.focus_next();
                            }
                        }*/
                        ChangeFlags::PAINT
                    }
                    KeyState::Up => {
                        self.state.active = false;
                        self.events.activated = Some(false);
                        self.events.clicked = true;
                        ChangeFlags::PAINT
                    }
                }
            }
            /*EventKind::FocusGained => {
                eprintln!("clickable FocusGained");
                self.focus.set(true);
                self.focused.signal(true);
                ctx.request_relayout();
            }
            EventKind::FocusLost => {
                eprintln!("clickable FocusLost");
                self.focus.set(false);
                self.focused.signal(false);
                ctx.request_relayout();
            }*/
            _ => ChangeFlags::NONE,
        }
    }

    fn natural_size(&mut self, axis: Axis, params: &LayoutParams) -> f64 {
        self.inner.natural_size(axis, params)
    }

    fn natural_baseline(&mut self, params: &LayoutParams) -> f64 {
        self.inner.natural_baseline(params)
    }

    fn hit_test(&self, ctx: &mut HitTestResult, position: Point) -> bool {
        let hit = self.inner.hit_test(ctx, position);
        if hit {
            ctx.add(self.id);
        }
        hit
    }

    fn paint(&mut self, ctx: &mut PaintCtx) {
        self.inner.paint(ctx)
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn debug(&self, visitor: &mut DebugWriter) {
        visitor.type_name("ClickableElement");
        visitor.property("id", self.id);
        visitor.property("state", self.state);
        visitor.property("events", self.events);
        visitor.child("inner", &self.inner);
    }
}
