use kyute::{application, composable, shell::application::Application, style::define_theme, theme, widget::{Button, ConstrainedBox, Container, Flex, Grid, GridLength, Text}, Alignment, BoxConstraints, Color, Environment, Orientation, Size, Widget, WidgetExt, WidgetPod, Window, EnvKey};
use kyute::style::{standard_theme, ThemeData};
use kyute_shell::AssetId;
use kyute_shell::winit::window::WindowBuilder;

#[composable(uncached)]
fn fixed_size_widget(w: f64, h: f64, name: &str) -> impl Widget {
    // TODO "debug widget" that draws a background pattern, with a border
    Text::new(name.to_string()).fix_size(Size::new(w, h))
}

#[composable(uncached)]
fn grid_layout_example() -> impl Widget + Clone {
    Grid::new()
        .with_column(GridLength::Fixed(100.0))
        .with_column(GridLength::Auto)
        .with_column(GridLength::Fixed(100.0))
        .with_row(GridLength::Fixed(100.0))
        .with_row(GridLength::Flex(1.0))
        .with(0, 0, fixed_size_widget(50.0, 50.0, "(0,0)"))
        .with(0, 1, fixed_size_widget(50.0, 50.0, "(0,1)"))
        .with(0, 2, fixed_size_widget(50.0, 50.0, "(0,2)"))
        .with(1, 0, fixed_size_widget(50.0, 50.0, "(1,0)"))
        .with(1, 1..=2, fixed_size_widget(150.0, 50.0, "(1,1)").centered())
    //.item(1, 2, fixed_size_widget(50.0, 50.0, "(1,2)"))
}

#[composable(uncached)]
fn align_in_constrained_box() -> impl Widget + Clone {
    use kyute::style::*;

    Flex::new(Orientation::Horizontal)
        .with(
            Text::new("ConstrainedBox".into())
                .aligned(Alignment::CENTER_RIGHT)
                .height_factor(1.0)
                .fix_width(300.0),
        )
        .with(grid_layout_example())
        .with(
            Container::new(Text::new("Container".into()))
                //.aligned(Alignment::CENTER_RIGHT)
                .fix_width(500.0)
                .box_style(BoxStyle::new().fill(Color::from_hex("#b9edc788"))),
        )
}

#[composable]
fn ui_root() -> WidgetPod {
    WidgetPod::new(Window::new(
        WindowBuilder::new().with_title("Layouts"),
        WidgetPod::new(Flex::new(Orientation::Vertical).with(align_in_constrained_box())),
        None,
    ))
}

const STANDARD_THEME_CONFIG: AssetId<ThemeData> = AssetId::new("../standard-theme.json");

fn main() {
    let _app = Application::new();

    let mut env = Environment::new();
    standard_theme::load(&mut env, STANDARD_THEME_CONFIG);
    env.push(kyute::widget::grid::SHOW_GRID_LAYOUT_LINES, true);

    tracing_subscriber::fmt()
        .compact()
        .with_target(false)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    application::run(ui_root, env);

    Application::shutdown();
}
