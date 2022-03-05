use anyhow::Error;
use kyute::{
    application, cache, composable,
    shell::{application::Application, winit::window::WindowBuilder},
    style::BoxStyle,
    text::{Attribute, FormattedText},
    theme,
    widget::{
        grid::{AlignItems, GridTrackDefinition},
        Canvas, Container, Flex, Formatter, Grid, GridLength, Image, Label, Null, Slider, Text, TextEdit, TextInput,
        TitledPane, ValidationResult,
    },
    Alignment, AssetId, BoxConstraints, Color, EnvKey, Environment, Offset, Orientation, Point, Size, State, UnitExt,
    Widget, WidgetExt, WidgetPod, Window,
};
use kyute_common::Transform;
use kyute_text::{Selection, TextAlignment};
use std::sync::Arc;
use tracing::info;

#[composable(cached)]
fn canvas_playground() -> impl Widget + Clone {
    #[state]
    let mut offset = Offset::zero();
    #[state]
    let mut scale = 1.0;

    let mut grid = Grid::with_column_definitions([
        GridTrackDefinition::new(GridLength::Fixed(200.0)),
        GridTrackDefinition::new(GridLength::Fixed(5.0)),
        GridTrackDefinition::new(GridLength::Flex(1.0)),
    ])
    .align_items(AlignItems::Baseline);

    grid.add_item(0, 0, Label::new("Offset X"));
    grid.add_item(0, 2, TextInput::number(offset.x).on_value_changed(|x| offset.x = x));

    grid.add_item(1, 0, Label::new("Offset Y"));
    grid.add_item(1, 2, TextInput::number(offset.y).on_value_changed(|y| offset.y = y));

    grid.add_item(2, 0, Label::new("Scale"));
    grid.add_item(2, 2, TextInput::number(scale).on_value_changed(|s| scale = s));

    let mut canvas = Canvas::new();
    canvas.set_transform(offset.to_transform().then_scale(scale, scale));
    canvas.add_item(Offset::new(0.0, 0.0), Label::new("Artist: 少女理論観測所"));

    grid.add_item(3, .., canvas.fix_height(800.0));

    Container::new(grid).box_style(BoxStyle::new().fill(theme::palette::BLUE_GREY_800))
}

#[composable]
fn ui_root() -> Arc<WidgetPod> {
    Arc::new(WidgetPod::new(Window::new(
        WindowBuilder::new().with_title("Canvas playground"),
        canvas_playground(),
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
