//! Windowing and drawing base for kyute.
//!
//! For now, it's win32 only.
pub mod drawing;
//pub mod text;
//pub mod imaging;
pub mod error;
pub mod application;
pub mod window;
mod backend;
mod shortcut;

// TODO: backend-agnostic wrapper
pub use backend::Menu;
pub use shortcut::Shortcut;


// Re-export winit for WindowBuilder and stuff
pub use winit;
// Re-export skia
pub use skia_safe as skia;
// Re-export graal
pub use graal;
