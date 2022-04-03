//#![feature(const_mut_refs)]

extern crate self as kyute;

#[macro_use]
extern crate tracing;

#[macro_use]
mod env;
mod bloom;
mod call_id;
mod core;
mod drawing;
mod layout;
mod state;
mod window;

pub mod animation;
pub mod application;
pub mod asset;
pub mod cache;
pub mod event;
pub mod region;
pub mod style;
pub mod theme;
pub mod util;
pub mod widget;

pub use kyute_macros::composable;

pub use crate::{
    asset::{Asset, AssetId, AssetLoader, AssetUri},
    bloom::Bloom,
    cache::{changed, environment, memoize, once, run_async, state, with_environment, Signal, State},
    core::{
        EventCtx, GpuFrameCtx, LayoutCtx, PaintCtx, Widget, WidgetExt, WidgetFilter, WidgetId, WidgetPod,
        SHOW_DEBUG_OVERLAY,
    },
    env::{EnvKey, EnvValue, Environment, ValueRef},
    event::{Event, InputEvent, InternalEvent, PointerEvent, PointerEventKind},
    layout::{align_boxes, Alignment, BoxConstraints, Measurements},
    widget::Orientation,
    window::Window,
};

pub use kyute_shell as shell;
pub use kyute_text as text;
// re-export graal
pub use kyute_shell::graal;

// re-export basic types from kyute-common
pub use kyute_common::{
    Angle, Color, Data, Dip, Length, Offset, PhysicalPoint, PhysicalSize, Point, PointI, Px, Rect, RectExt, RectI,
    RoundToPixel, SideOffsets, Size, SizeI, Transform, UnitExt, DIP, PX,
};
