//! Widget gallery
use kyute::{
    application, composable,
    shell::application::Application,
    style,
    style::{Style, WidgetState},
    theme,
    theme::palette,
    widget::{
        grid::{GridLayoutExt, TrackBreadth},
        Button, Checkbox, Grid, Null, Padding, Placeholder, StyledBox, Text, WidgetExt, WidgetPod, WidgetWrapper,
    },
    Alignment, Color, Data, Environment, Length, UnitExt, Widget, Window,
};
use kyute_shell::{
    text::{FontStyle, FormattedText},
    winit::window::WindowBuilder,
};
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Layer};

mod checkbox;
mod grids;
mod stepper;
mod table;

/// A 3-element application scaffolding: a sidebar on the left, a toolbar on the top and the rest is the content area.
#[derive(Widget)]
pub struct Scaffold {
    grid: StyledBox<Grid>,
}

impl Scaffold {
    #[composable(live_literals)]
    pub fn new(
        toolbar: impl Widget + 'static,
        sidebar: impl Widget + 'static,
        content: impl Widget + 'static,
    ) -> Scaffold {
        let mut grid = Grid::with_template("80px 2px 1fr / 300px 2px 1fr");
        // separators
        grid.insert(
            Null.fill()
                .style(
                    "[$dark-mode] background: rgb(40 40 40); \
                     [!$dark-mode] background: rgb(200 200 200);",
                )
                //.background(theme::palette::GREY_800, style::Shape::rectangle())
                .grid_row(1)
                .grid_column(1..),
        );
        grid.insert(
            Null.fill()
                .style(
                    "[$dark-mode] background: rgb(40 40 40); \
                     [!$dark-mode] background: rgb(200 200 200);",
                )
                //.background(theme::palette::GREY_800, style::Shape::rectangle())
                .grid_row(..)
                .grid_column(1),
        );

        grid.insert(toolbar.grid_area((0, 1)));
        grid.insert(sidebar.grid_area((0..1, 0)));
        grid.insert(content.grid_area((2, 2)));
        Scaffold {
            grid: grid.style(
                "[$dark-mode] background: rgb(71 71 71); \
                 [!$dark-mode] background: rgb(255 255 255);",
            ),
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Data)]
enum GalleryWidget {
    Home,
    FormattedText,
    Buttons,
    DropDown,
    ContextMenu,
    Grids,
    TextInput,
    TitledPanes,
    TreeView,
    Steppers,
    Checkboxes,
}

#[composable]
fn gallery_item(name: &str, kind: GalleryWidget, selected: &mut GalleryWidget) -> impl Widget {
    let button = Button::new(name);
    if button.clicked() {
        *selected = kind;
    }
    button.fill()
}

#[composable]
fn gallery_showcase_unimplemented(name: &str) -> Arc<WidgetPod> {
    Placeholder.fill().arc_pod()
}

#[composable]
fn root_view() -> impl Widget + Clone {
    #[state]
    let mut selected = GalleryWidget::Home;
    #[state]
    let mut dark_mode = false;

    // widget list
    let mut items = Grid::column(TrackBreadth::Flex(1.0));
    items.set_implicit_row_size(30.dip());
    items.set_row_gap(8.px());
    items.insert(gallery_item("Home", GalleryWidget::Home, &mut selected));
    items.insert(gallery_item("Buttons", GalleryWidget::Buttons, &mut selected));
    items.insert(gallery_item("Steppers", GalleryWidget::Steppers, &mut selected));
    items.insert(gallery_item(
        "Formatted Text",
        GalleryWidget::FormattedText,
        &mut selected,
    ));
    items.insert(gallery_item("Drop down", GalleryWidget::DropDown, &mut selected));
    items.insert(gallery_item("Checkboxes", GalleryWidget::Checkboxes, &mut selected));
    items.insert(gallery_item("Grids", GalleryWidget::Grids, &mut selected));
    items.insert(gallery_item("Context menu", GalleryWidget::ContextMenu, &mut selected));
    items.insert(gallery_item("Titled panes", GalleryWidget::TitledPanes, &mut selected));
    items.insert(gallery_item("Text input", GalleryWidget::TextInput, &mut selected));
    items.insert(gallery_item("Tree view", GalleryWidget::TreeView, &mut selected));
    items.insert(Checkbox::new(dark_mode).on_toggled(|v| dark_mode = v));

    // content pane
    let showcase = match selected {
        GalleryWidget::Home => gallery_showcase_unimplemented("Home"),
        GalleryWidget::FormattedText => gallery_showcase_unimplemented("Formatted text"),
        GalleryWidget::Buttons => gallery_showcase_unimplemented("Buttons"),
        GalleryWidget::Steppers => stepper::showcase(),
        GalleryWidget::DropDown => gallery_showcase_unimplemented("Drop-downs"),
        GalleryWidget::ContextMenu => gallery_showcase_unimplemented("Context menus"),
        GalleryWidget::Grids => gallery_showcase_unimplemented("Grids"),
        GalleryWidget::TextInput => gallery_showcase_unimplemented("Text input"),
        GalleryWidget::TitledPanes => gallery_showcase_unimplemented("Titled panes"),
        GalleryWidget::TreeView => table::showcase(),
        GalleryWidget::Checkboxes => checkbox::showcase(),
    };

    Scaffold::new(Null, items.padding(8.dip()), showcase)
        .fill()
        .theme(if dark_mode {
            theme::Theme::Dark
        } else {
            theme::Theme::Light
        })
        .arc_pod()
}

#[composable(cached)]
fn main_window() -> impl Widget + Clone {
    Window::new(
        WindowBuilder::new().with_title("Widget gallery"),
        root_view().text_color(theme::palette::GREY_100),
        None,
    )
}

fn main() {
    let subscriber = tracing_subscriber::Registry::default().with(
        tracing_tree::HierarchicalLayer::new(4)
            .with_bracketed_fields(true)
            .with_filter(tracing_subscriber::EnvFilter::from_default_env()),
    );
    tracing::subscriber::set_global_default(subscriber).unwrap();
    let mut env = Environment::new();
    //env.set(kyute::widget::grid::SHOW_GRID_LAYOUT_LINES, true);
    application::run_with_env(main_window, env);
}
