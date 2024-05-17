//!
//! # Features
//! * `vulkan` : enables vulkan support
//!     * skia: uses the vulkan backend
//!     * win32: enables vulkan interop for composition swap chains

#![feature(const_fn_floating_point_arithmetic)]
#![feature(impl_trait_in_assoc_type)] // should be stable soon

// public modules
pub mod application;
pub mod composition;
pub mod drawing;
pub mod event;
pub mod layout;
pub mod platform;
pub mod text;
pub mod theme;
pub mod utils;
pub mod widgets;
pub mod window;

// internal modules
mod app_globals;
//mod asset;
mod backend;
mod color;
//#[cfg(feature = "debug_window")]
//mod debug_window;
mod core;
mod environment;
mod length;
mod skia_backend;
mod style;
mod widget_ext;

// public exports
pub use app_globals::AppGlobals;
pub use application::AppLauncher;
pub use environment::Environment;
//pub use asset::{Asset, AssetId};
pub use color::Color;
//pub use element::{Element, TransformNode};
//pub use environment::{EnvKey, EnvValue, Environment};
pub use core::{
    Binding, Builder, ChangeFlags, HitTestResult, IntoWidget, LayoutCtx, PaintCtx, State, Widget, WidgetCtx, WidgetPod,
    WidgetPtr,
};
pub use event::Event;
pub use layout::{Alignment, BoxConstraints, Geometry};
pub use length::{LengthOrPercentage, UnitExt, IN_TO_DIP, PT_TO_DIP};
pub use widget_ext::WidgetExt;

/// Widget implementor prelude.
pub mod prelude {
    pub use crate::{
        BoxConstraints, ChangeFlags, Environment, Event, Geometry, HitTestResult, IntoWidget, LayoutCtx, PaintCtx,
        Point, Rect, Size, State, Widget, WidgetCtx, WidgetPod, WidgetPtr,
    };
}

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
