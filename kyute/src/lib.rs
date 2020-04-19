//! Kyute widget toolkit
#![feature(unsized_locals)]

pub mod application;
pub mod env;

pub mod event;

pub mod layout;
pub mod renderer;
pub mod state;
pub mod visual;
pub mod widget;

// re-exports
pub use self::visual::Node;
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

// Node: a node in the visual tree of the user interface
//      - contains useful retained state, children, layout information, and the visual
// Visual: widget-specific drawing and event behavior
// Widget: produces visuals
// Layout: Size+Baseline
// EventCtx: context passed on event propagation
// LayoutCtx: context passed on layout
// PaintCtx: context passed during painting
//
// - visual: Event, Paint, Visual, Node
//

// concerns:
// - layout
// - input state
// - event handling
//      -
// - painting
//      - renderer
//      - themes
// - window management + application event loop
// - node reconciliation
//
// Node and Visual is tied to almost all of them
