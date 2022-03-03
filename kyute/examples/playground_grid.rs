use anyhow::Error;
use kyute::{
    application, cache, composable,
    shell::{application::Application, winit::window::WindowBuilder},
    style::BoxStyle,
    text::{Attribute, FormattedText},
    theme,
    widget::{
        drop_down,
        grid::{AlignItems, JustifyItems},
        Container, DropDown, Flex, Formatter, Grid, GridLength, Image, Label, Null, Slider, Text, TextEdit, TextInput,
        TitledPane, ValidationResult,
    },
    Alignment, AssetId, BoxConstraints, Color, Data, EnvKey, Environment, Orientation, Point, Size, State, UnitExt,
    Widget, WidgetExt, WidgetPod, Window,
};
use kyute_text::{Selection, TextAlignment};
use std::sync::Arc;
use tracing::{info, trace};

#[composable]
fn fixed_size_widget(w: f64, h: f64, name: impl Into<String> + Data) -> impl Widget {
    // TODO "debug widget" that draws a background pattern, with a border
    Label::new(name).fix_size(Size::new(w, h))
}

#[composable(cached)]
fn playground_grid() -> impl Widget + Clone {
    let mut grid = Grid::with_columns([GridLength::Fixed(200.0), GridLength::Fixed(5.0), GridLength::Flex(1.0)])
        .align_items(AlignItems::Baseline);

    // row count
    let mut row = 0;

    let row_count = State::new(|| 2usize);
    let column_count = State::new(|| 2usize);
    let align_items = State::new(|| AlignItems::Start);
    let justify_items = State::new(|| JustifyItems::Start);

    {
        let input = TextInput::number(row_count.get() as f64);
        if let Some(v) = input.value_changed() {
            row_count.set(v as usize);
        }
        grid.add(row, 0, Label::new("Row count"));
        grid.add(row, 2, input);
        row += 1;
    }

    {
        let input = TextInput::number(column_count.get() as f64);
        if let Some(v) = input.value_changed() {
            column_count.set(v as usize);
        }
        grid.add(row, 0, Label::new("Column count"));
        grid.add(row, 2, input);
        row += 1;
    }

    {
        let drop_down = DropDown::with_selected(
            vec![
                AlignItems::Start,
                AlignItems::End,
                AlignItems::Center,
                AlignItems::Stretch,
                AlignItems::Baseline,
            ],
            align_items.get(),
            drop_down::DebugFormatter,
        );
        if let Some(align) = drop_down.selected_item_changed() {
            align_items.set(align);
        }
        grid.add(row, 0, Label::new("Item alignment"));
        grid.add(row, 2, drop_down);
        row += 1;
    }

    {
        let drop_down = DropDown::with_selected(
            vec![
                JustifyItems::Start,
                JustifyItems::End,
                JustifyItems::Center,
                JustifyItems::Stretch,
            ],
            justify_items.get(),
            drop_down::DebugFormatter,
        );
        if let Some(justify) = drop_down.selected_item_changed() {
            justify_items.set(justify);
        }
        grid.add(row, 0, Label::new("Item justify"));
        grid.add(row, 2, drop_down);
        row += 1;
    }

    eprintln!(
        "rows,columns = ({},{})",
        row_count.get() as usize,
        column_count.get() as usize
    );

    let row_defs = vec![GridLength::Flex(1.0); row_count.get() as usize];
    let column_defs = vec![GridLength::Flex(1.0); column_count.get() as usize];

    let mut play_grid = Grid::with_rows_columns(row_defs, column_defs)
        .align_items(align_items.get())
        .justify_items(justify_items.get());

    for i in 0..row_count.get() {
        for j in 0..column_count.get() {
            play_grid.add(i, j, fixed_size_widget(50.0, 50.0, "hello"))
        }
    }

    grid.add(row, 2, Container::new(play_grid).fixed_height(700.dip()));

    Container::new(grid).box_style(BoxStyle::new().fill(theme::palette::BLUE_GREY_800))
}

#[composable]
fn ui_root() -> Arc<WidgetPod> {
    Arc::new(WidgetPod::new(Window::new(
        WindowBuilder::new().with_title("Grid playground"),
        playground_grid(),
        None,
    )))
}

fn main() {
    let _app = Application::new();

    let mut env = Environment::new();
    theme::setup_default_style(&mut env);
    env.set(kyute::widget::grid::SHOW_GRID_LAYOUT_LINES, true);

    tracing_subscriber::fmt()
        .compact()
        .with_target(false)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    application::run(ui_root, env);
    Application::shutdown();
}
