//!
//! # Features
//! * `vulkan` : enables vulkan support
//!     * skia: uses the vulkan backend
//!     * win32: enables vulkan interop for composition swap chains

#![feature(const_fn_floating_point_arithmetic)]

// public modules
pub mod application;
pub mod composition;
//pub mod debug_util;
pub mod drawing;
pub mod event;
pub mod layout;
pub mod platform;
pub mod text;
pub mod theme;
pub mod utils;
pub mod widget;
pub mod window;

// internal modules
mod app_globals;
//mod asset;
mod backend;
mod color;
mod context;
//mod counter;
//#[cfg(feature = "debug_window")]
//mod debug_window;
//mod element;
//mod environment;
mod counter;
mod length;
mod skia_backend;
mod state;
mod style;
mod vec_diff;
mod widget_id;

// public exports
pub use app_globals::AppGlobals;
pub use application::AppLauncher;
//pub use asset::{Asset, AssetId};
pub use color::Color;
pub use context::{ContextDataHandle, HitTestResult, LayoutCtx, PaintCtx, TreeCtx};
//pub use element::{Element, TransformNode};
//pub use environment::{EnvKey, EnvValue, Environment};
pub use event::{Event, EventKind};
pub use layout::{Alignment, BoxConstraints, Geometry};
pub use length::{LengthOrPercentage, UnitExt, IN_TO_DIP, PT_TO_DIP};
pub use state::{with_ambient, AmbientKey, State};
pub use widget::{ChangeFlags, Widget};
pub use widget_id::WidgetId;
//pub use window::{AppWindowHandle, PopupOptions, PopupTarget};

// macro reexports
pub use kyute2_macros::grid_template;
//pub use kyute_compose::{composable, Signal, Widget};

// kurbo reexports
pub use kurbo::{self, Affine, Insets, Point, Rect, Size, Vec2};

// kyute-common reexports
pub use kyute_common::Data;

// graal reexport
#[cfg(feature = "vulkan")]
pub use graal;

// reexport the whole of palette
pub use palette;

// skia reexport
pub use skia_safe as skia;

// reexport keyboard types
pub use keyboard_types;

#[doc(hidden)]
pub use threadbound::ThreadBound;
