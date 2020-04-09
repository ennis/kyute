//! [`Events`](Event) sent to widgets, and related types.
use crate::layout::{Bounds, Layout};
use crate::Point;
use winit::event::WindowEvent;

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
    pub const LEFT: PointerButton = PointerButton(0);       // Or touch/pen contact
    pub const MIDDLE: PointerButton = PointerButton(1);
    pub const RIGHT: PointerButton = PointerButton(2);      // Or pen barrel
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
        self.0 |= (1u32 << button.0 as u32);
    }

    pub fn reset(&mut self, button: PointerButton) {
        self.0 &= !(1u32 << button.0 as u32);
    }

    pub fn is_empty(&self) -> bool { self.0 == 0 }
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
    /// The button that triggered this event, if there is one.
    pub button: Option<PointerButton>,
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

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct KeyboardEvent {
    pub scan_code: u32,
    pub key: Option<winit::event::VirtualKeyCode>,
    pub repeat: bool,
    pub modifiers: winit::event::ModifiersState,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct WheelEvent {
    pub delta_x: f64,
    pub delta_y: f64,
    pub delta_z: f64,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct InputEvent {
    character: char,
}

/// Events.
///
/// Events are sent to [Visuals](super::visual::Visual).
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Event {
    PointerDown(PointerEvent),
    PointerUp(PointerEvent),
    PointerMove(PointerEvent),
    Wheel(WheelEvent),
    KeyDown(KeyboardEvent),
    KeyUp(KeyboardEvent),
    Input(InputEvent),
}
