//! Widget gallery
use crate::GalleryWidget::ContextMenu;
use kyute::{
    application, composable,
    shell::application::Application,
    style,
    style::{Style, VisualState},
    theme,
    theme::palette,
    widget::{
        grid::{GridLayoutExt, TrackBreadth},
        Button, Grid, Null, Padding, StyledBox, Text, WidgetExt, WidgetPod, WidgetWrapper,
    },
    Alignment, Color, Data, Environment, Length, UnitExt, Widget, Window,
};
use kyute_shell::{
    text::{FontStyle, FormattedText},
    winit::window::WindowBuilder,
};
use std::sync::Arc;

mod grids;

/// A 3-element application scaffolding: a sidebar on the left, a toolbar on the top and the rest is the content area.
#[derive(WidgetWrapper)]
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
        let mut grid = Grid::with_template("150px 2px 1fr / 300px 2px 1fr");
        // separators
        grid.insert(
            Null.fill()
                .background(theme::palette::GREY_800, style::Shape::rectangle())
                .grid_row(1)
                .grid_column(..),
        );
        grid.insert(
            Null.fill()
                .background(theme::palette::GREY_800, style::Shape::rectangle())
                .grid_row(..)
                .grid_column(1),
        );

        grid.insert(toolbar.grid_area((0, 1)));
        grid.insert(sidebar.grid_area((0..1, 0)));
        grid.insert(content.grid_area((2, 2)));
        Scaffold {
            grid: grid.style("background: rgb(71 71 71)"),
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
}

#[composable]
fn gallery_sidebar_item(name: &str, kind: GalleryWidget, selected: &mut GalleryWidget) -> impl Widget {
    let button = Button::new(name);
    if button.clicked() {
        *selected = kind;
    }
    button.fill()
}

#[composable]
fn root_view() -> impl Widget + Clone {
    #[state]
    let mut selected = GalleryWidget::Home;

    // widget list
    let mut widget_list = Grid::column(TrackBreadth::Flex(1.0));
    widget_list.set_implicit_row_size(40.dip());
    widget_list.set_row_gap(8.px());

    widget_list.insert((
        gallery_sidebar_item("Home", GalleryWidget::Home, &mut selected),
        gallery_sidebar_item("Buttons", GalleryWidget::Buttons, &mut selected),
        gallery_sidebar_item("Formatted Text", GalleryWidget::FormattedText, &mut selected),
        gallery_sidebar_item("Drop down", GalleryWidget::DropDown, &mut selected),
        gallery_sidebar_item("Grids", GalleryWidget::Grids, &mut selected),
        gallery_sidebar_item("Context menu", GalleryWidget::ContextMenu, &mut selected),
        gallery_sidebar_item("Titled panes", GalleryWidget::TitledPanes, &mut selected),
        gallery_sidebar_item("Text input", GalleryWidget::TextInput, &mut selected),
        gallery_sidebar_item("Tree view", GalleryWidget::TreeView, &mut selected),
    ));

    // content pane
    let right_panel = match selected {
        GalleryWidget::Home => Text::new(
            FormattedText::from("Home (TODO)")
                .font_size(30.0)
                .font_style(FontStyle::Italic),
        )
        .centered(),
        GalleryWidget::FormattedText => Text::new(
            FormattedText::from("Formatted text (TODO)")
                .font_size(30.0)
                .font_style(FontStyle::Italic),
        )
        .centered(),
        GalleryWidget::Buttons => Text::new(
            FormattedText::from("Buttons (TODO)")
                .font_size(30.0)
                .font_style(FontStyle::Italic),
        )
        .centered(),
        GalleryWidget::DropDown => Text::new(
            FormattedText::from("Drop downs (TODO)")
                .font_size(30.0)
                .font_style(FontStyle::Italic),
        )
        .centered(),
        GalleryWidget::ContextMenu => Text::new(
            FormattedText::from("Context menu (TODO)")
                .font_size(30.0)
                .font_style(FontStyle::Italic),
        )
        .centered(),
        GalleryWidget::Grids => Text::new(
            FormattedText::from("Grids (TODO)")
                .font_size(30.0)
                .font_style(FontStyle::Italic),
        )
        .centered(),
        GalleryWidget::TextInput => Text::new(
            FormattedText::from("Text input (TODO)")
                .font_size(30.0)
                .font_style(FontStyle::Italic),
        )
        .centered(),
        GalleryWidget::TitledPanes => Text::new(
            FormattedText::from("Titled panes (TODO)")
                .font_size(30.0)
                .font_style(FontStyle::Italic),
        )
        .centered(),
        GalleryWidget::TreeView => Text::new(
            FormattedText::from("Tree View (TODO)")
                .font_size(30.0)
                .font_style(FontStyle::Italic),
        )
        .centered(),
    };

    Arc::new(WidgetPod::new(
        Scaffold::new(Null, widget_list.padding(8.dip()), right_panel).fill(),
    ))
}

#[composable(cached)]
fn main_window() -> impl Widget + Clone {
    Window::new(WindowBuilder::new().with_title("Widget gallery"), root_view(), None)
}

fn main() {
    tracing_subscriber::fmt()
        .compact()
        .with_target(false)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    let mut env = Environment::new();
    //env.set(kyute::widget::grid::SHOW_GRID_LAYOUT_LINES, true);
    application::run_with_env(main_window, env);
}
