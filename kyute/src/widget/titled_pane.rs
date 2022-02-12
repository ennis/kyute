use crate::{
    composable,
    state::State,
    widget::{Clickable, Container, Flex, SingleChildWidget, Text},
    Color, Orientation, SideOffsets, Widget,
};

/// A widget with a title TODO.
#[derive(Clone)]
pub struct TitledPane {
    inner: Flex,
    collapsed_changed: Option<bool>,
}

impl TitledPane {
    /// Creates a new collapsible pane.
    #[composable(uncached)]
    pub fn collapsible(
        title: impl Into<String>,
        initially_collapsed: bool,
        content: impl Widget + 'static,
    ) -> TitledPane {
        let state = State::new(|| initially_collapsed);
        let pane = Self::new(state.get(), title.into(), content);
        state.update(pane.collapsed_changed);
        pane
    }

    #[composable(uncached)]
    fn new(collapsed: bool, title: String, content: impl Widget + 'static) -> TitledPane {
        let mut inner = Flex::new(Orientation::Vertical);

        use kyute::style::*;

        // Title bar
        let title_bar = Clickable::new(Container::new(
            Flex::horizontal().with(
                Container::new(Text::new(title))
                    .content_padding(SideOffsets::new_all_same(2.0))
                    .box_style(BoxStyle::new().fill(Color::from_hex("#455574"))),
            ),
        ));

        let collapsed_changed = if title_bar.clicked() {
            Some(!collapsed)
        } else {
            None
        };

        inner.push(title_bar);

        // Add contents if not collapsed
        if !collapsed {
            inner.push(content);
        }

        TitledPane {
            inner,
            collapsed_changed,
        }
    }

    /// Returns whether the panel has been collapsed or expanded from user input.
    pub fn collapsed_changed(&mut self) -> Option<bool> {
        self.collapsed_changed
    }
}

impl SingleChildWidget for TitledPane {
    fn child(&self) -> &dyn Widget {
        &self.inner
    }
}
