use kyute::{
    application, composable,
    shell::{application::Application, winit::window::WindowBuilder},
    style::BoxStyle,
    theme,
    widget::{Button, Container, Flex, Grid, GridLength, Image, Label, Null, Popup, Text, TitledPane},
    Alignment, AssetId, BoxConstraints, Color, EnvKey, Environment, Orientation, Size, UnitExt, Widget, WidgetExt,
    WidgetPod, Window,
};
use std::sync::Arc;
use tracing::trace;

#[composable]
fn popup_test() -> impl Widget + Clone {
    let button = Button::new("Click Me".to_string());

    let contents = Image::from_uri("data/bonjour.jpg");
    let popup = Popup::new(Container::new(contents));

    if button.clicked() {
        trace!("button clicked");
        // launch popup?
        popup.show();
    }

    Grid::new().with(0, 0, button).with(0, 0, popup)
}

#[composable]
fn ui_root() -> Arc<WidgetPod> {
    Arc::new(WidgetPod::new(Window::new(
        WindowBuilder::new().with_title("Popup"),
        popup_test(),
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
