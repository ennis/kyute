#![feature(coerce_unsized)]
#![feature(unsize)]
#![feature(arc_new_cyclic)]
#![feature(const_str_from_utf8)]
#![feature(const_fn_floating_point_arithmetic)]
#![feature(const_mut_refs)]

extern crate self as kyute;

#[macro_use]
mod env;
mod bloom;
mod call_id;
mod core;
mod drawing;
mod layout;
mod state;
mod window;

pub mod application;
pub mod asset;
pub mod cache;
pub mod event;
pub mod region;
pub mod style;
pub mod text;
pub mod theme;
pub mod util;
pub mod widget;

pub use kyute_macros::{composable, Data};

pub use crate::{
    asset::{Asset, AssetId, AssetLoader, AssetUri},
    cache::{changed, environment, memoize, once, run_async, state, with_environment, Cache, Key},
    core::{EventCtx, GpuFrameCtx, LayoutCtx, PaintCtx, Widget, WidgetExt, WidgetId, WidgetPod, SHOW_DEBUG_OVERLAY},
    env::{EnvKey, EnvValue, Environment, ValueRef},
    event::{Event, InputEvent, InternalEvent, PointerEvent, PointerEventKind},
    layout::{align_boxes, Alignment, BoxConstraints, Measurements},
    state::{Signal, State},
    widget::Orientation,
    window::Window,
};

pub use kyute_shell as shell;
// re-export graal
pub use kyute_shell::graal;

// re-export basic types from kyute-common
pub use kyute_common::{
    Angle, Color, Data, Dip, Length, Offset, PhysicalPoint, PhysicalSize, Point, Px, Rect, RectExt, RoundToPixel,
    SideOffsets, Size, SizeI, Transform, UnitExt, DIP, PX,
};
