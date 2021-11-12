extern crate self as kyute;

#[macro_use]
mod data;
//pub mod application;
mod bloom;
//mod composition;
//mod core;
mod event;
//mod key;
mod layout;
//mod style;
mod util;
//pub mod widget;
//mod window;
pub mod region;
#[macro_use]
mod env;
pub mod application;
pub mod cache;
pub mod call_key;
mod core2;
pub mod widget;
mod window;
//mod style;

pub use cache::{Cache, CacheError, Key};
pub use core2::{EventCtx, LayoutCtx, PaintCtx, Widget, WidgetPod, WidgetId};
pub use data::Data;
pub use env::{EnvKey, EnvValue, Environment};
pub use event::{Event, InternalEvent};
pub use kyute_macros::{composable, Data};
pub use layout::{align_boxes, Alignment, BoxConstraints, LayoutItem, Measurements};
pub use window::Window;

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
