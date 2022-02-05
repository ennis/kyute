#![feature(const_fn_floating_point_arithmetic)]
#![feature(const_mut_refs)]

//! Windowing and drawing base for kyute.
//!
//! For now, it's win32 only.
pub mod drawing;
//pub mod text;
//pub mod imaging;
pub mod application;
mod backend;
pub mod error;
mod shortcut;
pub mod window;

// TODO: backend-agnostic wrapper
pub use backend::Menu;
pub use shortcut::{Shortcut, ShortcutKey};

// Re-export winit for WindowBuilder and stuff
pub use winit;
// Re-export skia
pub use skia_safe as skia;
// Re-export graal
pub use graal;
