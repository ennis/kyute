use kyute::{
    application, composable,
    shell::{application::Application, winit::window::WindowBuilder},
    style::BoxStyle,
    theme,
    widget::{grid::GridTrackDefinition, Container, Flex, Grid, GridLength, Image, Label, Null, TitledPane},
    Alignment, AssetId, BoxConstraints, Color, EnvKey, Environment, Orientation, Size, UnitExt, Widget, WidgetExt,
    WidgetPod, Window,
};
use std::sync::Arc;

#[composable]
fn titled_pane_test() -> impl Widget + Clone {
    let pane_1 = TitledPane::collapsible("Initially collapsed", true, Label::new("Hi!".to_string()));
    let pane_2 = TitledPane::collapsible("Initially expanded", false, Label::new("Hey!".to_string()));

    let mut v = Grid::column(GridTrackDefinition::new(400.dip()));
    v.add_row(pane_1);
    v.add_row(pane_2);

    Container::new(v)
}

#[composable]
fn ui_root() -> impl Widget {
    Window::new(WindowBuilder::new().with_title("Titled Pane"), titled_pane_test(), None)
}

fn main() {
    let _app = Application::new();

    tracing_subscriber::fmt()
        .compact()
        .with_target(false)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    application::run(ui_root);
    Application::shutdown();
}
