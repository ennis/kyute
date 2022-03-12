use kyute::{
    application, composable,
    shell::{application::Application, winit::window::WindowBuilder},
    theme,
    widget::{Container, Flex, Grid, GridLength, Image, Label, Null},
    Alignment, AssetId, BoxConstraints, Color, EnvKey, Environment, Orientation, Size, UnitExt, Widget, WidgetExt,
    WidgetPod, Window,
};
use std::sync::Arc;

#[composable]
fn fixed_size_widget(w: f64, h: f64, name: &str) -> impl Widget {
    // TODO "debug widget" that draws a background pattern, with a border
    Label::new(name.to_string()).fix_size(Size::new(w, h))
}

#[composable]
fn grid_layout_example() -> impl Widget + Clone {
    let mut grid = Grid::with_rows_columns(
        [GridLength::Fixed(100.0).into(), GridLength::Flex(1.0).into()],
        [
            GridLength::Fixed(100.0).into(),
            GridLength::Auto.into(),
            GridLength::Fixed(100.0).into(),
        ],
    );

    grid.add_item(0, 0, fixed_size_widget(50.0, 50.0, "(0,0)"));
    grid.add_item(0, 1, fixed_size_widget(50.0, 50.0, "(0,1)"));
    //grid.add(0, 2, fixed_size_widget(50.0, 50.0, "(0,2)"));
    grid.add_item(0, 2, Image::from_uri_async("data/haniyasushin_keiki.jpg"));
    grid.add_item(1, 0, fixed_size_widget(50.0, 50.0, "(1,0)"));
    grid.add_item(1, 1..=2, fixed_size_widget(150.0, 50.0, "(1,1)").centered());
    grid
}

#[composable]
fn align_in_constrained_box() -> impl Widget + Clone {
    use kyute::style::*;

    let mut grid = Grid::column(GridLength::Auto);
    grid.add_row(
        Label::new("ConstrainedBox")
            .aligned(Alignment::CENTER_RIGHT)
            .height_factor(1.0)
            .fix_width(300.0),
    );
    grid.add_row(grid_layout_example());
    grid.add_row(
        Container::new(Label::new("Container"))
            //.aligned(Alignment::CENTER_RIGHT)
            .fixed_width(500.dip())
            .box_style(BoxStyle::new().fill(Color::from_hex("#b9edc788"))),
    );

    grid
}

#[composable]
fn ui_root() -> Window {
    Window::new(
        WindowBuilder::new().with_title("Layouts"),
        Flex::new(Orientation::Vertical).with(align_in_constrained_box()),
        None,
    )
}

fn main() {
    tracing_subscriber::fmt()
        .compact()
        .with_target(false)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let _app = Application::new();
    let mut env = Environment::new();
    env.set(kyute::widget::grid::SHOW_GRID_LAYOUT_LINES, true);
    application::run_with_env(ui_root, env);
    Application::shutdown();
}
