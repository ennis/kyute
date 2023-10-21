//!
//! # Features
//! * `vulkan` : enables vulkan support
//!     * skia: uses the vulkan backend
//!     * win32: enables vulkan interop for composition swap chains
#![feature(const_fn_floating_point_arithmetic)]

extern crate self as kyute2;

// public modules
pub mod composition;
pub mod event;
pub mod layout;
pub mod platform;
pub mod text;
pub mod theme;
pub mod utils;
pub mod widget;

// internal modules
mod app_state;
mod application;
mod asset;
mod atoms;
mod backend;
mod color;
mod context;
mod counter;
mod drawing;
mod elem_node;
mod environment;
mod length;
mod reconcile;
mod skia;
mod style;
mod vec_diff;
mod widget_id;
mod widget_tree;
mod window;

// public exports
pub use app_state::{AppHandle, AppLauncher};
pub use application::AppGlobals;
pub use asset::{Asset, AssetId};
pub use atoms::Atom;
pub use color::Color;
pub use context::{EventCtx, HitTestResult, LayoutCtx, PaintCtx, RouteEventCtx, TreeCtx};
pub use elem_node::Element;
pub use environment::{EnvKey, EnvValue, Environment};
pub use event::{Event, EventKind};
pub use layout::{Alignment, Geometry, LayoutParams};
pub use length::{LengthOrPercentage, UnitExt};
pub use reconcile::reconcile_elements;
pub use widget_id::WidgetId;
pub use widget_tree::{AnyWidget, ChangeFlags, Widget};
pub use window::AppWindowBuilder;

// macro reexports
pub use kyute2_macros::grid_template;
pub use kyute_compose::{composable, Widget};

// kurbo reexports
pub use kurbo::{self, Affine, Insets, Point, Rect, Size, Vec2};

// kyute-common reexports
pub use kyute_common::Data;

// graal reexport
#[cfg(feature = "vulkan")]
pub use graal;

// reexport the whole of palette
pub use palette;
