//! Checkboxes.
use crate::{
    widget::{form, prelude::*, Clickable, Label, StyledBox, Text},
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

    text.font_size(14.dip()).style(
        r#"
width: 14px;
height: 14px;
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

pub struct CheckboxField<Label> {
    label: Label,
    checkbox: Checkbox,
}

impl<Label> CheckboxField<Label> {
    #[composable]
    pub fn new(label: Label, checked: bool) -> CheckboxField<Label> {
        let checkbox = Checkbox::new(checked);
        CheckboxField { label, checkbox }
    }

    pub fn on_toggled(self, f: impl FnOnce(bool)) -> Self {
        if let Some(state) = self.toggled() {
            f(state);
        }
        self
    }

    pub fn toggled(&self) -> Option<bool> {
        self.checkbox.toggled()
    }
}

impl<Label> From<CheckboxField<Label>> for form::Row
where
    Label: Widget + 'static,
{
    fn from(field: CheckboxField<Label>) -> Self {
        form::Row::Field {
            label: field.label.vertical_alignment(Alignment::FirstBaseline).arc_pod(),
            content: field.checkbox.vertical_alignment(Alignment::FirstBaseline).arc_pod(),
            swap_content_and_label: true,
        }
    }
}
