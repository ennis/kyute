use crate::{
    cache, composable,
    state::{Signal, State},
    style::Rectangle,
    widget::{Clickable, Container, Flex, Text},
    BoxConstraints, Environment, Event, Key, Measurements, Orientation, PaintCtx, Rect,
    SideOffsets, Widget, WidgetPod,
};
use kyute::{EventCtx, LayoutCtx};
use kyute_shell::{drawing::Color, skia::gradient_shader::GradientShaderColors::Colors};

/// A widget with a title TODO.
#[derive(Clone)]
pub struct TitledPane {
    inner: WidgetPod<Flex>,
    collapsed: bool,
    collapsed_changed: Option<bool>,
}

impl TitledPane {
    /// Creates a new collapsible pane.
    #[composable(uncached)]
    pub fn collapsible(
        title: impl Into<String>,
        initially_collapsed: bool,
        content: impl Widget+'static,
    ) -> TitledPane {
        let state = State::new(|| initially_collapsed);
        let pane = Self::new(state.get(), title.into(), content);
        state.update(pane.collapsed_changed);
        pane
    }

    #[composable(uncached)]
    fn new(collapsed: bool, title: String, content: impl Widget+'static) -> TitledPane {
        let mut inner = Flex::new(Orientation::Vertical);

        use kyute::style::*;

        // Title bar
        let title_bar = Clickable::new(
            Container::new(Text::new(title))
                .content_padding(SideOffsets::new_all_same(2.0))
                .visual(Rectangle::new().fill(Color::from_hex("#455574"))),
        );

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
            inner: WidgetPod::new(inner),
            collapsed,
            collapsed_changed,
        }
    }

    /// Returns whether the panel has been collapsed or expanded from user input.
    pub fn collapsed_changed(&mut self) -> Option<bool> { self.collapsed_changed }
}

impl Widget for TitledPane {
    fn event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        self.inner.event(ctx, event, env);
    }

    fn layout(
        &self,
        ctx: &mut LayoutCtx,
        constraints: BoxConstraints,
        env: &Environment,
    ) -> Measurements {
        self.inner.layout(ctx, constraints, env)
    }

    fn paint(&self, ctx: &mut PaintCtx, bounds: Rect, env: &Environment) {
        self.inner.paint(ctx, bounds, env);
    }
}
