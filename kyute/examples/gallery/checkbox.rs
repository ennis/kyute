use kyute::{
    composable,
    widget::{
        grid::{AlignItems, TrackBreadth},
        Button, Checkbox, Grid, Stepper, Text, WidgetPod,
    },
    UnitExt, Widget,
};
use std::sync::Arc;

#[composable]
pub fn showcase() -> Arc<WidgetPod> {
    #[state]
    let mut current_value = false;
    let mut hbox = Grid::row(20.dip());
    //hbox.set_align_items(AlignItems::Baseline);
    let checkbox = Checkbox::new(current_value).on_toggled(|value| current_value = value);
    hbox.insert((Text::new("Checkbox:"), Button::new("gello")));
    Arc::new(WidgetPod::new(hbox))
}
