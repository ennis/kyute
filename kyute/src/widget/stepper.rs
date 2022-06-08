use crate::{
    event::PointerEventKind,
    style::{BoxStyle, VisualState},
    theme, tweak,
    widget::{grid::GridLayoutExt, prelude::*, Button, Container, Grid, Label, WidgetWrapper},
    Color, Signal, UnitExt, WidgetExt,
};
use std::{
    cell::Cell,
    cmp,
    ops::{Add, Neg},
};

/// Two small up & down arrows to select a numeric value
#[derive(WidgetWrapper)]
pub struct Stepper<T> {
    grid: Grid,
    new_value: Option<T>,
}

impl<T> Stepper<T>
where
    T: Copy + Neg<Output = T> + Add<Output = T> + PartialOrd,
{
    #[composable]
    pub fn new(value: T, min: T, max: T, step: T) -> Stepper<T> {
        // TODO icon buttons
        let up = Button::new("+".to_string());
        let down = Button::new("-".to_string());

        let mut new_value = None;

        if up.clicked() {
            if value + step <= max {
                new_value = Some(value + step);
            }
        }
        if down.clicked() {
            if value + (-step) >= min {
                new_value = Some(value + (-step));
            }
        }

        let mut grid = Grid::with_template(tweak!("12 12 / 12 / 1 0"));
        grid.insert((up.grid_row(0), down.grid_row(1)));
        Stepper { grid, new_value }
    }

    pub fn on_value_changed(self, f: impl FnOnce(T)) -> Self {
        self.new_value.map(f);
        self
    }

    pub fn value_changed(&self) -> Option<T> {
        self.new_value
    }
}
