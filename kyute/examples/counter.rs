use kyute::{
    application, composable,
    shell::{
        application::Application,
        winit::{dpi::LogicalSize, window::WindowBuilder},
    },
    widget::{grid::GridTrackDefinition, Button, Grid, GridLength, Text},
    Alignment, Widget, WidgetExt, Window,
};

// All functions creating widgets or using features like `#[state]` must be marked as `#[composable]`.
#[composable]
fn counter_demo() -> impl Widget {
    // Declare persistent state with `#[state]`.
    // The value will be remembered between invocations of `counter_demo` at the same position in the call tree.
    #[state]
    let mut counter = 0;

    // text element
    let text = Text::new(format!("Counter value: {}", counter));

    // Buttons to increment and decrement the counter.
    // The framework will detect if the value of `counter` changed, and will re-run the function if this is the case.
    // Note that the callback passed to `on_clicked` is run immediately, so you can borrow stuff from the surrounding scope.
    let button_increment = Button::new("+".to_string()).on_clicked(|| counter += 1);
    let button_decrement = Button::new("-".to_string()).on_clicked(|| counter -= 1);

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

    // Layout the widgets in a grid.
    //
    // Grids are the primary layout mechanism in kyute.
    // They are modeled after the CSS Grid specification.
    let mut grid = Grid::new();

    // Define grid rows and columns
    // 2 flex columns, available space will be divided evenly among them
    grid.push_column_definition(GridTrackDefinition::new(GridLength::Flex(1.0)));
    grid.push_column_definition(GridTrackDefinition::new(GridLength::Flex(1.0)));
    // 2 rows, sized according to the widgets placed in the row's cells.
    grid.push_row_definition(GridTrackDefinition::new(GridLength::Auto));
    grid.push_row_definition(GridTrackDefinition::new(GridLength::Auto));

    // Insert the widgets in the grid
    // Row 0, span all columns, Z-order 0, center the text in the cell.
    grid.add_item(0, .., 0, text.centered());
    // Row 1, Column 0, Z-order 0, align the button to the top right corner of the cell.
    grid.add_item(1, 0, 0, button_decrement.aligned(Alignment::TOP_RIGHT));
    // Row 1, Column 0, Z-order 0, align the button to the top right corner of the cell.
    grid.add_item(1, 1, 0, button_increment.aligned(Alignment::TOP_LEFT));

    grid
}

#[composable]
fn main_window() -> impl Widget {
    // Create the main window widget.
    // For now we use a for of winit under the hood, hence the `WindowBuilder`.
    Window::new(
        WindowBuilder::new()
            .with_title("Counter demo")
            .with_inner_size(LogicalSize::new(200, 100)),
        counter_demo(),
        None,
    )
}

fn main() {
    let _app = Application::new();
    application::run(main_window);
    Application::shutdown();
}
