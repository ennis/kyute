use kyute::{
    application, cache, composable,
    shell::{application::Application, winit::window::WindowBuilder},
    style::BoxStyle,
    text::{Attribute, FormattedText},
    theme,
    widget::{Container, Flex, Grid, GridLength, Image, Label, Null, Slider, Text, TextEdit, TitledPane},
    Alignment, AssetId, BoxConstraints, Color, EnvKey, Environment, Orientation, Point, Size, State, UnitExt, Widget,
    WidgetExt, WidgetPod, Window,
};
use kyute_text::{Selection, TextAlignment};
use std::sync::Arc;

#[composable]
fn text_edit(font_size: f64, grid: &mut Grid) {
    let label = Label::new(format!("Font size: {}dip", font_size));
    let text = State::new(|| Arc::from(format!("{}dip text", font_size)));
    let formatted_text = FormattedText::new(text.get())
        .font_size(font_size)
        .text_alignment(TextAlignment::Center);

    let text_edit = TextEdit::new(formatted_text);

    if let Some(new_text) = text_edit.text_changed() {
        text.set(new_text);
    }

    let row = grid.row_count();
    grid.add(row, 0, label);
    grid.add(row, 2, text_edit);
}

#[composable(cached)]
fn text_playground() -> impl Widget + Clone {
    let base_font_size = 14.0;
    let mut grid = Grid::with_columns([GridLength::Fixed(200.0), GridLength::Fixed(5.0), GridLength::Flex(1.0)]);
    for i in 0..6 {
        cache::scoped(i, || {
            text_edit(base_font_size + (i as f64) * 4.0, &mut grid);
        });
    }

    let row = grid.row_count();
    grid.add(row, 0, Label::new("Custom font size".to_string()));
    let custom_font_size = State::new(|| 14.0);
    let custom_font_size_slider = Slider::new(3.0, 80.0, custom_font_size.get());
    if let Some(value) = custom_font_size_slider.value_changed() {
        custom_font_size.set(value);
    }
    grid.add(row, 2, custom_font_size_slider);
    text_edit(custom_font_size.get(), &mut grid);

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
