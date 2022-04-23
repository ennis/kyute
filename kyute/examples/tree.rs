use kyute::{
    application, composable,
    shell::{application::Application, winit::window::WindowBuilder},
    style::BoxStyle,
    theme,
    widget::{
        grid::GridTrackDefinition, Button, ColumnHeaders, Container, Flex, Grid, GridLength, Image, Label, Null, Popup,
        ScrollArea, TableRow, TableSelection, TableView, TableViewParams, Text, TextEdit, TitledPane, WidgetPod,
    },
    Alignment, AssetId, BoxConstraints, Color, EnvKey, Environment, Length, Orientation, SideOffsets, Size, UnitExt,
    Widget, WidgetExt, Window,
};
use kyute_common::{Atom, Data};
use std::sync::Arc;
use tracing::trace;

#[composable(cached)]
fn cell(text: impl Into<String> + Data) -> impl Widget {
    // FIXME: it's very easy to forgot the `Arc` here, and if we don't put it, we lose
    // any caching inside WidgetPod
    // (because we return a clone every time, and the cached data is reset)
    //
    // Basically, `Clone` widgets with cached data are **EVIL**
    //
    // Solution:
    // - inside WidgetPod, put cached data inside Arc => hidden cost
    // - don't make widgets `Clone`.
    Arc::new(WidgetPod::layered(Text::new(text.into()).padding(
        0.dip(),
        5.dip(),
        0.dip(),
        5.dip(),
    )))
}

#[composable]
fn edit() -> impl Widget {
    #[state]
    let mut text: Arc<str> = Arc::from("Leaf node. Doesn't contain anything.");
    TextEdit::new(text.clone()).padding(0.dip(), 5.dip(), 0.dip(), 5.dip())
}

#[composable]
fn tree_test() -> impl Widget + Clone {
    #[state]
    let mut selection = TableSelection::default();

    let mut root = TableRow::new(Atom::from("root"), cell("root"));
    //#[composable(scope)]
    for i in 0..3 {
        let id = Atom::from(format!("n.{}", i));
        let mut n1 = TableRow::new(id, cell(format!("Node {}", i)));
        n1.add_cell(1, cell("Level 1 container of nodes"));

        //#[composable(scope)]
        for j in 0..3 {
            let id = Atom::from(format!("n.{}.{}", i, j));
            let mut n2 = TableRow::new(id, cell(format!("Node {}.{}", i, j)));
            n2.add_cell(1, cell("Level 2 container of nodes"));

            //#[composable(scope)]
            for k in 0..2 {
                let id = Atom::from(format!("n.{}.{}.{}", i, j, k));
                let mut n3 = TableRow::new(id, cell(format!("Node {}.{}.{}", i, j, k)));
                n3.add_cell(1, cell("Leaf node. Doesn't contain anything."));
                n2.add_row(n3);
            }
            n1.add_row(n2);
        }
        root.add_row(n1);
    }

    let params = TableViewParams {
        selection: Some(&mut selection),
        columns: vec![
            GridTrackDefinition::new(GridLength::Fixed(200.dip())),
            GridTrackDefinition::new(GridLength::Flex(1.0)),
        ],
        column_headers: Some(ColumnHeaders::new().add(cell("Name")).add(cell("Description"))),
        main_column: 0,
        row_height: GridLength::Fixed(20.dip()),
        rows: vec![root],
        row_indent: 20.dip(),
        resizeable_columns: true,
        reorderable_rows: false,
        reorderable_columns: false,
        background: theme::palette::GREY_800.into(),
        alternate_background: theme::palette::GREY_700.into(),
        row_separator_width: 1.px(),
        column_separator_width: 1.px(),
        row_separator_background: theme::palette::GREY_900.into(),
        column_separator_background: theme::palette::GREY_900.into(),
        selected_style: BoxStyle::new().radius(8.dip()).fill(theme::palette::BLUE_800),
        ..Default::default()
    };

    let table = TableView::new(params);
    ScrollArea::new(table).fix_width(Length::Proportional(1.0))
}

#[composable]
fn ui_root() -> impl Widget {
    Window::new(WindowBuilder::new().with_title("Tree view"), tree_test(), None)
}

fn main() {
    /*tracing_subscriber::fmt()
    .compact()
    .with_target(false)
    .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
    .init();*/
    use tracing_subscriber::layer::SubscriberExt;
    tracing::subscriber::set_global_default(
        tracing_subscriber::registry().with(tracing_tracy::TracyLayer::new().with_stackdepth(0)),
    )
    .expect("set up the subscriber");
    let mut env = Environment::new();
    env.set(kyute::widget::grid::SHOW_GRID_LAYOUT_LINES, true);
    application::run_with_env(ui_root, env);
}
