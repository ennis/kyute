//! Checkboxes.
use crate::{
    widget::{prelude::*, Clickable, StyledBox, Text},
    Font,
};
use kyute_common::Color;

type CheckboxInner = impl Widget;

fn checkbox_inner(state: bool) -> CheckboxInner {
    // TODO crude, replace with a cached WidgetPod
    let text = if state {
        Text::new("✓").color(Color::from_hex("#161616"))
    } else {
        Text::new("")
    };

    text.font_size(30.dip()).style(
        r#"
width: 40px;
height: 40px;
background: rgb(255 255 255);
border-radius: 5px;
border: solid 1px rgb(180 180 180);
box-shadow: 0px 1px 3px -1px rgb(180 180 180);
            "#,
    )
}

#[derive(Widget)]
pub struct Checkbox {
    inner: Clickable<CheckboxInner>,
    state: bool,
}

impl Checkbox {
    #[composable]
    pub fn new(state: bool) -> Checkbox {
        Checkbox {
            inner: checkbox_inner(state).clickable(),
            state,
        }
    }

    pub fn on_toggled(self, f: impl FnOnce(bool)) -> Self {
        if let Some(state) = self.toggled() {
            f(state);
        }
        self
    }

    pub fn toggled(&self) -> Option<bool> {
        if self.inner.clicked() {
            Some(!self.state)
        } else {
            None
        }
    }
}
