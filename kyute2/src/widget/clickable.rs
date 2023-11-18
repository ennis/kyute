//! Clickable widget wrapper
use std::mem;

use keyboard_types::{Key, KeyState};
use tracing::trace;

use crate::{
    context::Ambient,
    widget::{prelude::*, WidgetState},
};

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
            on_clicked: |_cx| {
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
        let inner = cx.with_ambient(&WidgetState::default(), |cx| cx.build(self.inner));
        ClickableElement {
            id,
            content: inner,
            state: Default::default(),
            events: Default::default(),
            hovered: false,
        }
    }

    fn update(self, cx: &mut TreeCtx, element: &mut Self::Element) -> ChangeFlags {
        //eprintln!("clickable update");
        let events = mem::take(&mut element.events);
        if events.clicked {
            (self.on_clicked)(cx);
        }

        let prev_state = WidgetState::ambient(cx).cloned().unwrap_or_default();
        cx.with_ambient(
            &WidgetState {
                hovered: element.state.hovered,
                active: element.state.active,
                ..prev_state
            },
            |cx| cx.update(self.inner, &mut element.content),
        )
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
    content: Inner,
    state: ClickableState,
    events: ClickableEvents,
    hovered: bool,
}

impl<Inner: Element> Element for ClickableElement<Inner> {
    fn id(&self) -> ElementId {
        self.id
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, constraints: &BoxConstraints) -> Geometry {
        ctx.layout(&mut self.content, constraints)
    }

    fn event(&mut self, ctx: &mut EventCtx, event: &mut Event) -> ChangeFlags {
        let mut change_flags = ChangeFlags::empty();

        // We don't capture anything, so forward to children first.
        if let Some(target) = event.next_target() {
            assert_eq!(self.content.id(), target);
            change_flags |= ctx.event(&mut self.content, event);
        }

        if event.handled {
            return change_flags;
        }

        match event.kind {
            EventKind::PointerDown(ref _p) => {
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
            EventKind::PointerUp(ref _p) => {
                event.handled = true;
                self.state.active = false;
                self.events.activated = Some(false);
                self.events.clicked = true;
                ChangeFlags::PAINT
            }
            EventKind::PointerOver(ref _p) => {
                //eprintln!("clickable PointerOver");
                self.state.hovered = true;
                ChangeFlags::NONE
                //ChangeFlags::PAINT
            }
            EventKind::PointerOut(ref _p) => {
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
                        if self.state.active {
                            self.events.activated = Some(false);
                            self.events.clicked = true;
                            self.state.active = false;
                        }
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

    fn natural_width(&mut self, height: f64) -> f64 {
        self.content.natural_width(height)
    }

    fn natural_height(&mut self, width: f64) -> f64 {
        self.content.natural_height(width)
    }

    fn natural_baseline(&mut self, params: &BoxConstraints) -> f64 {
        self.content.natural_baseline(params)
    }

    fn hit_test(&self, ctx: &mut HitTestResult, position: Point) -> bool {
        let hit = self.content.hit_test(ctx, position);
        if hit {
            ctx.add(self.id);
        }
        hit
    }

    fn paint(&mut self, ctx: &mut PaintCtx) {
        ctx.paint(&mut self.content);
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn debug(&self, visitor: &mut DebugWriter) {
        visitor.type_name("ClickableElement");
        visitor.property("id", self.id);
        visitor.property("state", self.state);
        visitor.property("events", self.events);
        visitor.child("inner", &self.content);
    }
}
