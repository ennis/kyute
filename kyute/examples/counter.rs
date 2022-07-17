use kyute::{
    application, composable,
    shell::{
        application::Application,
        winit::{dpi::LogicalSize, window::WindowBuilder},
    },
    style,
    text::{FontWeight, FormattedText},
    widget::{grid::GridLayoutExt, BaseTextEdit, Button, Grid, Text, WidgetExt},
    Alignment, Color, UnitExt, Widget, Window,
};
use std::sync::Arc;

// All functions creating widgets or using features like `#[state]` must be marked as `#[composable]`.
#[composable]
fn counter_demo() -> impl Widget {
    // Declare persistent state with `#[state]`.
    // The value will be remembered between invocations of `counter_demo` at the same position in the call tree.
    #[state]
    let mut counter = 0;
    #[state]
    let mut text: Arc<str> = Arc::from("Hello");

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

    let mut grid = Grid::with_template("40px 40px / 1fr 1fr");
    grid.insert((
        Text::new(FormattedText::from(format!("Counter value: {}", counter)).attribute(14.., FontWeight::BOLD))
            .centered()
            .grid_column_span(2),
        Button::new("+")
            .on_click(|| counter += 1)
            .padding(5.dip())
            .horizontal_alignment(Alignment::END)
            .vertical_alignment(Alignment::END),
        Button::new("-")
            .on_click(|| counter -= 1)
            .padding(5.dip())
            .horizontal_alignment(Alignment::START)
            .vertical_alignment(Alignment::END),
        Text::new("Text edit:"),
        BaseTextEdit::new(text.clone()).on_editing_finished(|new_text| text = new_text),
    ));

    grid.centered()
        .frame(100.percent(), 100.percent())
        .background("rgb(71 71 71)", style::Shape::rectangle())
        .text_color(Color::from_rgb_u8(200, 200, 200))
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
