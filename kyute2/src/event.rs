use crate::{keyboard_types::Modifiers, Affine, Point, WidgetId};
use std::{cell::RefCell, collections::HashSet, fmt};
use tracing::warn;

pub use crate::keyboard_types::KeyboardEvent;

/// Represents the type of pointer.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum PointerType {
    Mouse,
    Pen,
    Stylus,
    Other,
}

/// Represents a pointer button.
// TODO why u no bitflags?
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct PointerButton(pub u16);

impl PointerButton {
    pub const LEFT: PointerButton = PointerButton(0); // Or touch/pen contact
    pub const MIDDLE: PointerButton = PointerButton(1);
    pub const RIGHT: PointerButton = PointerButton(2); // Or pen barrel
    pub const X1: PointerButton = PointerButton(3);
    pub const X2: PointerButton = PointerButton(4);
    pub const ERASER: PointerButton = PointerButton(5);
}

/// The state of the mouse buttons.
// TODO why u no bitflags?
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct PointerButtons(pub u32);

impl PointerButtons {
    pub const ALL: PointerButtons = PointerButtons(0xFFFFFFFF);

    pub fn new() -> PointerButtons {
        PointerButtons(0)
    }

    pub fn with(self, button: PointerButton) -> Self {
        PointerButtons(self.0 | (1u32 << button.0 as u32))
    }

    /// Checks if the specified mouse button is pressed.
    pub fn test(self, button: PointerButton) -> bool {
        self.0 & (1u32 << button.0 as u32) != 0
    }
    pub fn set(&mut self, button: PointerButton) {
        self.0 |= 1u32 << button.0 as u32;
    }
    pub fn reset(&mut self, button: PointerButton) {
        self.0 &= !(1u32 << button.0 as u32);
    }
    pub fn intersects(&self, buttons: PointerButtons) -> bool {
        (self.0 & buttons.0) != 0
    }
    pub fn is_empty(&self) -> bool {
        self.0 == 0
    }
}

impl fmt::Debug for PointerButtons {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{{")?;
        if self.test(PointerButton::LEFT) {
            write!(f, "LEFT")?;
        }
        if self.test(PointerButton::RIGHT) {
            write!(f, "RIGHT")?;
        }
        if self.test(PointerButton::MIDDLE) {
            write!(f, "MIDDLE")?;
        }
        if self.test(PointerButton::X1) {
            write!(f, "X1")?;
        }
        if self.test(PointerButton::X2) {
            write!(f, "X2")?;
        }
        write!(f, " +{:04x}", self.0)?;
        write!(f, "}}")?;
        Ok(())
    }
}

impl Default for PointerButtons {
    fn default() -> Self {
        PointerButtons::new()
    }
}

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
    pub button: Option<PointerButton>,
    /// The repeat count for double, triple (and more) for button press events (`Event::PointerDown`).
    /// Otherwise, the value is unspecified.
    pub repeat_count: u8,
    /// Global-to-local transform
    pub transform: Affine,
}

impl PointerEvent {
    /*/// Converts from `glazier::PointerEvent`.
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
    }*/

    /// Local position
    pub fn local_position(&self) -> Point {
        self.transform.inverse() * self.position
    }

    pub fn transformed(mut self, transform: Affine) -> PointerEvent {
        let transform = self.transform * transform;
        PointerEvent { transform, ..self }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

/*/// Keyboard event.
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct KeyboardEvent {
    pub state: ElementState,
    pub key: Key,
    pub physical_key: PhysicalKey,
    pub text: Option<SmolStr>,
    pub location: KeyLocation,
    pub modifiers: Modifiers,
    pub repeat: bool,
    pub is_composing: bool,
}*/

/*#[derive(Debug)]
pub enum InternalEvent<'a> {
    /// Hit-test results
    HitTest {
        position: Point,
        hovered: &'a RefCell<HashSet<WidgetId>>,
        hot: &'a RefCell<Option<WidgetId>>,
    },
}*/

/// Events.
#[derive(Clone, Debug)]
pub enum EventKind {
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
    Keyboard(KeyboardEvent),
    //Internal(InternalEvent<'a>),
}

pub struct Event<'a> {
    pub route: &'a [WidgetId],
    pub handled: bool,
    pub kind: EventKind,
}

impl<'a> Event<'a> {
    pub fn new(route: &'a [WidgetId], kind: EventKind) -> Event<'a> {
        Event {
            route,
            handled: false,
            kind,
        }
    }

    pub fn next_target(&mut self) -> Option<WidgetId> {
        let (next, rest) = self.route.split_first()?;
        self.route = rest;
        Some(*next)
    }

    /*pub fn set_handled(&mut self) {
        if self.handled {
            warn!("Event::set_handled: event already handled");
        }
        self.handled = true;
    }*/

    //pub fn propagate(&mut self) {}

    /*pub fn kind(&self) -> &EventKind {
        &self.kind
    }*/

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
