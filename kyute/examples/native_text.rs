use kyute::{
    application, composable,
    shell::{application::Application, winit::window::WindowBuilder},
    style::BoxStyle,
    theme,
    widget::{Container, Flex, Grid, GridLength, Image, Label, Null, TitledPane},
    Alignment, AssetId, BoxConstraints, Color, EnvKey, Environment, Orientation, Point, Size, UnitExt, Widget,
    WidgetExt, WidgetPod, Window,
};

use kyute_text::FormattedText;

use std::sync::Arc;

#[composable]
fn native_text_test() -> impl Widget + Clone {
    let text = FormattedText::from("hello world 😒")
        .with_attribute(0..5, kyute_text::Attribute::Color(Color::new(0.7, 0.7, 0.7, 1.0)))
        .with_attribute(.., kyute_text::Attribute::FontSize(17.0))
        .with_attribute(.., kyute_text::FontFamily::new("Cambria"))
        .with_attribute(.., kyute_text::FontStyle::Italic);

    let paragraph = text.create_paragraph(Size::new(500.0, 500.0));
    let glyph_runs = paragraph.get_rasterized_glyph_runs(1.0, Point::origin());
    eprintln!("{:?}", glyph_runs);

    let pane_1 = TitledPane::collapsible("Initially collapsed", true, Label::new("Hi!".to_string()));
    let pane_2 = TitledPane::collapsible("Initially expanded", false, Label::new("Hey!".to_string()));

    let mut v = Grid::column(400.dip());
    v.add_row(pane_1);
    v.add_row(pane_2);

    Container::new(v)
}

#[composable]
fn ui_root() -> Arc<WidgetPod> {
    Arc::new(WidgetPod::new(Window::new(
        WindowBuilder::new().with_title("Native text"),
        native_text_test(),
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
