mod colors;
mod renderer;
mod text;

use crate::layout::Offset;
use crate::widget::textedit::Selection;
use kyute_shell::drawing::Point;

/// Represents a 2D line segment
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct LineSegment {
    pub start: Point,
    pub end: Point,
}

/// Represents the state of a button.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct ButtonState {
    pub disabled: bool,
    pub clicked: bool,
    pub hot: bool,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum TextState {
    Default,
    Disabled,
}

/// Tri-state checkbox state.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum CheckBoxState {
    Unchecked,
    PartiallyChecked,
    Checked,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct CheckBoxOptions {
    enabled: bool,
    state: CheckBoxState,
}
