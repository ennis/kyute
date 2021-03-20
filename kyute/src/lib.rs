//! Kyute widget toolkit
#![feature(unsized_locals)]

pub mod application;
#[macro_use]
pub mod env;
pub mod component;
pub mod event;
pub mod layout;
pub mod node;
pub mod renderer;
pub mod state;
pub mod style;
pub mod theme;
pub mod visual;
pub mod widget;
pub mod window;

pub type SideOffsets = euclid::default::SideOffsets2D<f64>;
pub type Size = kyute_shell::drawing::Size;
pub type Rect = kyute_shell::drawing::Rect;
pub type Offset = kyute_shell::drawing::Offset;
pub type Point = kyute_shell::drawing::Point;

// re-exports
pub use self::widget::{BoxedWidget, TypedWidget, Widget, WidgetExt};

pub use self::component::{CommandSink, Component, State, Update};

pub use self::node::{EventCtx, LayoutCtx, PaintCtx};

pub use self::visual::{DummyVisual, LayoutBox, Visual};

pub use self::layout::{Alignment, BoxConstraints, Measurements};

pub use self::env::Environment;
