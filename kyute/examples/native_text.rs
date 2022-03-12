use kyute::{
    application, composable,
    shell::{application::Application, winit::window::WindowBuilder},
    style::BoxStyle,
    text::FormattedText,
    theme,
    widget::{grid::GridTrackDefinition, Container, Flex, Grid, GridLength, Image, Label, Null, Text, TitledPane},
    Alignment, AssetId, BoxConstraints, Color, EnvKey, Environment, Orientation, Point, Size, UnitExt, Widget,
    WidgetExt, WidgetPod, Window,
};
use std::sync::Arc;

#[composable(cached)]
fn native_text_test() -> impl Widget + Clone {
    let text = FormattedText::from("⬤⬤⬤⬤⬤⬤⬤⬤⬤⬤⬤⬤")
        .attribute(0..6, kyute_text::Attribute::Color(Color::from_hex("#DDDDDD")))
        .attribute(.., kyute_text::Attribute::FontSize(15.0));

    let text_widget = Text::new(text);

    /*let paragraph = text.create_paragraph(Size::new(500.0, 500.0));
    let glyph_runs = paragraph.get_rasterized_glyph_runs(1.0, Point::origin());
    eprintln!("{:?}", glyph_runs);*/

    let pane_1 = TitledPane::collapsible("Initially collapsed", true, Label::new("Hi!".to_string()));
    let pane_2 = TitledPane::collapsible("Initially expanded", false, text_widget);

    let mut v = Grid::column(GridTrackDefinition::new(400.dip()));
    v.add_row(pane_1);
    v.add_row(pane_2);

    Container::new(v).box_style(theme::TITLED_PANE_HEADER)
}

#[composable]
fn ui_root() -> impl Widget {
    Window::new(WindowBuilder::new().with_title("Native text"), native_text_test(), None)
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
