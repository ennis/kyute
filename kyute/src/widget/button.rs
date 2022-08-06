use crate::{
    event::PointerEventKind,
    layout::Alignment,
    style,
    style::WidgetState,
    widget::{prelude::*, Clickable, Label, WidgetExt},
    Color, Signal, UnitExt,
};
use std::cell::Cell;

////////////////////////////////////////////////////////////////////////////////////////////////////
// Button style
////////////////////////////////////////////////////////////////////////////////////////////////////

type ButtonInner = impl Widget;

/// The built-in button style, compatible with light & dark modes.
const BUTTON_STYLE: &str = r#"
border-radius: 8px;
padding: 3px;
min-width: 80px;
min-height: 30px;

[$dark-mode] {
    background: rgb(88 88 88);
    border: solid 1px rgb(49 49 49);
    box-shadow: inset 0px 1px rgb(115 115 115), 0px 1px 2px -1px rgb(49 49 49);
    [:hover] background: rgb(100 100 100);
    [:focus] border: solid 1px #3895f2;
    [:active] background: rgb(60 60 60);
    [:active] box-shadow: none; 
}

[!$dark-mode] {
    background: rgb(255 255 255);
    border: solid 1px rgb(180 180 180);
    box-shadow: 0px 1px 3px -1px rgb(180 180 180);
    [:hover] background: rgb(240 240 240);
    [:active] background: rgb(240 240 240);
    [:active] box-shadow: none;
    [:focus] border: solid 1px #3895f2;
}
"#;

#[composable]
fn button_inner(label: String) -> ButtonInner {
    Label::new(label)
        .horizontal_alignment(Alignment::CENTER)
        .vertical_alignment(Alignment::CENTER)
        .style(BUTTON_STYLE)
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Widget definition
////////////////////////////////////////////////////////////////////////////////////////////////////

/// Button widget.
///
/// A button widget with the default visual style. To add button-like behavior to your visual, you can use the
/// `Clickable` wrapper.
#[derive(Widget)]
pub struct Button {
    inner: Clickable<ButtonInner>,
}

impl Button {
    /// Creates a new button with the specified label.
    #[composable]
    pub fn new(label: impl Into<String>) -> Button {
        let inner = button_inner(label.into()).clickable();
        Button { inner }
    }

    /// Returns whether this button has been clicked.
    pub fn clicked(&self) -> bool {
        self.inner.clicked()
    }

    /// Runs the function when the button has been clicked.
    pub fn on_click(self, f: impl FnOnce()) -> Self {
        Button {
            inner: self.inner.on_click(f),
        }
    }
}
