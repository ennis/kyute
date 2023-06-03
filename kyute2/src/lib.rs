//!
//! # Features
//! * `vulkan` : enables vulkan support
//!     * skia: uses the vulkan backend
//!     * win32: enables vulkan interop for composition swap chains
#![feature(const_fn_floating_point_arithmetic)]

// public modules
pub mod composition;
pub mod event;
pub mod platform;
pub mod widget;

// internal modules
mod app_state;
mod application;
mod atoms;
mod backend;
mod color;
mod context;
mod counter;
mod elem_node;
mod environment;
mod layout;
mod skia;
mod vec_diff;
mod widget_id;
mod widget_tree;
mod window;

// public exports
pub use app_state::{AppHandle, AppLauncher};
pub use application::Application;
pub use atoms::Atom;
pub use color::Color;
pub use context::{EventCtx, LayoutCtx, TreeCtx};
pub use elem_node::Element;
pub use environment::{EnvValue, Environment};
pub use event::Event;
pub use layout::{Alignment, Geometry, LayoutParams};
pub use widget_id::WidgetId;
pub use widget_tree::{AnyWidget, ChangeFlags, Widget, WidgetNode};
pub use window::AppWindowBuilder;

// macro reexports
pub use kyute_compose::{composable, Widget};

// kurbo reexports
pub use kurbo::{Affine, Insets, Rect, Size, Vec2};

// graal reexport
#[cfg(feature = "vulkan")]
pub use graal;

// reexport the whole of palette
pub use palette;
