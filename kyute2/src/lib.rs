//!
//! # Features
//! * `vulkan` : enables vulkan support
//!     * skia: uses the vulkan backend
//!     * win32: enables vulkan interop for composition swap chains
#![feature(const_fn_floating_point_arithmetic)]

extern crate core;
extern crate self as kyute2;

// public modules
pub mod application;
pub mod composition;
pub mod debug_util;
pub mod drawing;
pub mod event;
pub mod layout;
pub mod platform;
pub mod text;
pub mod theme;
pub mod utils;
pub mod widget;

// internal modules
mod app_globals;
mod asset;
mod atoms;
mod backend;
mod color;
mod context;
mod counter;
#[cfg(feature = "debug_window")]
mod debug_window;
mod element;
mod environment;
mod length;
mod reconcile;
mod skia_backend;
mod style;
mod vec_diff;
mod window;

// public exports
pub use app_globals::AppGlobals;
pub use application::{AppCtx, AppLauncher};
pub use asset::{Asset, AssetId};
pub use atoms::Atom;
pub use color::Color;
pub use context::{ElementId, ElementTree, EventCtx, HitTestResult, LayoutCtx, PaintCtx, RouteEventCtx, TreeCtx};
pub use element::Element;
pub use environment::{EnvKey, EnvValue, Environment};
pub use event::{Event, EventKind};
pub use layout::{Alignment, Geometry, LayoutParams};
pub use length::{LengthOrPercentage, UnitExt};
pub use widget::{AnyWidget, ChangeFlags, Stateful, StatefulElement, Widget};
pub use window::{AppWindowBuilder, AppWindowHandle};

// macro reexports
pub use kyute2_macros::grid_template;
pub use kyute_compose::{composable, Signal, State, Widget};

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
