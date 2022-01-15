//! built-in widgets.
mod button;
//mod container;
mod flex;
mod text;
mod graphics;
mod menu;
mod baseline;
mod slider;
//mod textedit;
//mod grid;
//mod slider;
//mod text;
//mod textedit;
//mod window;

pub use flex::{Axis, CrossAxisAlignment, Flex, MainAxisAlignment, MainAxisSize};
pub use text::Text;
pub use button::Button;
pub use menu::{MenuItem, Menu, Action, Shortcut};
pub use baseline::Baseline;
pub use slider::Slider;

/*pub use button::{button, ButtonResult};
pub use container::container;
pub use slider::slider;
pub use text::text;
pub use textedit::text_line_edit;
pub use window::window;

use crate::{style::StyleSet, CompositionCtx};
use std::sync::Arc;
*/