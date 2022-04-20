use crate::{
    style::BoxStyle,
    theme,
    widget::{prelude::*, Container, Null},
};
use kyute_common::UnitExt;

/// Creates a horizontal separator.
#[composable]
pub fn separator(orientation: Orientation) -> impl Widget + Clone {
    match orientation {
        Orientation::Vertical => Container::new(Null).fixed_width(2.px()).centered().box_style({
            |env| BoxStyle::new().fill(theme::keys::SEPARATOR_COLOR.get(env).unwrap())
        }
            as fn(&Environment) -> BoxStyle),
        Orientation::Horizontal => Container::new(Null).fixed_height(2.px()).centered().box_style({
            |env| BoxStyle::new().fill(theme::keys::SEPARATOR_COLOR.get(env).unwrap())
        }
            as fn(&Environment) -> BoxStyle),
    }
}
