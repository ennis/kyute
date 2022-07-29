use kyute::{
    composable,
    widget::{grid::TrackBreadth, Grid, Stepper, Text, WidgetPod},
    UnitExt, Widget,
};
use std::sync::Arc;

#[composable]
pub fn showcase() -> Arc<WidgetPod> {
    #[state]
    let mut current_value = 0;
    let mut hbox = Grid::row(TrackBreadth::Fixed(20.dip()));
    let stepper = Stepper::new(current_value, -20, 20, 1).on_value_changed(|value| current_value = value);
    hbox.insert((Text::new("Stepper:"), stepper));
    Arc::new(WidgetPod::new(hbox))
}
