//! Kyute widget toolkit
#![feature(unsized_locals)]
#![feature(coerce_unsized)]
#![feature(unsize)]

pub mod application;
pub mod env;

pub mod event;

pub mod layout;
pub mod renderer;
pub mod state;
pub mod visual;
pub mod widget;

// re-exports
pub use self::visual::NodeData;
pub use self::visual::Visual;

pub use self::widget::BoxedWidget;
pub use self::widget::Widget;
pub use self::widget::WidgetExt;

pub use self::layout::Alignment;
pub use self::layout::Bounds;
pub use self::layout::BoxConstraints;
pub use self::layout::Layout;
pub use self::layout::Point;
pub use self::layout::Size;
