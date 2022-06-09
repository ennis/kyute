use kyute::{
    application, composable,
    shell::{
        application::Application,
        winit::{dpi::LogicalSize, window::WindowBuilder},
    },
    text::{FontWeight, FormattedText},
    theme,
    theme::palette::GREY_800,
    widget::{
        grid::{FlowDirection, GridLayoutExt, TrackSizePolicy},
        Button, Container, Grid, GridLength, Image, Scaling, Text,
    },
    Alignment, UnitExt, Widget, WidgetExt, Window,
};

#[composable(live_literals)]
fn live_toolbar() -> impl Widget {
    let mut toolbar_grid = Grid::with_template("40 10 / 40 / 10 10");
    toolbar_grid.set_auto_flow(FlowDirection::Column);

    toolbar_grid.insert((
        Image::from_uri("data/icons/file_folder.png", Scaling::Contain)
            .colorize(GREY_800)
            .fix_size(32.dip(), 32.dip())
            .centered(),
        Text::new("Open").color(GREY_800).centered(),
    ));

    let toolbar = Container::new(toolbar_grid)
        .background("linear-gradient(90deg,#AAAAAA,#CCCCCC)")
        .centered();

    toolbar
}

#[composable]
fn main_window() -> impl Widget {
    Window::new(
        WindowBuilder::new()
            .with_title("Live literals demo")
            .with_inner_size(LogicalSize::new(200, 100)),
        live_toolbar(),
        None,
    )
}

fn main() {
    tracing_subscriber::fmt()
        .compact()
        .with_target(false)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    application::run(main_window);
}
