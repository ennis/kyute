use kyute::{
    application, composable,
    shell::{application::Application, winit::window::WindowBuilder},
    style::BoxStyle,
    theme,
    widget::{
        grid::GridTrack, ColorPicker, ColorPickerParams, Container, Flex, Grid, GridLength, Image, Label,
        Null, Text, TitledPane,
    },
    Alignment, AssetId, BoxConstraints, Color, EnvKey, Environment, Orientation, Size, UnitExt, Widget, WidgetExt,
    Window,
};
use kyute_shell::winit::dpi::LogicalSize;
use std::sync::Arc;
use tracing::trace_span;
use kyute::style::Paint;

#[composable]
fn color_picker() -> impl Widget {
    #[state]
    let mut color = Color::from_hex("#022f78");
    let picker = ColorPicker::new(
        color,
        &ColorPickerParams {
            enable_alpha: true,
            palette: None,
            enable_hex_input: true,
        },
    )
    .on_color_changed(|c| color = c);
    Container::new(picker.centered()).background(Paint::from(theme::palette::BLUE_GREY_900))
}

#[composable]
fn ui_root() -> impl Widget {
    Window::new(
        WindowBuilder::new()
            .with_inner_size(LogicalSize::new(500, 180))
            .with_title("Color pickers"),
        color_picker(),
        None,
    )
}

fn main() {
    /*use tracing_subscriber::layer::SubscriberExt;
    tracing::subscriber::set_global_default(tracing_subscriber::registry().with(tracing_tracy::TracyLayer::new()))
        .expect("set up the subscriber");
*/
    tracing_subscriber::fmt()
    .compact()
    .with_target(false)
    .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
    .init();

    application::run(ui_root);
}
