use kyute::{
    composable,
    widget::{
        form,
        form::{Form, Section},
        grid::{AlignItems, TrackBreadth},
        Button, Checkbox, CheckboxField, Grid, Stepper, Text, TextEdit, TextField, WidgetExt, WidgetPod,
    },
    Alignment, UnitExt, Widget,
};
use std::sync::Arc;

#[composable]
pub fn showcase() -> Arc<WidgetPod> {
    let form = Form::new([
        Section::new(
            Text::new("Properties"),
            [
                TextField::new("Type", "MPEG Audio file").into(),
                TextField::new("Tag version", "ID3v2.3").into(),
                TextField::new("Size", "13.3 MB").into(),
                TextField::new("Duration", "5:45").into(),
                TextField::new("Bitrate", "320k").into(),
            ],
        )
        .into(),
        Section::new(
            Text::new("Tags"),
            [
                TextField::new("Track title", "Rendezvous").into(),
                TextField::new("Artist", "Chen-U").into(),
                TextField::new("Album artist", "発熱巫女～ず").into(),
                TextField::new("Album", "Re:Clockwiser & A Narcissus").into(),
                TextField::new("Year", "2011").into(),
                TextField::new("Genre", "Arrange").into(),
            ],
        )
        .into(),
        CheckboxField::new("Favorite", true).into(),
    ]);

    Arc::new(WidgetPod::new(form))
}
