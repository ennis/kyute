use crate::input::keyboard::Modifiers;
use bitflags::bitflags;
use kyute_common::Point;

/// Represents the type of pointer.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum PointerType {
    Mouse,
    Pen,
    Stylus,
    Other,
}

bitflags! {
    #[derive(Default)]
    pub struct PointerButtons: u32 {
        const LEFT = 0x01;
        const MIDDLE = 0x2;
        const RIGHT = 0x4;
        const X1 = 0x8;
        const X2 = 0x10;
        const ERASER = 0x20;
    }
}

/// Pointer ID.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct PointerId(u64);

/// Modeled after [W3C's PointerEvent](https://www.w3.org/TR/pointerevents3/#pointerevent-interface)
#[derive(Copy, Clone, PartialEq)]
pub struct PointerInputEvent {
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
    pub pointer_id: PointerId,
    /// The button that triggered this event, if there is one.
    pub button: Option<PointerButtons>,
    /// The repeat count for double, triple (and more) for button press events (`Event::PointerDown`).
    /// Otherwise, the value is unspecified.
    pub repeat_count: u32,
    pub contact_width: f64,
    pub contact_height: f64,
    pub pressure: f32,
    pub tangential_pressure: f32,
    pub tilt_x: i32,
    pub tilt_y: i32,
    pub twist: i32,
    pub pointer_type: PointerType,
    pub primary: bool,
}
