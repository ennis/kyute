//! Checkboxes.
use crate::{
    drawing::ToSkia,
    text::FormattedText,
    theme,
    widget::{form, prelude::*, Clickable, Drawable, Label, Null, StyledBox, Text},
    Color, Font,
};
use skia_safe as sk;

type CheckboxInner = impl Widget;

fn checkbox_inner(checked: bool) -> CheckboxInner {
    Drawable::new(Size::new(18.0, 18.0), None, move |ctx, state, env| {
        // TODO: a better drawing API, or something to author "parametric vector graphics", because writing skia code by hand is miserable
        if checked {
            let path = sk::PathBuilder::new()
                .move_to((2.5, 13.0))
                .line_to((9.0, 18.0))
                .line_to((18.0, 4.0))
                .detach();
            //let dark_mode = env.get(&theme::DARK_MODE).unwrap_or(false);
            let color = Color::from_hex("#FFFFFF");
            let mut paint = sk::Paint::new(color.to_skia(), None);
            paint.set_anti_alias(true);
            paint.set_stroke_miter(1.5);
            paint.set_stroke_cap(sk::PaintCap::Square);
            paint.set_stroke_join(sk::PaintJoin::Miter);
            paint.set_style(sk::PaintStyle::Stroke);
            paint.set_stroke_width(4.0);
            ctx.surface.canvas().clear(Color::from_hex("#00A7FF").to_skia());
            ctx.surface.canvas().save();
            ctx.surface.canvas().scale((0.8, 0.8));
            ctx.surface.canvas().draw_path(&path, &paint);
            ctx.surface.canvas().restore();
        }
    })
    .style(
        r#"
background: $text-background-color;
border-radius: 5px;
[!$dark-mode] border: solid 1px rgb(180 180 180);
[!$dark-mode] box-shadow: 0px 1px 3px -1px rgb(180 180 180);
[$dark-mode] border: solid 1px rgb(49 49 49);
[$dark-mode] box-shadow: 0px 1px 2px -1px rgb(49 49 49);
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

pub struct CheckboxField {
    label: Text,
    checkbox: Checkbox,
}

impl CheckboxField {
    #[composable]
    pub fn new(label: impl Into<FormattedText>, checked: bool) -> CheckboxField {
        let checkbox = Checkbox::new(checked);
        CheckboxField {
            label: Text::new(label),
            checkbox,
        }
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

impl From<CheckboxField> for form::Row {
    fn from(field: CheckboxField) -> Self {
        form::Row::Field {
            label: Null.arc_pod(),
            content: field
                .label
                .right_of(field.checkbox.padding_right(4.dip()), Alignment::CENTER)
                .arc_pod(),
            swap_content_and_label: false,
        }
    }
}
