use anyhow::Error;
use kyute::{
    application, cache, composable,
    shell::{application::Application, winit::window::WindowBuilder},
    style::{Border, BoxStyle},
    text::{Attribute, FormattedText},
    theme,
    widget::{
        grid::{AlignItems, TrackSizePolicy},
        Action, Canvas, ConstrainedBox, Container, ContextMenu, DragController, Flex, Formatter, Grid, GridLength,
        Image, Label, Menu, MenuItem, Null, Slider, Text, TextEdit, TextInput, Thumb, TitledPane, ValidationResult,
    },
    Alignment, AssetId, BoxConstraints, Color, EnvKey, Environment, Offset, Orientation, Point, Size, UnitExt, Widget,
    WidgetExt, WidgetPod, Window,
};
use kyute_common::Transform;
use kyute_text::{Selection, TextAlignment};
use std::sync::Arc;
use tracing::info;

#[composable(cached)]
fn canvas_playground() -> impl Widget + Clone {
    #[state]
    let mut offset = Offset::zero();
    #[state]
    let mut scale = 1.0;
    #[state]
    let mut tmp_offset = Offset::zero();

    let mut grid = Grid::with_template("auto-flow auto / 200 5 1fr");
    /*let col_label = GridTrackDefinition::new(GridLength::Fixed(200.dip()));
    let col_sep = GridTrackDefinition::new(GridLength::Fixed(5.dip()));
    let col_widgets = GridTrackDefinition::new(GridLength::Flex(1.0));
    grid.push_column_definition(col_label);
    grid.push_column_definition(col_sep);
    grid.push_column_definition(col_widgets);*/

    grid.set_align_items(AlignItems::Baseline);

    grid.insert((
        ////////////////////
        Label::new("Offset X"),
        (),
        TextInput::number(offset.x).on_value_changed(|x| offset.x = x),
        ////////////////////
        Label::new("Offset Y"),
        (),
        TextInput::number(offset.y).on_value_changed(|y| offset.y = y),
        ////////////////////
        Label::new("Scale"),
        (),
        TextInput::number(scale).on_value_changed(|s| scale = s),
    ));

    /*grid.add_item(0, 0, 0, Label::new("Offset X"));
    grid.add_item(0, 2, 0, TextInput::number(offset.x).on_value_changed(|x| offset.x = x));

    grid.add_item(1, 0, 0, Label::new("Offset Y"));
    grid.add_item(1, 2, 0, TextInput::number(offset.y).on_value_changed(|y| offset.y = y));

    grid.add_item(2, 0, 0, Label::new("Scale"));
    grid.add_item(2, 2, 0, TextInput::number(scale).on_value_changed(|s| scale = s));*/

    let mut canvas = Canvas::new();
    let canvas_transform = offset.to_transform().then_scale(scale, scale);
    let inv_transform = canvas_transform.inverse().unwrap();
    canvas.set_transform(canvas_transform);
    //canvas.set_bounds(0, 0, 100.percent(), 100.percent());
    canvas.add_item(0.0, 0.0, Label::new("Artist: 少女理論観測所"));

    // make a draggable canvas
    let drag_controller = DragController::new(canvas)
        .on_started(|| tmp_offset = offset)
        .on_delta(|delta| offset = tmp_offset + inv_transform.transform_vector(delta));

    // context menu handler
    grid.push_row_definition(TrackSizePolicy::new(GridLength::Flex(1.0)));
    let add_node_action = Action::new().on_triggered(|| eprintln!("add node"));
    let add_comment_action = Action::new().on_triggered(|| eprintln!("add comment"));

    let context_menu = Menu::new(vec![
        MenuItem::Action {
            text: "Add Node".to_string(),
            action: add_node_action,
        },
        MenuItem::Action {
            text: "Add Comment".to_string(),
            action: add_comment_action,
        },
    ]);

    let context_menu_area = Container::new(ContextMenu::new(context_menu, drag_controller))
        .box_style(BoxStyle::new().border(Border::inside(2.px()).paint(Color::from_hex("#FFB500"))));

    grid.place(GridArea::after_last_row(), context_menu_area);

    Container::new(grid).box_style(BoxStyle::new().fill(theme::palette::BLUE_GREY_800))
}

#[composable]
fn ui_root() -> impl Widget {
    Window::new(
        WindowBuilder::new().with_title("Canvas playground"),
        canvas_playground(),
        None,
    )
}

fn main() {
    tracing_subscriber::fmt()
        .compact()
        .with_target(false)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let _app = Application::new();
    application::run(ui_root);
    Application::shutdown();
}
