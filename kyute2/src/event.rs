use crate::{Affine, Point, WidgetId};
use std::{cell::RefCell, collections::HashSet};

////////////////////////////////////////////////////////////////////////////////////////////////////

// glazier reexports
pub use glazier::{
    Code as KeyCode, KeyEvent, KeyState, Modifiers, MouseButton, MouseButtons, MouseEvent, PointerButton,
    PointerButtons, PointerType,
};

////////////////////////////////////////////////////////////////////////////////////////////////////

/// Modeled after [W3C's PointerEvent](https://www.w3.org/TR/pointerevents3/#pointerevent-interface)
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct PointerEvent {
    /// The widget for which this event is intended. Can be `None` if the target is not known, and determined on the fly by hit-testing.
    pub target: Option<WidgetId>,
    /// Position in device-independent (logical) pixels, relative to the parent window.
    pub position: Point,
    /// State of the keyboard modifiers when this event was emitted.
    pub modifiers: Modifiers,
    /// The state of the mouse buttons when this event was emitted.
    pub buttons: PointerButtons,
    /// The button that triggered this event, if there is one.
    pub button: PointerButton,
    /// The repeat count for double, triple (and more) for button press events (`Event::PointerDown`).
    /// Otherwise, the value is unspecified.
    pub repeat_count: u8,
    /// Global-to-local transform
    pub transform: Affine,
}

impl PointerEvent {
    /// Converts from `glazier::PointerEvent`.
    pub fn from_glazier(event: &glazier::PointerEvent) -> PointerEvent {
        PointerEvent {
            target: None,
            position: event.pos,
            modifiers: event.modifiers,
            buttons: event.buttons,
            button: event.button,
            repeat_count: event.count,
            transform: Default::default(),
        }
    }

    /// Local position
    pub fn local_position(&self) -> Point {
        self.transform * self.position
    }

    pub fn transformed(mut self, transform: Affine) -> PointerEvent {
        let transform = self.transform * transform;
        PointerEvent { transform, ..self }
    }
}

#[derive(Debug)]
pub enum InternalEvent<'a> {
    /// Hit-test results
    HitTest {
        position: Point,
        hovered: &'a RefCell<HashSet<WidgetId>>,
        hot: &'a RefCell<Option<WidgetId>>,
    },
}

/// Events.
#[derive(Debug)]
pub enum EventKind<'a> {
    FocusGained,
    FocusLost,
    MenuCommand(usize),
    PointerMove(PointerEvent),
    PointerUp(PointerEvent),
    PointerDown(PointerEvent),
    PointerOver(PointerEvent),
    PointerOut(PointerEvent),
    PointerEnter(PointerEvent),
    PointerExit(PointerEvent),
    /// A keyboard event.
    Keyboard(KeyEvent),
    Internal(InternalEvent<'a>),
}

pub struct Event<'a> {
    pub(crate) route: &'a [WidgetId],
    pub(crate) kind: EventKind<'a>,
}

impl<'a> Event<'a> {
    pub fn next_target(&mut self) -> Option<WidgetId> {
        let (next, rest) = self.route.split_first()?;
        self.route = rest;
        Some(*next)
    }

    pub fn kind(&self) -> &EventKind {
        &self.kind
    }

    fn set_transform(&mut self, transform: &Affine, append: bool) -> Option<Affine> {
        match self.kind {
            EventKind::PointerMove(ref mut pe)
            | EventKind::PointerUp(ref mut pe)
            | EventKind::PointerDown(ref mut pe)
            | EventKind::PointerOver(ref mut pe)
            | EventKind::PointerOut(ref mut pe)
            | EventKind::PointerEnter(ref mut pe)
            | EventKind::PointerExit(ref mut pe) => {
                let prev_transform = pe.transform;
                if append {
                    pe.transform *= *transform;
                } else {
                    pe.transform = *transform;
                }
                Some(prev_transform)
            }
            _ => None,
        }
    }

    pub fn with_transform<R>(&mut self, transform: &Affine, f: impl FnOnce(&mut Event) -> R) -> R {
        let prev_transform = self.set_transform(transform, true);
        let r = f(self);
        if let Some(prev_transform) = prev_transform {
            self.set_transform(&prev_transform, false);
        }
        r
    }
}
