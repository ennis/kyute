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
// Widget definition
////////////////////////////////////////////////////////////////////////////////////////////////////

type ButtonInner = impl Widget;

/// Button widget.
///
/// A button widget with the default visual style. To add button-like behavior to your visual, you can use the
/// `Clickable` wrapper.
#[derive(Widget)]
pub struct Button {
    inner: Clickable<ButtonInner>,
}

#[composable]
fn button_inner(label: String) -> ButtonInner {
    let mut style = "background: rgb(88 88 88);\
             border-radius: 8px;\
             padding: 5px;\
             min-width: 80px;\
             min-height: 30px;\
             border: solid 1px rgb(49 49 49);\
             box-shadow: inset 0px 1px rgb(115 115 115), 0px 1px 2px -1px rgb(49 49 49);\
             [if :hover] background: rgb(100 100 100);\
             [if :focus] border: solid 1px #3895f2;\
             [if :active] background: rgb(60 60 60);\
             [if :active] box-shadow: none;"
        .to_string();

    Label::new(label)
        .text_color(Color::from_rgb_u8(200, 200, 200))
        .horizontal_alignment(Alignment::CENTER)
        .vertical_alignment(Alignment::CENTER)
        .style(style.as_str())
}

// widget state flags:
// - hover: set by buttons.
// - active: set by clickable stuff, etc.
// - focus: set by focusable widgets
// - disabled: set by disabled modifier
//
// They propagate to child widgets via the environment?
// - except focus?
// - focus propagates until encountering another focusable widget in the chain

// should clickables be stateless?
// - only emit events, the parent handles the widget state (hovered, active, etc.)

//
// StyleBox:
// - if the style has a dependency on hover, track hover state
// -
//

//

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

        let mut state_flags = WidgetState::default();
        if hover {
            state_flags |= WidgetState::HOVER;
        }
        if active {
            state_flags |= WidgetState::ACTIVE;
        }
        if focus {
            state_flags |= WidgetState::FOCUS;
        }

        let inner = button_inner(label.into())
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
