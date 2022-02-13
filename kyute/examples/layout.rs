use kyute::{
    application, composable,
    shell::application::Application,
    style::ThemeData,
    theme,
    widget::{Button, ConstrainedBox, Container, Flex, Grid, GridLength, Label},
    Alignment, BoxConstraints, Color, EnvKey, Environment, Orientation, Size, Widget, WidgetExt,
    WidgetPod, Window,
};
use kyute_shell::{winit::window::WindowBuilder, AssetId};

#[composable(uncached)]
fn fixed_size_widget(w: f64, h: f64, name: &str) -> impl Widget {
    // TODO "debug widget" that draws a background pattern, with a border
    Label::new(name.to_string()).fix_size(Size::new(w, h))
}

#[composable(uncached)]
fn grid_layout_example() -> impl Widget + Clone {
    let mut grid = Grid::with_rows_columns(
        [GridLength::Fixed(100.0), GridLength::Flex(1.0)],
        [
            GridLength::Fixed(100.0),
            GridLength::Auto,
            GridLength::Fixed(100.0),
        ],
    );

    grid.add(0, 0, fixed_size_widget(50.0, 50.0, "(0,0)"));
    grid.add(0, 1, fixed_size_widget(50.0, 50.0, "(0,1)"));
    grid.add(0, 2, fixed_size_widget(50.0, 50.0, "(0,2)"));
    grid.add(1, 0, fixed_size_widget(50.0, 50.0, "(1,0)"));
    grid.add(1, 1..=2, fixed_size_widget(150.0, 50.0, "(1,1)").centered());

    grid
}

#[composable(uncached)]
fn align_in_constrained_box() -> impl Widget + Clone {
    use kyute::style::*;

    let mut grid = Grid::column(GridLength::Auto);
    grid.add_row(
        Label::new("ConstrainedBox".into())
            .aligned(Alignment::CENTER_RIGHT)
            .height_factor(1.0)
            .fix_width(300.0),
    );
    grid.add_row(grid_layout_example());
    grid.add_row(
        Container::new(Label::new("Container".into()))
            //.aligned(Alignment::CENTER_RIGHT)
            .fix_width(500.0)
            .box_style(BoxStyle::new().fill(Color::from_hex("#b9edc788"))),
    );

    grid
}

#[composable]
fn ui_root() -> WidgetPod {
    WidgetPod::new(Window::new(
        WindowBuilder::new().with_title("Layouts"),
        WidgetPod::new(Flex::new(Orientation::Vertical).with(align_in_constrained_box())),
        None,
    ))
}

fn main() {
    let _app = Application::new();

    let mut env = Environment::new();
    theme::setup_default_style(&mut env);
    env.set(kyute::widget::grid::SHOW_GRID_LAYOUT_LINES, true);

    tracing_subscriber::fmt()
        .compact()
        .with_target(false)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    application::run(ui_root, env);

    Application::shutdown();
}
