use crate::{
    widget::{formatter::Formatter, grid::GridLayoutExt, prelude::*, BaseTextEdit, Grid, ValidationResult},
    Widget,
};
use kyute::widget::Stepper;
use kyute_common::Data;
use std::{
    marker::PhantomData,
    ops::{Add, Neg},
};

/// A text edit widget that validates its input with a `Formatter`.
pub struct TextInput<T> {
    text_edit: BaseTextEdit,
    new_value: Option<T>,
    _phantom: PhantomData<T>,
}

/*impl<T> TextInput<T> {
    /// Creates a new TextInput for a f64.
    #[composable]
    pub fn float_64(value: f64) -> TextInput<f64> {
        Self::new(value, NumberFormatter)
    }

    #[composable]
    pub fn float_32(value: i32) -> TextInput<f64> {
        Self::new(value, NumberFormatter)
    }
}*/

impl<T> TextInput<T>
where
    T: Data,
{
    #[composable]
    pub fn new(value: T, formatter: impl Formatter<T>) -> TextInput<T> {
        // current text during editing
        #[state]
        let mut editing_text = None;

        // if currently editing (editing_text != None), use that, otherwise get the text by formatting the given value
        let text = if let Some(text) = editing_text.clone() {
            text
        } else {
            formatter.format(&value)
        };

        let text_edit = BaseTextEdit::new(text);

        if let Some(text) = text_edit.text_changed() {
            // update editing text
            editing_text = Some(formatter.format_partial_input(&text));
        }

        let mut new_value = None;
        if let Some(text) = text_edit.editing_finished() {
            // Editing finished (Enter pressed or tabbed out). Validate and notify that a value may be available.
            match formatter.validate_partial_input(&text) {
                ValidationResult::Valid => {
                    // editing finished and valid input: clear the editing text, set the new value
                    editing_text = None;
                    new_value = Some(formatter.parse(&text).unwrap());
                }
                ValidationResult::Invalid => {
                    // invalid: keep current (invalid) text, let the formatter highlight the error if it wants
                }
                ValidationResult::Incomplete => {
                    // same as invalid
                }
            }
        }

        TextInput {
            text_edit,
            new_value,
            _phantom: PhantomData,
        }
    }

    /// Returns whether the current value has changed.
    pub fn value_changed(&self) -> Option<T> {
        self.new_value.clone()
    }

    /// Runs the function when the value has changed.
    pub fn on_value_changed(self, f: impl FnOnce(T)) -> Self {
        if let Some(v) = self.new_value.clone() {
            f(v);
        }
        self
    }
}

impl<T> Widget for TextInput<T> {
    fn widget_id(&self) -> Option<WidgetId> {
        self.text_edit.widget_id()
    }

    fn layout(&self, ctx: &mut LayoutCtx, constraints: &LayoutParams, env: &Environment) -> Geometry {
        self.text_edit.layout(ctx, constraints, env)
    }

    fn event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        self.text_edit.route_event(ctx, event, env)
    }

    fn paint(&self, ctx: &mut PaintCtx) {
        self.text_edit.paint(ctx)
    }
}

/// A combo of a numeric TextInput and a stepper.
#[derive(Widget)]
pub struct StepperTextInput<T> {
    grid: Grid,
    new_value: Option<T>,
}

impl<T> StepperTextInput<T>
where
    T: Copy + Neg<Output = T> + Add<Output = T> + PartialOrd + Data,
{
    #[composable]
    pub fn new(value: T, min: T, max: T, step: T, formatter: impl Formatter<T>) -> StepperTextInput<T> {
        let text_input = TextInput::new(value, formatter);
        let stepper = Stepper::new(value, min, max, step);
        let mut new_value = None;

        if let Some(value) = text_input.value_changed() {
            new_value = Some(value);
        }
        if let Some(value) = stepper.value_changed() {
            new_value = Some(value);
        }

        let mut grid = Grid::with_template("26 / 1fr 13 / 1 1");
        grid.insert(text_input.grid_column(0));
        grid.insert(stepper.grid_column(1));
        StepperTextInput { grid, new_value }
    }

    /// Returns whether the current value has changed.
    pub fn value_changed(&self) -> Option<T> {
        self.new_value.clone()
    }

    /// Runs the function when the value has changed.
    pub fn on_value_changed(self, f: impl FnOnce(T)) -> Self {
        if let Some(v) = self.new_value.clone() {
            f(v);
        }
        self
    }
}
