mod align;
mod flex;
mod frame;
pub mod grid;
mod label;
mod null;
mod relative;

use crate::Widget;

////////////////////////////////////////////////////////////////////////////////////////////////////
pub use flex::{VBox, VBoxElement};
pub use frame::{Frame, FrameElement};
pub use grid::{Grid, GridTemplate};
pub use label::{Text, TextElement};
pub use null::{Null, NullElement};

////////////////////////////////////////////////////////////////////////////////////////////////////

pub trait HasLayoutProperties<T> {
    type Widget: Widget;

    fn into_widget(self) -> (Self::Widget, T);
}

pub struct Attached<W, T> {
    pub widget: W,
    pub props: T,
}

impl<W, T> HasLayoutProperties<T> for Attached<W, T>
where
    W: Widget,
{
    type Widget = W;

    fn into_widget(self) -> (Self::Widget, T) {
        (self.widget, self.props)
    }
}

impl<W, T> HasLayoutProperties<T> for W
where
    W: Widget,
    T: Default,
{
    type Widget = W;

    fn into_widget(self) -> (Self::Widget, T) {
        (self, T::default())
    }
}

/// Axis.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Axis {
    Horizontal,
    Vertical,
}
