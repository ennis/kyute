//! `Event`s sent to widgets, and related types.
//use enumflags2::BitFlags;
use winit::event::WindowEvent;
use crate::layout::{Bounds, Layout};
use crate::Point;

/// Represents the type of pointer.
#[derive(Copy,Clone,Debug,Eq,PartialEq,Hash)]
pub enum PointerType {
    Mouse,
    Pen,
    Stylus,
    Other,
}

/// The state of keyboard modifiers.
#[derive(Copy,Clone,Debug,Eq,PartialEq,Hash)]
pub struct ModifierState {
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
    pub meta: bool,
}

/// Represents a mouse button.
#[derive(Copy,Clone,Debug,Eq,PartialEq,Hash)]
pub struct MouseButton(pub u16);

impl MouseButton {
    pub const LEFT: MouseButton = MouseButton(0);
    pub const MIDDLE: MouseButton = MouseButton(1);
    pub const RIGHT: MouseButton = MouseButton(2);
    pub const X1: MouseButton = MouseButton(3);
    pub const X2: MouseButton = MouseButton(4);
}

/// The state of the mouse buttons.
#[derive(Copy,Clone,Debug,Eq,PartialEq,Hash)]
pub struct MouseButtons(pub u32);

impl MouseButtons {
    /// Checks if the specified mouse button is pressed.
    pub fn pressed(self, button: MouseButton) -> bool {
        self.0 & (1u32 << button.0 as u32) != 0
    }
}


/// Modeled after [W3C's PointerEvent](https://www.w3.org/TR/pointerevents3/#pointerevent-interface)
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct PointerEvent {
    /// Position in device-independent (logical) pixels.
    pub position: Point,
    /// State of the keyboard modifiers when this event was emitted.
    pub modifiers: ModifierState,
    /// The button that triggered this event, if there is one.
    pub button: Option<MouseButton>,
    /// The state of the mouse buttons when this event was emitted.
    pub buttons: MouseButtons,
    /// Identifies the pointer.
    pub pointer_id: u32,

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


#[derive(Copy,Clone,Debug,Eq,PartialEq,Hash)]
pub struct KeyboardEvent {
    pub scan_code: u32,
    pub key: winit::event::VirtualKeyCode,
    pub text: char,
    pub repeat: bool,
    pub modifiers: ModifierState,
}

#[derive(Copy,Clone,Debug,PartialEq)]
pub struct WheelEvent {
    pub delta_x: f64,
    pub delta_y: f64,
    pub delta_z: f64
}

#[derive(Copy,Clone,Debug,Eq,PartialEq,Hash)]
pub struct InputEvent {
    c: char,
}

/// Events.
///
/// Events are sent to [Visuals](super::visual::Visual).
#[derive(Copy,Clone,Debug,PartialEq)]
pub enum Event {
    PointerDown(PointerEvent),
    PointerUp(PointerEvent),
    PointerMove(PointerEvent),
    Wheel(WheelEvent),
    KeyDown(KeyboardEvent),
    KeyUp(KeyboardEvent),
    Input(InputEvent)
}

/// Context for event propagation.
pub struct EventCtx {
    /// The bounds of the current visual.
    pub bounds: Bounds,
}

impl EventCtx {
}