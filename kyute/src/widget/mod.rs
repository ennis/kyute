//! built-in widgets.
mod button;
//mod container;
mod align;
mod baseline;
mod constrained;
mod drop_down;
mod flex;
mod layout_wrapper;
mod menu;
mod padding;
mod slider;
mod text;
mod textedit;
//mod splitter;
pub mod grid;
//mod collapsible;
mod container;
mod collapsible;
mod clickable;
//mod grid;
//mod slider;
//mod text;
//mod textedit;
//mod window;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Orientation {
    Horizontal,
    Vertical,
}

impl Orientation {
    pub fn cross_orientation(self) -> Orientation {
        match self {
            Orientation::Horizontal => Orientation::Vertical,
            Orientation::Vertical => Orientation::Horizontal,
        }
    }
}

pub use align::Align;
pub use baseline::Baseline;
pub use button::Button;
pub use constrained::ConstrainedBox;
pub use container::Container;
pub use drop_down::DropDown;
pub use flex::{CrossAxisAlignment, Flex, MainAxisAlignment, MainAxisSize};
pub use grid::{Grid, GridLength};
pub use layout_wrapper::LayoutWrapper;
pub use menu::{Action, Menu, MenuItem, Shortcut};
pub use padding::Padding;
pub use slider::Slider;
pub use text::Text;
pub use textedit::TextEdit;
pub use clickable::Clickable;

/*pub use button::{button, ButtonResult};
pub use container::container;
pub use slider::slider;
pub use text::text;
pub use textedit::text_line_edit;
pub use window::window;

use crate::{style::StyleSet, CompositionCtx};
use std::sync::Arc;
*/
