//! Windowing and drawing base for kyute.
//!
//! For now, it's win32 only.

#[macro_use]
extern crate tracing;

pub mod animation;
pub mod application;
mod backend;
mod clipboard;
pub mod drawing;
mod error;
pub mod input;
mod menu;
mod shortcut;
pub mod text;
pub mod window;

pub use clipboard::TypedData;
pub use error::{Error, Result};
pub use kyute_common::PointI;
pub use menu::Menu;
pub use shortcut::{Shortcut, ShortcutKey};

// Re-export graal
pub use graal;
