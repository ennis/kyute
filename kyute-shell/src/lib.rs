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

// TODO: backend-agnostic wrapper
pub use backend::Menu;

// Re-export winit for WindowBuilder and stuff
pub use winit;
// Re-export skia
pub use skia_safe as skia;
// Re-export graal
pub use graal;
