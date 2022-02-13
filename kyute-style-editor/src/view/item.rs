use crate::model::{Item, ItemColor, ItemKind, StyleSheet};
use kyute::{
    composable,
    shell::drawing::Color,
    widget::{Grid, GridLength, Label, TextEdit},
    State, Widget,
};

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

    grid.push(row, 0, Label::new(item.name().to_string()));
    grid.push(row, 1, item_widget);
}

#[composable(uncached)]
pub fn items_ui(stylesheet: &mut StyleSheet) -> impl Widget {
    // items grid
    let mut grid = Grid::new()
        .with_column(GridLength::Fixed(200.0))
        .with_column(GridLength::Flex(1.0));

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
