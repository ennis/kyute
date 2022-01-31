use euclid::Size2D;
use kyute::{
    application, composable,
    shell::application::Application,
    theme,
    widget::{Button, ConstrainedBox, Flex, Grid, GridLength, Text},
    Alignment, BoxConstraints, Orientation, Size, Widget, WidgetExt, WidgetPod, Window,
};
use kyute_shell::winit::window::WindowBuilder;

#[composable(uncached)]
fn fixed_size_widget(w: f64, h: f64, name: &str) -> impl Widget {
    Text::new(name.to_string()).fix_size(Size::new(w,h))
}

#[composable(uncached)]
fn grid_layout_example() -> impl Widget + Clone {
    Grid::new()
        .column(GridLength::Fixed(100.0))
        .column(GridLength::Auto)
        .column(GridLength::Flex(1.0))
        .row(GridLength::Fixed(100.0))
        .row(GridLength::Flex(1.0))
        .item(0, 0, fixed_size_widget(50.0, 50.0, "(0,0)"))
        .item(0, 1, fixed_size_widget(50.0, 50.0, "(0,1)"))
        .item(0, 2, fixed_size_widget(50.0, 50.0, "(0,2)"))
        .item(1, 0, fixed_size_widget(50.0, 50.0, "(1,0)"))
        .item(1, 1, fixed_size_widget(50.0, 50.0, "(1,1)"))
        .item(1, 2, fixed_size_widget(50.0, 50.0, "(1,2)"))
}

#[composable(uncached)]
fn align_in_constrained_box() -> impl Widget + Clone {
    Flex::new(Orientation::Horizontal)
        .with(
            Text::new("ConstrainedBox".into())
                .aligned(Alignment::CENTER_RIGHT)
                .fix_width(300.0),
        )
        .with(ConstrainedBox::new(
            BoxConstraints::loose(Size::new(500.0, 500.0)),
            grid_layout_example(),
        ))
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

    tracing_subscriber::fmt()
        .compact()
        .with_target(false)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    application::run(ui_root, theme::get_default_application_style());

    Application::shutdown();
}