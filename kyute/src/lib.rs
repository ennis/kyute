#![feature(coerce_unsized)]
#![feature(unsize)]
#![feature(arc_new_cyclic)]
#![feature(const_str_from_utf8)]
extern crate self as kyute;

#[macro_use]
mod data;
//pub mod application;
mod bloom;
//mod composition;
//mod core;
pub mod event;
//mod key;
mod layout;
mod util;
//pub mod widget;
//mod window;
pub mod region;
#[macro_use]
mod env;
pub mod application;
pub mod cache;
mod call_key;
mod core;
mod state;
pub mod style;
pub mod text;
pub mod theme;
pub mod widget;
mod window;
//mod style;

pub use kyute_macros::{composable, Data};
pub use kyute_shell::AssetUri;

pub use crate::{
    cache::{changed, environment, memoize, once, run_async, state, with_environment, Cache, Key},
    core::{EventCtx, GpuFrameCtx, LayoutCtx, PaintCtx, Widget, WidgetExt, WidgetId, WidgetPod, SHOW_DEBUG_OVERLAY},
    data::Data,
    env::{EnvKey, EnvValue, Environment},
    event::{Event, InternalEvent},
    layout::{align_boxes, Alignment, BoxConstraints, Measurements},
    state::{Signal, State},
    widget::Orientation,
    window::Window,
};

pub use kyute_shell as shell;
// re-export graal
pub use kyute_shell::graal;

#[cfg(feature = "imbl")]
pub use imbl;

pub type Dip = kyute_shell::drawing::Dip;
pub type Px = kyute_shell::drawing::Px;

pub type DipToPx = euclid::Scale<f64, Dip, Px>;
pub type PxToDip = euclid::Scale<f64, Px, Dip>;
pub type SideOffsets = euclid::SideOffsets2D<f64, Dip>;
pub type Size = kyute_shell::drawing::Size;
pub type PhysicalSize = kyute_shell::drawing::PhysicalSize;
pub type Rect = kyute_shell::drawing::Rect;
pub type Offset = kyute_shell::drawing::Offset;
pub type Point = kyute_shell::drawing::Point;
pub type PhysicalPoint = kyute_shell::drawing::PhysicalPoint;
pub type Color = kyute_shell::drawing::Color;
