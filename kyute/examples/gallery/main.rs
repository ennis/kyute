//! Widget gallery
use crate::GalleryWidget::ContextMenu;
use kyute::{
    application, composable,
    shell::application::Application,
    style::{BoxStyle, LinearGradient, VisualState},
    theme,
    theme::palette,
    widget::{
        grid::GridTrack, Align, Container, Grid, GridLength, Null, Padding, Selectable, Text, WidgetWrapper,
    },
    Alignment, Color, Data, Environment, Length, UnitExt, Widget, WidgetExt, Window,
};
use kyute_shell::winit::window::WindowBuilder;
use kyute_text::{FontStyle, FormattedText};

mod grids;

/// A 3-element application scaffolding: a sidebar on the left, a toolbar on the top and the rest is the content area.
#[derive(Clone, WidgetWrapper)]
pub struct Scaffold {
    grid: Grid,
}

impl Scaffold {
    #[composable]
    pub fn new() -> Scaffold {

        let mut grid = Grid::with_template("150 2 1fr / 300 2 1fr");
        // separators
        grid.place("1 / ..", Container::new(Null).background(theme::palette::GREY_800));
        grid.place(".. / 1", Container::new(Null).background(theme::palette::GREY_800));

        Scaffold { grid }
    }

    pub fn sidebar(mut self, sidebar: impl Widget + 'static) -> Self {
        self.set_sidebar(sidebar);
        self
    }

    pub fn set_sidebar(&mut self, sidebar: impl Widget + 'static) {
        self.grid.add_item(0..1, 0, 0, sidebar);
    }

    pub fn toolbar(mut self, toolbar: impl Widget + 'static) -> Self {
        self.set_toolbar(toolbar);
        self
    }

    pub fn set_toolbar(&mut self, toolbar: impl Widget + 'static) {
        self.grid.add_item(0, 1, 0, toolbar);
    }

    pub fn content(mut self, content: impl Widget + 'static) -> Self {
        self.set_content(content);
        self
    }

    pub fn set_content(&mut self, content: impl Widget + 'static) {
        self.grid.add_item(1, 1, 0, content);
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
    let widget = Text::new(name);
    let mut container = Container::new(widget)
        .alignment(Alignment::CENTER_LEFT)
        .content_padding(8.dip(), 8.dip(), 8.dip(), 8.dip())
        .fill();
    container.push_alternate_box_style(
        VisualState::HOVER,
        BoxStyle::new().radius(8.dip()).fill(palette::GREY_100.with_alpha(0.2)),
    );
    if kind == *selected {
        container.set_box_style(BoxStyle::new().radius(8.dip()).fill(palette::BLUE_700.with_alpha(0.8)));
    }
    Selectable::new(selected, kind, container)
}

#[composable]
fn root_view() -> impl Widget + Clone {
    #[state]
    let mut selected = GalleryWidget::Home;

    let mut scaffold = Scaffold::new();

    // widget list
    let mut widget_list = Grid::with_template("auto-flow 35dip / 1fr / 8 0");

    // widgets

    widget_list.insert(
        (
            gallery_sidebar_item("Home", GalleryWidget::Home, &mut selected),
            gallery_sidebar_item("Buttons", GalleryWidget::Buttons, &mut selected),
            gallery_sidebar_item("Formatted Text", GalleryWidget::FormattedText, &mut selected),
            gallery_sidebar_item("Drop down", GalleryWidget::DropDown, &mut selected),
            gallery_sidebar_item("Grids", GalleryWidget::Grids, &mut selected),
            gallery_sidebar_item("Context menu", GalleryWidget::ContextMenu, &mut selected),
            gallery_sidebar_item("Titled panes", GalleryWidget::TitledPanes, &mut selected),
            gallery_sidebar_item("Text input", GalleryWidget::TextInput, &mut selected),
            gallery_sidebar_item("Tree view", GalleryWidget::TreeView, &mut selected),
        ),
    );

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

    scaffold.set_sidebar(widget_list.padding(8.dip(), 8.dip(), 8.dip(), 8.dip()));
    scaffold.set_content(right_panel);

    scaffold.fix_size(100.percent(), 100.percent())
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
    let _app = Application::new();
    let mut env = Environment::new();
    //env.set(kyute::widget::grid::SHOW_GRID_LAYOUT_LINES, true);
    application::run_with_env(main_window, env);
    Application::shutdown();
}
