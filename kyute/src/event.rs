//! [`Events`](Event) sent to widgets, and related types.
use crate::Point;
use std::collections::HashMap;
use winit::event::DeviceId;
use winit::event::ModifiersState;

/// Represents the type of pointer.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum PointerType {
    Mouse,
    Pen,
    Stylus,
    Other,
}

/// Represents a pointer button.
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
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct PointerButtons(pub u32);

impl PointerButtons {
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
    pub fn is_empty(&self) -> bool {
        self.0 == 0
    }
}

impl Default for PointerButtons {
    fn default() -> Self {
        PointerButtons(0)
    }
}

/// Modeled after [W3C's PointerEvent](https://www.w3.org/TR/pointerevents3/#pointerevent-interface)
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct PointerEvent {
    /// Position in device-independent (logical) pixels, relative to the visual node that the event
    /// is delivered to.
    pub position: Point,
    /// Window position.
    pub window_position: Point,
    /// State of the keyboard modifiers when this event was emitted.
    pub modifiers: winit::event::ModifiersState,
    /// The state of the mouse buttons when this event was emitted.
    pub buttons: PointerButtons,
    /// Identifies the pointer.
    pub pointer_id: winit::event::DeviceId,
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

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct PointerButtonEvent {
    pub pointer: PointerEvent,
    /// The button that triggered this event, if there is one.
    pub button: Option<PointerButton>,
    /// The repeat count for double, triple (and more) for button press events (`Event::PointerDown`).
    /// Otherwise, the value is unspecified.
    pub repeat_count: u32,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct KeyboardEvent {
    pub scan_code: u32,
    pub key: Option<winit::event::VirtualKeyCode>,
    pub repeat: bool,
    pub modifiers: winit::event::ModifiersState,
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

/// Events.
///
/// Events are sent to [Visuals](super::visual::Visual).
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Event {
    PointerDown(PointerButtonEvent),
    PointerUp(PointerButtonEvent),
    PointerMove(PointerEvent),
    PointerOver(PointerEvent),
    PointerOut(PointerEvent),
    Wheel(WheelEvent),
    KeyDown(KeyboardEvent),
    KeyUp(KeyboardEvent),
    Input(InputEvent),
    FocusIn,
    FocusOut,
}

impl Event {
    pub fn pointer_event(&self) -> Option<&PointerEvent> {
        match self {
            Event::PointerMove(p) => Some(p),
            Event::PointerUp(p) => Some(&p.pointer),
            Event::PointerDown(p) => Some(&p.pointer),
            _ => None,
        }
    }

    pub fn keyboard_event(&self) -> Option<&KeyboardEvent> {
        match self {
            Event::KeyDown(p) => Some(p),
            Event::KeyUp(p) => Some(p),
            _ => None,
        }
    }

    pub fn input_event(&self) -> Option<&InputEvent> {
        match self {
            Event::Input(p) => Some(p),
            _ => None,
        }
    }
}

/// Last known state of a pointer.
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
pub struct InputState {
    /// Current state of keyboard modifiers.
    pub mods: ModifiersState,
    /// Current state of pointers.
    pub pointers: HashMap<DeviceId, PointerState>,
}

impl InputState {
    pub fn synthetic_pointer_event(&self, device_id: DeviceId) -> Option<PointerEvent> {
        self.pointers.get(&device_id).map(|state| PointerEvent {
            position: state.position,
            window_position: state.position,
            modifiers: self.mods,
            buttons: state.buttons,
            pointer_id: device_id,
        })
    }
}

impl Default for InputState {
    fn default() -> Self {
        InputState {
            mods: winit::event::ModifiersState::default(),
            pointers: HashMap::new(),
        }
    }
}
