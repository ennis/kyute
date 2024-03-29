use crate::{
    composable,
    widget::{grid, Clickable, Grid, Image, Null, Scaling, Text, WidgetExt},
    Alignment, UnitExt, Widget,
};

/// A widget with a title.
#[derive(Widget)]
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
        let mut inner = Grid::column(grid::TrackBreadth::Flex(1.0));

        //use kyute::style::*;

        let icon = Image::from_uri(
            if collapsed {
                "data/icons/chevron-collapsed.png"
            } else {
                "data/icons/chevron.png"
            },
            Scaling::Contain,
        )
        .min_width(20.dip())
        .min_height(20.dip());

        // Title bar

        let title_bar = {
            let mut grid = Grid::with_template("auto / 20 3 1fr 20");
            grid.insert((
                icon,
                Null,
                Text::new(title)
                    .vertical_alignment(Alignment::CENTER)
                    .horizontal_alignment(Alignment::START),
            ));
            Clickable::new(
                grid.padding(2.dip()), //.box_style(theme::TITLED_PANE_HEADER.get(&cache::environment()).unwrap()),
            )
        };

        let collapsed_changed = if title_bar.clicked() { Some(!collapsed) } else { None };

        inner.insert(title_bar);
        //inner.insert(separator(Orientation::Horizontal));

        // Add contents if not collapsed
        if !collapsed {
            inner.insert(content);
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
