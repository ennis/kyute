use std::{cell::OnceCell, sync::Arc};

use kurbo::Insets;
use kyute2::{
    widget::Null,
    window::{UiHostWindowHandler, UiHostWindowOptions},
    AppLauncher, Widget,
};
use tracing_subscriber::layer::SubscriberExt;

////////////////////////////////////////////////////////////////////////////////////////////////////

fn main_window_contents() -> impl Widget {
    Null
}

fn main() {
    tracing_subscriber::fmt()
        .compact()
        .with_target(false)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    /*use tracing_subscriber::layer::SubscriberExt;
    tracing::subscriber::set_global_default(tracing_subscriber::registry().with(tracing_tracy::TracyLayer::new()))
        .expect("set up the subscriber");*/

    let mut launcher = AppLauncher::new();

    launcher.run(UiHostWindowHandler::new(
        Box::new(main_window_contents()),
        UiHostWindowOptions::default(),
    ));
}
