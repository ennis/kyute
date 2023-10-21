use glazier::{
    kurbo::Size, raw_window_handle::HasRawWindowHandle, AppHandler, Cursor, FileDialogToken, FileInfo, IdleToken,
    KeyEvent, PointerEvent, Region, Scalable, TimerToken, WinHandler, WindowHandle,
};
use kyute2::{
    composable,
    text::{TextSpan, TextStyle},
    theme::palette,
    widget::{grid::GridArea, Frame, Grid, Null, Text},
    Alignment, AppHandle, AppLauncher, AppWindowBuilder, Color, UnitExt, Widget,
};
use kyute2_macros::grid_template;
use skia_safe as sk;
use std::{
    any::Any,
    sync::Arc,
    time::{Duration, Instant},
};
use tracing_subscriber::{layer::SubscriberExt, Layer};

////////////////////////////////////////////////////////////////////////////////////////////////////

grid_template!(GRID: [START] 100px 1fr 1fr [END] / [TOP] 50px [BOTTOM] );

#[composable]
fn main_window_contents() -> impl Widget {
    let text_style = Arc::new(
        TextStyle::new()
            .font_size(20.0)
            .font_family("Courier New")
            .color(palette::PINK_200),
    );
    let text = TextSpan::new("Hello, world!", text_style);
    let mut grid = Grid::from_template(&GRID);
    grid.add(
        GridArea {
            row: Some(0),
            column: Some(0),
            row_span: 1,
            column_span: 1,
        },
        Alignment::START,
        Alignment::START,
        Text::new(text),
    );

    Frame::new(100.percent(), 100.percent(), grid)
}

/// This function is run whenever the UI of a window needs to be rebuilt,
/// or the application receives a message that it is interested in.
#[composable]
fn application(app_handle: AppHandle) {
    // build or rebuild the main window
    AppWindowBuilder::new(main_window_contents())
        .title("Main window")
        .build(app_handle);
}

fn main() {
    tracing_subscriber::fmt()
        .compact()
        .with_target(false)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    AppLauncher::new(application).run();
}
