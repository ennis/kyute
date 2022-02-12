use crate::model::{Item, ItemColor, ItemKind, StyleSheet};
use kyute::{
    composable,
    shell::drawing::Color,
    widget::{Grid, GridLength, Text, TextEdit},
    State, Widget,
};

// we still pass &mut StyleSheet
// This is because widgets don't really have a "local" influence on the state due to links
// for example, when modifying the value of an item with dependents we should modify the value
// of all connected items, which are siblings.
// => "Lensing"

// fn create_item_color_ui(item: &mut ItemColor) -> Widget
// -> problem: not enough, in the UI we want to display a popup containing all Items to which we can connect
// -> thus, must pass a copy of the items vector, and can't mutate in place
//

#[composable(uncached)]
pub fn create_item_color_ui(
    stylesheet: &mut StyleSheet,
    item: &Item,
    item_color: &ItemColor,
) -> impl Widget + 'static {
    let text_input = TextEdit::new(item_color.color.to_hex());
    if let Some(new_text) = text_input.text_changed() {
        match Color::try_from_hex(&new_text.plain_text) {
            Ok(color) => {
                stylesheet.set_color(item, color);
            }
            Err(_) => {
                eprintln!("invalid color spec");
            }
        }
    }
    text_input
}

#[composable(uncached)]
pub fn create_item_ui(stylesheet: &mut StyleSheet, grid: &mut Grid, row: usize, item: &Item) {
    let mut item_widget = match item.kind() {
        ItemKind::Color(item_color) => create_item_color_ui(stylesheet, item, item_color),
    };

    grid.push(row, 0, Text::new(item.name().to_string()));
    grid.push(row, 1, item_widget);
}

#[composable(uncached)]
pub fn items_ui(stylesheet: &mut StyleSheet) -> impl Widget {
    // items grid
    let mut grid = Grid::new()
        .with_column(GridLength::Fixed(200.0))
        .with_column(GridLength::Flex(1.0));

    // create rows for each item
    for i in 0..stylesheet.items().len() {
        grid.add_row(GridLength::Auto);
    }

    let items = stylesheet.items().clone();
    for (row, item) in items.values().enumerate() {
        create_item_ui(stylesheet, &mut grid, row, item)
    }

    grid
}

#[composable(uncached)]
pub fn main_ui() {
    let stylesheet_state = State::new(|| StyleSheet::new());

    let mut stylesheet = stylesheet.get();
}
