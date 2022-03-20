//! Widget gallery
use crate::GalleryWidget::ContextMenu;
use kyute::{
    application, composable,
    shell::application::Application,
    style::{BoxStyle, LinearGradient, VisualState},
    theme,
    theme::palette,
    widget::{Align, Container, Grid, GridLength, Null, Padding, Selectable, Text},
    Alignment, Color, Data, Environment, Length, UnitExt, Widget, WidgetExt, Window,
};
use kyute_shell::winit::window::WindowBuilder;
use kyute_text::{FontStyle, FormattedText};

mod grids;

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

    // v-split
    let mut g = Grid::with_rows_columns(
        [GridLength::Flex(1.0).into()],
        [
            GridLength::Fixed(300.dip()).into(),
            GridLength::Fixed(3.px()).into(),
            GridLength::Flex(1.0).into(),
        ],
    );

    g.add_item(
        0,
        0,
        Container::new(Null).fill().box_style(
            BoxStyle::new().fill(
                LinearGradient::new()
                    .stop(Color::from_hex("#21020E"), 1.0)
                    .stop(Color::from_hex("#140900"), 0.0)
                    .angle(90.degrees()),
            ),
        ),
    );

    // widget list
    let mut widget_list = Grid::column(GridLength::Flex(1.0))
        .row_template(GridLength::Fixed(35.dip()))
        .row_gap(8.dip());

    // widgets

    // issue: since most env keys are resolved outside of widget composition, there's no way to change it without recomp.
    // also, changing it during recomp means that we have to do `cache::with_environment(|| ...)`, which is annoying

    widget_list.add_row(gallery_sidebar_item("Home", GalleryWidget::Home, &mut selected));
    widget_list.add_row(gallery_sidebar_item(
        "Formatted Text",
        GalleryWidget::FormattedText,
        &mut selected,
    ));
    widget_list.add_row(gallery_sidebar_item(
        "Drop down",
        GalleryWidget::DropDown,
        &mut selected,
    ));
    widget_list.add_row(gallery_sidebar_item("Buttons", GalleryWidget::Buttons, &mut selected));
    widget_list.add_row(gallery_sidebar_item("Grids", GalleryWidget::Grids, &mut selected));
    widget_list.add_row(gallery_sidebar_item(
        "Context menu",
        GalleryWidget::ContextMenu,
        &mut selected,
    ));
    widget_list.add_row(gallery_sidebar_item(
        "Titled panes",
        GalleryWidget::TitledPanes,
        &mut selected,
    ));
    widget_list.add_row(gallery_sidebar_item(
        "Text input",
        GalleryWidget::TextInput,
        &mut selected,
    ));
    widget_list.add_row(gallery_sidebar_item(
        "Tree view",
        GalleryWidget::TreeView,
        &mut selected,
    ));

    g.add_item(0, 0, widget_list.padding(8.dip(), 8.dip(), 8.dip(), 8.dip()));

    // separator
    g.add_item(
        0,
        1,
        Container::new(Null)
            .fill()
            .box_style(BoxStyle::new().fill(theme::palette::GREY_400)),
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

    g.add_item(0, 2, right_panel);

    g.fix_size(100.percent(), 100.percent())
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
