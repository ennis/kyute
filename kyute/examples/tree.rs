use kyute::{
    application, composable,
    shell::{application::Application, winit::window::WindowBuilder},
    style::Style,
    theme,
    widget::{
        grid::GridTemplate, table, table::TableViewStyle, Button, Flex, Grid, Image, Label, Null, Popup, ScrollArea,
        TableSelection, TableView, TableViewParams, Text, TextEdit, TitledPane, WidgetExt, WidgetPod,
    },
    Alignment, AssetId, BoxConstraints, Color, EnvKey, Environment, Length, Orientation, SideOffsets, Size, UnitExt,
    Widget, Window,
};
use kyute_common::{Atom, Data};
use std::{convert::TryFrom, sync::Arc};
use tracing::trace;
use tracing_subscriber::{layer::SubscriberExt, Layer};

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
    Arc::new(WidgetPod::with_surface(Text::new(text.into()).padding_trbl(
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
    TextEdit::new(text.clone()).padding_trbl(0.dip(), 5.dip(), 0.dip(), 5.dip())
}

#[composable]
fn tree_test() -> impl Widget {
    #[state]
    let mut selection = TableSelection::default();

    let col_name = table::Column::new(cell("Name")).outline().resizable(200.0);
    let col_description = table::Column::new(cell("Description")).resizable(400.0);

    let mut root_row = table::Row::new(Atom::from("root")).expanded(true);
    root_row.push_cell(&col_name, cell("root"));
    //#[composable(scope)]
    for i in 0..5 {
        let id = Atom::from(format!("n.{i}"));
        let mut n1 = table::Row::new(id);
        n1.push_cell(&col_name, cell(format!("Node {i}")));
        n1.push_cell(&col_description, cell("Level 1 node"));

        //#[composable(scope)]
        for j in 0..5 {
            let id = Atom::from(format!("n.{i}.{j}"));
            let mut n2 = table::Row::new(id);
            n2.push_cell(&col_name, cell(format!("Node {i}.{j}")));
            n2.push_cell(&col_description, cell("Level 2 node"));

            //#[composable(scope)]
            for k in 0..5 {
                let id = Atom::from(format!("n.{i}.{j}.{k}"));
                let mut n3 = table::Row::new(id);
                n3.push_cell(&col_name, cell(format!("Node {i}.{j}.{k}")));
                n3.push_cell(&col_description, cell("Leaf node"));
                n2.add_row(n3);
            }

            n1.add_row(n2);
        }
        root_row.add_row(n1);
    }

    let mut params = TableViewParams::default().column(col_name).column(col_description);
    params.selection = Some(&mut selection);
    params.rows.push(root_row);
    params.style = TableViewStyle {
        indentation: 20.dip(),
        background: theme::palette::GREY_800.into(),
        alternate_background: theme::palette::GREY_700.into(),
        row_separator_width: 1.px(),
        column_separator_width: 1.px(),
        row_separator_background: theme::palette::GREY_900.into(),
        column_separator_background: theme::palette::GREY_900.into(),
        selected_style: Style::try_from("border-radius: 8px; background: #1565c0;").unwrap(),
        ..Default::default()
    };

    let table = TableView::new(params);
    ScrollArea::new(table).fill()
}

#[composable]
fn ui_root() -> impl Widget {
    Window::new(WindowBuilder::new().with_title("Tree view"), tree_test(), None)
}

fn main() {
    let subscriber = tracing_subscriber::Registry::default().with(
        tracing_tree::HierarchicalLayer::new(4)
            .with_bracketed_fields(true)
            .with_filter(tracing_subscriber::EnvFilter::from_default_env()),
    );
    tracing::subscriber::set_global_default(subscriber).unwrap();
    let mut env = Environment::new();
    //env.set(&kyute::widget::grid::SHOW_GRID_LAYOUT_LINES, true);
    application::run_with_env(ui_root, env);
}
