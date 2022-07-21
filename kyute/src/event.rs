//! [`Events`](Event) sent to widgets, and related types.
use crate::{bloom::Bloom, Point, WidgetId};
use std::collections::HashMap;
use winit::event::DeviceId;
// FIXME: reexport/import from kyute-shell?
use crate::core::DebugWidgetTreeNode;
pub use keyboard_types::{CompositionEvent, Key, KeyboardEvent, Modifiers};
use kyute_common::Transform;
use kyute_shell::winit;

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
}

/// The state of the mouse buttons.
// TODO why u no bitflags?
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
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

impl Default for PointerButtons {
    fn default() -> Self {
        PointerButtons::new()
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum PointerEventKind {
    PointerDown,
    PointerUp,
    PointerMove,
    PointerOver,
    PointerOut,
}

/// Modeled after [W3C's PointerEvent](https://www.w3.org/TR/pointerevents3/#pointerevent-interface)
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct PointerEvent {
    pub kind: PointerEventKind,
    /// Position in device-independent (logical) pixels, relative to the visual node that the event
    /// is delivered to.
    pub position: Point,
    /// Window position.
    pub window_position: Point,
    /// State of the keyboard modifiers when this event was emitted.
    pub modifiers: Modifiers,
    /// The state of the mouse buttons when this event was emitted.
    pub buttons: PointerButtons,
    /// Identifies the pointer.
    pub pointer_id: winit::event::DeviceId,
    /// The button that triggered this event, if there is one.
    pub button: Option<PointerButton>,
    /// The repeat count for double, triple (and more) for button press events (`Event::PointerDown`).
    /// Otherwise, the value is unspecified.
    pub repeat_count: u32,
    //pub contact_width: f64,
    //pub contact_height: f64,
    //pub pressure: f32,
    //pub tangential_pressure: f32,
    //pub tilt_x: i32,
    //pub tilt_y: i32,
    //pub twist: i32,
    //pub pointer_type: PointerType,
    //pub primary: bool,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum WheelDeltaMode {
    Pixel,
    Line,
    Page,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct WheelEvent {
    pub pointer: PointerEvent,
    pub delta_x: f64,
    pub delta_y: f64,
    pub delta_z: f64,
    pub delta_mode: WheelDeltaMode,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct InputEvent {
    pub character: char,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum MoveFocusDirection {
    Before,
    After,
}

#[derive(Clone, Debug)]
pub enum LifecycleEvent {}

#[derive(Debug)]
pub enum InternalEvent<'a> {
    /// Update composition layers.
    UpdateLayers {
        skia_direct_context: &'a mut skia_safe::gpu::DirectContext,
    },
    RouteEvent {
        target: WidgetId,
        event: Box<Event<'a>>,
    },
    /// Contains a pointer event delivered directly to a target widget (because e.g. it captured
    /// the pointer).
    RoutePointerEvent {
        target: WidgetId,
        event: PointerEvent,
    },
    RouteWindowEvent {
        target: WidgetId,
        event: winit::event::WindowEvent<'static>,
    },
    RouteRedrawRequest(WidgetId),
    //RouteInitialize,
    UpdateChildFilter {
        filter: &'a mut Bloom<WidgetId>,
    },
    DumpTree {
        nodes: &'a mut Vec<DebugWidgetTreeNode>,
    },
}

/// Events.
#[derive(Debug)]
pub enum Event<'a> {
    /// Event sent after recomposition.
    Initialize,
    FocusGained,
    FocusLost,
    MenuCommand(usize),
    Pointer(PointerEvent),
    Wheel(WheelEvent),
    /// A keyboard event.
    Keyboard(KeyboardEvent),
    /// A composition event.
    Composition(CompositionEvent),
    WindowEvent(winit::event::WindowEvent<'static>),
    WindowRedrawRequest,
    BuildFocusChain {
        chain: &'a mut Vec<WidgetId>,
    },
    Internal(InternalEvent<'a>),
}

impl<'a> Event<'a> {
    /// If this event contains a relative pointer location, subtracts the specified offset to it and
    /// runs the provided closure with the modified event.
    /// Otherwise, runs the provided closure with this event, unmodified.
    pub fn with_local_coordinates<R>(&mut self, transform: &Transform, f: impl FnOnce(&mut Event) -> R) -> R {
        match *self {
            Event::Internal(InternalEvent::RoutePointerEvent { ref event, target }) => {
                let mut event_copy = *event;
                event_copy.position = transform.inverse().unwrap().transform_point(event_copy.position);
                f(&mut Event::Internal(InternalEvent::RoutePointerEvent {
                    event: event_copy,
                    target,
                }))
            }
            Event::Pointer(ref pointer_event) => {
                let mut event_copy = *pointer_event;
                event_copy.position = transform.inverse().unwrap().transform_point(event_copy.position);
                f(&mut Event::Pointer(event_copy))
            }
            _ => f(self),
        }
    }

    pub fn pointer_event(&self) -> Option<&PointerEvent> {
        match self {
            Event::Pointer(p) => Some(p),
            _ => None,
        }
    }

    pub fn keyboard_event(&self) -> Option<&KeyboardEvent> {
        match self {
            Event::Keyboard(p) => Some(p),
            _ => None,
        }
    }

    pub fn composition_event(&self) -> Option<&CompositionEvent> {
        match self {
            Event::Composition(p) => Some(p),
            _ => None,
        }
    }
}

/// Last known state of a pointer.
#[derive(Copy, Clone, Debug)]
pub struct PointerState {
    pub(crate) buttons: PointerButtons,
    pub(crate) position: Point,
}

impl Default for PointerState {
    fn default() -> Self {
        PointerState {
            buttons: PointerButtons(0),
            position: Point::origin(),
        }
    }
}

/// Last known state of various input devices.
#[derive(Clone, Default)]
pub struct InputState {
    /// Current state of keyboard modifiers.
    pub modifiers: Modifiers,
    /// Current state of pointers.
    pub pointers: HashMap<DeviceId, PointerState>,
}

impl InputState {
    pub fn synthetic_pointer_event(
        &self,
        device_id: DeviceId,
        kind: PointerEventKind,
        button: Option<PointerButton>,
    ) -> Option<PointerEvent> {
        self.pointers.get(&device_id).map(|state| PointerEvent {
            kind,
            position: state.position,
            window_position: state.position,
            modifiers: self.modifiers,
            buttons: state.buttons,
            pointer_id: device_id,
            button,
            repeat_count: 0,
        })
    }
}
