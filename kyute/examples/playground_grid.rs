use kyute::{
    application, cache, composable,
    shell::{application::Application, winit::window::WindowBuilder},
    style::BoxStyle,
    text::{Attribute, FormattedText},
    theme,
    widget::{
        drop_down,
        grid::{AlignItems, GridTrackDefinition, JustifyItems},
        Container, DropDown, Flex, Formatter, Grid, GridLength, Image, Label, Null, Slider, Text, TextEdit, TextInput,
        Thumb, TitledPane, ValidationResult,
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
fn playground_grid(test: usize) -> impl Widget + Clone {
    #[state]
    let mut row_count = 2usize;
    #[state]
    let mut column_count = 2usize;
    #[state]
    let mut align_items = AlignItems::Start;
    #[state]
    let mut justify_items = JustifyItems::Start;

    let mut grid = Grid::with_column_definitions([
        GridTrackDefinition::new(GridLength::Fixed(200.0)),
        GridTrackDefinition::new(GridLength::Fixed(5.0)),
        GridTrackDefinition::new(GridLength::Flex(1.0)),
    ])
    .align_items(AlignItems::Baseline);

    // row count
    let mut row = 0;

    {
        grid.add_item(row, 0, Label::new("Row count"));
        grid.add_item(
            row,
            2,
            TextInput::number(row_count as f64).on_value_changed(|v| row_count = v as usize),
        );

        row += 1;
    }

    {
        grid.add_item(row, 0, Label::new("Column count"));
        grid.add_item(
            row,
            2,
            TextInput::number(column_count as f64).on_value_changed(|v| column_count = v as usize),
        );
        row += 1;
    }

    {
        grid.add_item(row, 0, Label::new("Item alignment"));
        grid.add_item(
            row,
            2,
            DropDown::with_selected(
                align_items,
                vec![
                    AlignItems::Start,
                    AlignItems::End,
                    AlignItems::Center,
                    AlignItems::Stretch,
                    AlignItems::Baseline,
                ],
                drop_down::DebugFormatter,
            )
            .on_selected_item_changed(|align| align_items = align),
        );
        row += 1;
    }

    {
        grid.add_item(row, 0, Label::new("Item justify"));
        grid.add_item(
            row,
            2,
            DropDown::with_selected(
                justify_items,
                vec![
                    JustifyItems::Start,
                    JustifyItems::End,
                    JustifyItems::Center,
                    JustifyItems::Stretch,
                ],
                drop_down::DebugFormatter,
            )
            .on_selected_item_changed(|justify| justify_items = justify),
        );
        row += 1;
    }

    eprintln!("rows,columns = ({},{})", row_count, column_count);

    let row_defs = vec![GridLength::Flex(1.0).into(); row_count];
    let column_defs = vec![GridLength::Flex(1.0).into(); column_count];

    let mut play_grid = Grid::with_rows_columns(row_defs, column_defs)
        .align_items(align_items)
        .justify_items(justify_items);

    for i in 0..row_count {
        cache::scoped(i, || {
            for j in 0..column_count {
                cache::scoped(j, || play_grid.add_item(i, j, Thumb::draggable(Label::new("hello"))));
            }
        });
    }

    grid.add_item(row, 2, Container::new(play_grid).fixed_height(700.dip()));

    Container::new(grid).box_style(BoxStyle::new().fill(theme::palette::BLUE_GREY_800))
}

#[composable]
fn ui_root() -> Arc<WidgetPod> {
    Arc::new(WidgetPod::new(Window::new(
        WindowBuilder::new().with_title("Grid playground"),
        playground_grid(0),
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
