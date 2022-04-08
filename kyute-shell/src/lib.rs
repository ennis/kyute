//! Windowing and drawing base for kyute.
//!
//! For now, it's win32 only.

#[macro_use]
extern crate tracing;

pub mod animation;
pub mod application;
mod backend;
pub mod error;
mod shortcut;
pub mod window;

pub use backend::Menu;
pub use shortcut::{Shortcut, ShortcutKey};

// Re-export winit for WindowBuilder and stuff
pub use winit;
// Re-export graal
pub use graal;
