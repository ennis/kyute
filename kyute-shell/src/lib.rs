//! Windowing and drawing base for kyute.
//!
//! For now, it's win32 only.

#![feature(const_fn_floating_point_arithmetic)]
#![feature(const_mut_refs)]
pub mod application;
pub mod asset;
mod backend;
pub mod drawing;
pub mod error;
mod shortcut;
pub mod window;

// TODO: backend-agnostic wrapper
pub use asset::{Asset, AssetId, AssetLoader, RawAssetId, AssetLoadError};
pub use backend::Menu;
pub use shortcut::{Shortcut, ShortcutKey};

// Re-export winit for WindowBuilder and stuff
pub use winit;
// Re-export skia
pub use skia_safe as skia;
// Re-export graal
pub use graal;
