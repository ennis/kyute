use std::sync::Arc;

use kyute2::{
    text::TextStyle,
    theme,
    widgets::{button, Align, Flex, Frame},
    window::{UiHostWindowHandler, UiHostWindowOptions},
    Alignment, AppLauncher, UnitExt, Widget, WidgetExt, WidgetPtr,
};

////////////////////////////////////////////////////////////////////////////////////////////////////

fn main_window_contents() -> WidgetPtr {
    let text_style = Arc::new(
        TextStyle::new()
            .font_size(20.0)
            .font_family("Courier New")
            .color(theme::palette::PINK_200),
    );

    let mut row = Flex::row();
    row.push(button("hello"));
    row.push(button("world"));
    row.push(button("hello"));
    row.push(button("world"));
    let row = row.to_widget_ptr();

    Align::new(
        Alignment::CENTER,
        Alignment::CENTER,
        Frame::new(20.percent(), 20.percent(), row),
    )
}

/*
fn slider(cx: &ImCtx) {

    // called on layout, during which drawing functions do nothing
    // -> receives box constraints and three output variables: width, height and baseline


    let width = bc.min.width;



    // geometry
    let r = space();    // available space as rectangle
    let knob_radius = r.height / 2.0;
    let mut pressed : bool = var();

    let trk_start = point(r.left() + knob_radius, r.center().y);
    let trk_end = point(r.right() - knob_radius, r.center().y);

    let trk = horizontal(trk_start, trk_end);

    let trk_rect = stroke(trk, Center, 2.0);
    let knob_pos = trk.line().lerp(slider_pos);

    let knob_circle = circle(knob_pos, knob_radius);
    let label_pos = knob_circle.north().move_up(2.0);


    // drawing
    fill(trk_rect, Color(...));
    drop_shadow(knob_circle, ...);
    fill(knob_circle, Color(...));
    label(label_pos, format!("{slider_pos}"), Center, Baseline);    // Alignment::Center, Alignment::Baseline



    // interacting
    match state {
        Idle => {
            match r.interact() {
                MousePress(p) => {
                    state = Pressed;
                },
                _ => {}
            }
        },
        Pressed => {
            match r.interact() {
                MouseMove(p) => {
                    slider_pos.set(trk.line().inv_lerp(p.x));
                }
                MouseRelease(_) | MouseExit => {
                    state = Idle;
                }
            }
        },
    }
}*/

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
        main_window_contents(),
        UiHostWindowOptions::default(),
    ));
}
