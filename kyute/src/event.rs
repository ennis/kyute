use druid_shell::KeyModifiers;
pub use druid_shell::{KeyEvent, MouseEvent, WinHandler};
use piet::kurbo;

/// Widget event
pub enum Event {
    KeyUpEvent(KeyEvent),
    KeyDownEvent(KeyEvent),
    MouseUpEvent(MouseEvent),
    MouseDownEvent(MouseEvent),
    MouseMoveEvent(MouseEvent),
    Zoom {
        delta: f64,
    },
    WheelEvent {
        delta: kurbo::Vec2,
        mods: KeyModifiers,
    },
}
