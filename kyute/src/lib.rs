//#![feature(const_mut_refs)]
#![feature(type_alias_impl_trait)]

extern crate self as kyute;

#[macro_use]
extern crate tracing;

#[macro_use]
mod env;

#[macro_use]
mod atoms;

pub mod application;
pub mod asset;
mod bloom;
pub mod cache;
mod call_id;
mod core;
mod css;
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

pub use crate::{
    asset::{Asset, AssetId, AssetLoader, AssetUri},
    atoms::Atom,
    bloom::Bloom,
    cache::{changed, environment, memoize, once, run_async, state, with_environment, Signal, State},
    core::{
        DebugNode, EventCtx, LayerPaintCtx, LayoutCache, LayoutCtx, Widget, WidgetFilter, WidgetId, SHOW_DEBUG_OVERLAY,
    },
    drawing::PaintCtx,
    env::{EnvKey, EnvRef, EnvValue, Environment},
    event::{Event, InputEvent, InternalEvent, PointerEvent, PointerEventKind},
    font::Font,
    layout::{Alignment, BoxConstraints, Layout, LayoutConstraints, Measurements},
    live_literal::live_literal,
    style::{Length, LengthOrPercentage, UnitExt},
    widget::Orientation,
    window::Window,
};

pub use kyute_macros::{composable, Widget};
pub use kyute_shell as shell;
pub use kyute_shell::{graal, text};

// re-export basic types from kyute-common
pub use kyute_common::{
    Angle, Color, Data, Dip, Offset, PhysicalPoint, PhysicalSize, Point, PointI, Px, Rect, RectExt, RectI,
    RoundToPixel, SideOffsets, Size, SizeI, Transform, DIP, PX,
};
