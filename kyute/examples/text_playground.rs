use anyhow::Error;
use kyute::{
    application, cache, composable,
    shell::{application::Application, winit::window::WindowBuilder},
    style::BoxStyle,
    text::{Attribute, FormattedText},
    theme,
    widget::{
        grid::{AlignItems, GridTrackDefinition},
        Container, Flex, Formatter, Grid, GridLength, Image, Label, Null, Slider, Text, TextEdit, TextInput,
        TitledPane, ValidationResult,
    },
    Alignment, AssetId, BoxConstraints, Color, EnvKey, Environment, Orientation, Point, Size, State, UnitExt, Widget,
    WidgetExt, WidgetPod, Window,
};
use kyute_text::{Selection, TextAlignment};
use std::sync::Arc;
use tracing::info;

#[composable]
fn text_edit(font_size: f64, grid: &mut Grid) {
    #[state]
    let mut text = Arc::from(format!("{}dip text", font_size));

    let label = Label::new(format!("Font size: {}dip", font_size));
    let formatted_text = FormattedText::new(text.get())
        .font_size(font_size)
        .text_alignment(TextAlignment::Center);

    let text_edit = TextEdit::new(formatted_text);

    if let Some(new_text) = text_edit.text_changed() {
        text.set(new_text);
    }

    let row = grid.row_count();
    grid.add_item(row, 0, label);
    grid.add_item(row, 2, text_edit);
}

#[composable(cached)]
fn text_playground() -> impl Widget + Clone {
    #[state]
    let mut custom_font_size = 14.0;
    #[state]
    let mut input_value = 0.0;

    let base_font_size = 14.0;

    let mut grid = Grid::with_column_definitions([
        GridTrackDefinition::new(GridLength::Fixed(200.0)),
        GridTrackDefinition::new(GridLength::Fixed(5.0)),
        GridTrackDefinition::new(GridLength::Flex(1.0)),
    ])
    .align_items(AlignItems::Baseline);

    for i in 0..6 {
        cache::scoped(i, || {
            text_edit(base_font_size + (i as f64) * 4.0, &mut grid);
        });
    }

    // TODO:
    // - widgets:     `on_*` methods for easier event handling
    // - grid:        push_row_definition, push_column_definition
    // - grid:        GridRow, GridColumn types
    // - grid:        named rows and columns
    // - composable:  `#[state]` statement attribute.
    // - composable:  `#[identifiable(expr)]` statement attribute.

    {
        let row = grid.row_count();
        grid.add_item(row, 0, Label::new("Custom font size".to_string()));
        let custom_font_size_slider =
            Slider::new(3.0, 80.0, custom_font_size).on_value_changed(|v| custom_font_size = v);
        grid.add_item(row, 2, custom_font_size_slider);
        text_edit(custom_font_size.get(), &mut grid);
    }

    // text input test
    {
        let row = grid.row_count();
        grid.add_item(row, 0, Label::new("Validated text input".to_string()));

        let text_input = TextInput::number(input_value).on_value_changed(|value| {
            info!("input value changed: {:.6}", value);
            input_value = value;
        });
        grid.add_item(row, 2, text_input);
    }

    Container::new(grid).box_style(BoxStyle::new().fill(theme::palette::BLUE_GREY_800))
}

#[composable]
fn ui_root() -> Arc<WidgetPod> {
    Arc::new(WidgetPod::new(Window::new(
        WindowBuilder::new().with_title("Text playground"),
        text_playground(),
        None,
    )))
}

fn main() {
    let _app = Application::new();

    let mut env = Environment::new();
    theme::setup_default_style(&mut env);

    tracing_subscriber::fmt()
        .compact()
        .with_target(false)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    application::run(ui_root, env);
    Application::shutdown();
}
