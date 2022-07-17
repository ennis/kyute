use kyute::{
    application, composable,
    shell::{
        application::Application,
        winit::{dpi::LogicalSize, window::WindowBuilder},
    },
    style,
    text::{FontWeight, FormattedText},
    widget::{grid::GridLayoutExt, Button, Grid, Text, WidgetExt},
    Alignment, UnitExt, Widget, Window,
};
use kyute_common::Color;

// All functions creating widgets or using features like `#[state]` must be marked as `#[composable]`.
#[composable]
fn counter_demo() -> impl Widget {
    // Declare persistent state with `#[state]`.
    // The value will be remembered between invocations of `counter_demo` at the same position in the call tree.
    #[state]
    let mut counter = 0;

    // Text element with attributes
    let text = Text::new(
        FormattedText::from(format!("Counter value: {}", counter))
            .font_size(16.0)
            .attribute(14.., FontWeight::BOLD),
    )
    .padding(10.dip()); // and some padding around it

    // Buttons to increment and decrement the counter.
    // The framework will detect if the value of `counter` changed, and will re-run the function if this is the case.
    // Note that the callback passed to `on_clicked` is run immediately, so you can borrow stuff from the surrounding scope.

    //let button_increment = Button::new("+".to_string()).on_clicked(|| counter += 1);
    //let button_decrement = Button::new("-".to_string()).on_clicked(|| counter -= 1);

    // Another way of writing the code above without closures:
    //
    //    let button_increment = Button::new("+".to_string());
    //    let button_decrement = Button::new("-".to_string());
    //    if button_increment.clicked() {
    //        counter += 1;
    //    }
    //    if button_decrement.clicked() {
    //        counter -= 1;
    //    }
    //

    let mut grid = Grid::with_template("70 70 / 1fr 1fr");
    grid.insert((
        Text::new(FormattedText::from(format!("Counter value: {}", counter)).attribute(14.., FontWeight::BOLD))
            .color(Color::from_rgb_u8(200, 200, 200))
            .grid_column_span(2),
        Button::new("+").on_click(|| counter += 1).padding(5.dip()),
        Button::new("-").on_click(|| counter -= 1).padding(5.dip()),
    ));

    grid.background("rgb(71 71 71)", style::Shape::rectangle()).fill()
}

#[composable]
fn main_window() -> impl Widget {
    // Create the main window widget.
    // For now we use a fork of winit under the hood, hence the `WindowBuilder`.
    Window::new(
        WindowBuilder::new()
            .with_title("Counter demo")
            .with_inner_size(LogicalSize::new(200, 100)),
        counter_demo(),
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
