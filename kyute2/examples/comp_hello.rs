use glazier::{
    kurbo::Size, raw_window_handle::HasRawWindowHandle, AppHandler, Cursor, FileDialogToken, FileInfo, IdleToken,
    KeyEvent, PointerEvent, Region, Scalable, TimerToken, WinHandler, WindowHandle,
};
use kyute2::{
    composable,
    widget::{Frame, Null},
    AppHandle, AppLauncher, AppWindowBuilder, UnitExt, Widget,
};
use skia_safe as sk;
use std::{
    any::Any,
    time::{Duration, Instant},
};
use tracing_subscriber::{layer::SubscriberExt, Layer};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[composable]
fn main_window_contents() -> impl Widget {
    Frame::new(100.percent(), 100.percent(), Null)
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
