use crate::{
    style::Style,
    theme,
    widget::{prelude::*, Container, Null},
};
use kyute_common::UnitExt;

/// Creates a horizontal separator.
#[composable]
pub fn separator(orientation: Orientation) -> impl Widget {
    match orientation {
        Orientation::Vertical => Container::new(Null)
            .fixed_width(2.px())
            .centered()
            .box_style(Style::new().background(theme::palette::GREY_700)),
        Orientation::Horizontal => Container::new(Null)
            .fixed_height(2.px())
            .centered()
            .box_style(Style::new().background(theme::palette::GREY_700)),
    }
}
