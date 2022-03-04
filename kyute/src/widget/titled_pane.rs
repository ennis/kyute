use crate::{
    composable,
    state::State,
    theme,
    widget::{
        grid::GridTrackDefinition, separator::separator, Clickable, Container, Grid, GridLength, Image, Label,
        SingleChildWidget,
    },
    Alignment, Orientation, SideOffsets, Widget, WidgetExt,
};

/// A widget with a title.
#[derive(Clone)]
pub struct TitledPane {
    inner: Grid,
    collapsed_changed: Option<bool>,
}

impl TitledPane {
    /// Creates a new collapsible pane.
    #[composable]
    pub fn collapsible(
        title: impl Into<String>,
        initially_collapsed: bool,
        content: impl Widget + 'static,
    ) -> TitledPane {
        let collapsed_state = State::new(|| initially_collapsed);
        let pane = Self::new(collapsed_state.get(), title.into(), content);
        collapsed_state.update(pane.collapsed_changed());
        pane
    }

    #[composable]
    fn new(collapsed: bool, title: String, content: impl Widget + 'static) -> TitledPane {
        let mut inner = Grid::column(GridLength::Flex(1.0));

        //use kyute::style::*;

        let icon = Image::from_uri(if collapsed {
            "data/icons/chevron-collapsed.png"
        } else {
            "data/icons/chevron.png"
        });

        // Title bar
        let title_bar = Clickable::new(
            Container::new(
                Grid::with_column_definitions([
                    GridTrackDefinition::new(GridLength::Fixed(20.0)),
                    GridTrackDefinition::new(GridLength::Fixed(3.0)),
                    GridTrackDefinition::new(GridLength::Flex(1.0)),
                    GridTrackDefinition::new(GridLength::Fixed(20.0)),
                ])
                .with_item(0, 0, icon)
                .with_item(0, 2, Label::new(title).aligned(Alignment::CENTER_LEFT)),
            )
            .content_padding(SideOffsets::new_all_same(2.0))
            .box_style(theme::TITLED_PANE_HEADER),
        );

        let collapsed_changed = if title_bar.clicked() { Some(!collapsed) } else { None };

        inner.add_item(inner.row_count(), 0, title_bar);
        inner.add_item(inner.row_count(), 0, separator(Orientation::Horizontal));

        // Add contents if not collapsed
        if !collapsed {
            inner.add_item(inner.row_count(), 0, content);
        }

        TitledPane {
            inner,
            collapsed_changed,
        }
    }

    /// Returns whether the panel has been collapsed or expanded from user input.
    pub fn collapsed_changed(&self) -> Option<bool> {
        self.collapsed_changed
    }
}

impl SingleChildWidget for TitledPane {
    fn child(&self) -> &dyn Widget {
        &self.inner
    }
}
