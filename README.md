<img align="right" width="128" height="128" src="data/logo_kyute2.png">

Kyute GUI library
========================================

This is a GUI library in Rust. Inspired by [druid](https://github.com/linebender/druid) and [moxie](https://github.com/anp/moxie).

Uses skia under the hood for rendering.
Currently **windows-only** because it's using DirectWrite for text rendering & paragraph layout.

Features
--------------------------
* Compose widgets with mostly idiomatic and straightforward Rust code
    * Few macros, designed to work well with autocomplete
* The UI is transparently invalidated whenever a piece of state changes. No need for manual change detection and invalidation.
* Widgets
    * Buttons
    * Drop downs (using native menus)
    * Sliders
    * Text line editor & validated text input
    * Images (with async loading & hot-reloading)
    * Scrollable areas
    * Hierarchical table view
    * Simple color picker (WIP)
* Layouts
    * A versatile grid layout container based on CSS grid   
* Live editing of some literals (see below)

Examples
--------------------------

Counter demo

![Counter demo](docs/screenshots/counter.png)

```rust
use kyute::{
    application, composable,
    shell::{
        application::Application,
        winit::{dpi::LogicalSize, window::WindowBuilder},
    },
    text::{FontWeight, FormattedText},
    widget::{
        grid::{GridLayoutExt, TrackSizePolicy},
        Button, Container, Grid, GridLength, Text,
    },
    Alignment, UnitExt, Widget, WidgetExt, Window,
};

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
        .padding(10.dip(), 10.dip(), 10.dip(), 10.dip()); // and some padding around it

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

    // 2 rows, sized according to the widgets placed in the row's cells.
    // 2 flex columns, available space will be divided evenly among them
    let mut grid = Grid::with_template("auto auto / 1fr 1fr");

    // Insert the widgets in the grid

    grid.insert((
        // Row 0, span all columns, center the text in the cell.
        text.centered().grid_area((0, ..)),
        // Row 1, Column 0, align the button to the top right corner of the cell.
        button_decrement.aligned(Alignment::TOP_RIGHT).grid_area((1, 0)),
        // Row 1, Column 0, align the button to the top right corner of the cell.
        button_increment.aligned(Alignment::TOP_LEFT).grid_area((1, 1)),
    ));

    // wrap grid in a container to fill the window and apply some styling to the background
    Container::new(grid)
        // a subset of CSS is supported here
        .background("linear-gradient(90deg, #111111, #333333)")
        .fill()
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
    application::run(main_window);
}
```

Live-editing of literals
--------------------------

Kyute supports live-editing of some literal expressions in composable functions by adding the `#[composable(live_literals)]`.
When this feature is active, the runtime detects changes in literal values in the source file containing the function, 
and recomposes the user interface automatically to reflect the most recent value.
This is especially powerful when combined with the support of CSS property values in some places:

![Live literals demo](docs/screenshots/live_literals.gif "live literals")

Screenshots
--------------------------

Hierarchical table view

![](docs/screenshots/table.png)

Text edit

![](docs/screenshots/text_edit.png)

Color picker

![](docs/screenshots/color_picker.png)
