use crate::{
    composable, theme,
    widget::{
        grid::GridTrack, separator::separator, Clickable, Container, Grid, GridLength, Image, Scaling, Text,
        WidgetWrapper,
    },
    Alignment, Orientation, Widget, WidgetExt,
};
use kyute_common::UnitExt;

/// A widget with a title.
#[derive(WidgetWrapper)]
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
        #[state]
        let mut collapsed_state = initially_collapsed;
        Self::new(collapsed_state, title.into(), content).on_collapsed_changed(|v| collapsed_state = v)
    }

    #[composable]
    fn new(collapsed: bool, title: String, content: impl Widget + 'static) -> TitledPane {
        let mut inner = Grid::column(GridTrack::new(GridLength::Flex(1.0)));

        //use kyute::style::*;

        let icon = Image::from_uri(
            if collapsed {
                "data/icons/chevron-collapsed.png"
            } else {
                "data/icons/chevron.png"
            },
            Scaling::Contain,
        )
        .fix_size(20.dip(), 20.dip());

        // Title bar

        let title_bar = {
            let mut grid = Grid::new();
            grid.append_column_definitions([
                GridTrack::new(GridLength::Fixed(20.dip())),
                GridTrack::new(GridLength::Fixed(3.dip())),
                GridTrack::new(GridLength::Flex(1.0)),
                GridTrack::new(GridLength::Fixed(20.dip())),
            ]);
            grid.add_item(0, 0, 0, icon);
            grid.add_item(0, 2, 0, Text::new(title).aligned(Alignment::CENTER_LEFT));
            Clickable::new(
                Container::new(grid)
                    .content_padding(2.dip(), 2.dip(), 2.dip(), 2.dip())
                    .box_style(theme::TITLED_PANE_HEADER),
            )
        };

        let collapsed_changed = if title_bar.clicked() { Some(!collapsed) } else { None };

        inner.add_item(inner.row_count(), 0, 0, title_bar);
        inner.add_item(inner.row_count(), 0, 0, separator(Orientation::Horizontal));

        // Add contents if not collapsed
        if !collapsed {
            inner.add_item(inner.row_count(), 0, 0, content);
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

    pub fn on_collapsed_changed(self, f: impl FnOnce(bool)) -> Self {
        self.collapsed_changed.map(f);
        self
    }
}
