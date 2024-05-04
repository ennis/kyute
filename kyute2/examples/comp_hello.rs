use std::{cell::OnceCell, sync::Arc};

use kurbo::Insets;
use kyute2::{
    text::{TextSpan, TextStyle},
    theme,
    widget::{button, Clickable, Frame, Null, Text},
    window::{UiHostWindowHandler, UiHostWindowOptions},
    AppLauncher, UnitExt, Widget,
};
use tracing_subscriber::layer::SubscriberExt;

////////////////////////////////////////////////////////////////////////////////////////////////////

fn main_window_contents() -> impl Widget {
    let text_style = Arc::new(
        TextStyle::new()
            .font_size(20.0)
            .font_family("Courier New")
            .color(theme::palette::PINK_200),
    );
    //let text = TextSpan::new("Hello, world!", text_style);
    Frame::new(100.percent(), 100.percent(), button("hello"))
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
