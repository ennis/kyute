use crate::{
    event::PointerEventKind,
    layout::Alignment,
    style,
    style::VisualState,
    widget::{prelude::*, Clickable, Label, WidgetExt},
    Color, Signal, UnitExt,
};
use std::cell::Cell;

////////////////////////////////////////////////////////////////////////////////////////////////////
// Widget definition
////////////////////////////////////////////////////////////////////////////////////////////////////

type ButtonInner = impl Widget;

#[derive(Widget)]
pub struct Button {
    inner: Clickable<ButtonInner>,
}

#[composable]
fn button_inner(label: String, active: bool, hover: bool, focus: bool) -> ButtonInner {
    let mut style = "background: rgb(88 88 88);\
             border-radius: 8px;\
             padding: 5px;\
             min-width: 80px;\
             min-height: 30px;\
             border: solid 1px rgb(49 49 49);\
             box-shadow: inset 0px 1px rgb(115 115 115), 0px 1px 2px -1px rgb(49 49 49);"
        .to_string();

    if hover {
        style.push_str("background: rgb(100 100 100);");
    }
    if active {
        style.push_str("background: rgb(60 60 60); box-shadow: none;");
    }
    if focus {
        // TODO outline
        style.push_str("border: solid 1px #3895f2;");
    }

    Label::new(label)
        .text_color(Color::from_rgb_u8(200, 200, 200))
        .horizontal_alignment(Alignment::CENTER)
        .style(style.as_str())
}

impl Button {
    /// Creates a new button with the specified label.
    #[composable]
    pub fn new(label: impl Into<String>) -> Button {
        #[state]
        let mut hover = false;
        #[state]
        let mut active = false;
        #[state]
        let mut focus = false;

        let inner = button_inner(label.into(), active, hover, focus)
            .clickable()
            .on_activated(|| active = true)
            .on_deactivated(|| active = false)
            .on_pointer_entered(|| hover = true)
            .on_pointer_exited(|| hover = false)
            .on_focus_changed(|f| focus = f);

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
