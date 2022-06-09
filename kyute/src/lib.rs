//#![feature(const_mut_refs)]

extern crate self as kyute;

#[macro_use]
extern crate tracing;

#[macro_use]
mod env;

pub mod animation;
pub mod application;
pub mod asset;
mod bloom;
pub mod cache;
mod call_id;
mod core;
mod drawing;
pub mod event;
mod font;
mod layout;
mod live_literal;
pub mod region;
mod state;
pub mod style;
pub mod theme;
pub mod util;
pub mod widget;
mod window;

pub use kyute_macros::composable;

pub use crate::{
    animation::PaintCtx,
    asset::{Asset, AssetId, AssetLoader, AssetUri},
    bloom::Bloom,
    cache::{changed, environment, memoize, once, run_async, state, with_environment, Signal, State},
    core::{
        EventCtx, GpuFrameCtx, LayerPaintCtx, LayoutCache, LayoutCtx, Widget, WidgetExt, WidgetFilter, WidgetId,
        SHOW_DEBUG_OVERLAY,
    },
    env::{EnvKey, EnvRef, EnvValue, Environment},
    event::{Event, InputEvent, InternalEvent, PointerEvent, PointerEventKind},
    font::Font,
    layout::{align_boxes, Alignment, BoxConstraints, Measurements},
    live_literal::live_literal,
    widget::Orientation,
    window::Window,
};

pub use kyute_shell as shell;
pub use kyute_shell::{graal, text};

// re-export basic types from kyute-common
pub use kyute_common::{
    Angle, Color, Data, Dip, Length, Offset, PhysicalPoint, PhysicalSize, Point, PointI, Px, Rect, RectExt, RectI,
    RoundToPixel, SideOffsets, Size, SizeI, Transform, UnitExt, DIP, PX,
};
