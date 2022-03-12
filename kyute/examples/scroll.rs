use kyute::{
    application, composable,
    shell::{application::Application, winit::window::WindowBuilder},
    style::BoxStyle,
    text::FormattedText,
    theme,
    widget::{
        grid::GridTrackDefinition, Canvas, Container, Flex, Grid, GridLength, Image, Label, Null, ScrollArea, Text,
        TitledPane,
    },
    Alignment, AssetId, BoxConstraints, Color, EnvKey, Environment, Orientation, Point, Size, UnitExt, Widget,
    WidgetExt, WidgetPod, Window,
};
use std::sync::Arc;

#[composable(cached)]
fn scroll_test() -> impl Widget + Clone {
    let mut canvas = Canvas::new();

    for i in 0..50 {
        let y_pos = (i as f64) / 50.0 * 10000.0;
        canvas.add_item(
            300.0,
            y_pos,
            Container::new(Text::new(format!("y-pos={}", y_pos)))
                .box_style(theme::DROP_DOWN)
                .fix_width(100.0),
        );
    }

    ScrollArea::new(canvas.fix_size(Size::new(900.0, 10000.0)))
}

#[composable]
fn ui_root() -> impl Widget {
    Window::new(WindowBuilder::new().with_title("Scrolling"), scroll_test(), None)
}

fn main() {
    tracing_subscriber::fmt()
        .compact()
        .with_target(false)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let _app = Application::new();
    application::run(ui_root);
    Application::shutdown();
}
