//! built-in widgets.
mod button;
//mod container;
mod flex;
mod text;
mod graphics;
mod menu;
mod baseline;
mod slider;
mod textedit;
mod drop_down;
mod align;
mod layout_wrapper;
mod constrained;
mod padding;
//mod splitter;
mod grid;
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

pub use layout_wrapper::LayoutWrapper;
pub use flex::{CrossAxisAlignment, Flex, MainAxisAlignment, MainAxisSize};
pub use text::Text;
pub use button::Button;
pub use menu::{MenuItem, Menu, Action, Shortcut};
pub use baseline::Baseline;
pub use slider::Slider;
pub use textedit::TextEdit;
pub use drop_down::DropDown;
pub use align::Align;
pub use constrained::ConstrainedBox;
pub use padding::Padding;
pub use grid::{GridLength,Grid};

/*pub use button::{button, ButtonResult};
pub use container::container;
pub use slider::slider;
pub use text::text;
pub use textedit::text_line_edit;
pub use window::window;

use crate::{style::StyleSet, CompositionCtx};
use std::sync::Arc;
*/