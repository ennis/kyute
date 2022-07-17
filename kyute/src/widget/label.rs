//! Text elements
use crate::{
    composable,
    env::Environment,
    event::Event,
    theme,
    widget::{prelude::*, Text},
    Color, Data, EnvRef,
};
use kyute_shell::text::FormattedText;

/// Simple text label.
pub struct Label {
    text: Text,
    color: EnvRef<Color>,
}

impl Label {
    /// Creates a new text label.
    #[composable]
    pub fn new(text: impl Into<String> + Data) -> Label {
        let text = text.into();
        Label {
            text: Text::new(FormattedText::new(text.into())),
            color: theme::keys::LABEL_COLOR.into(),
        }
    }
}

impl Widget for Label {
    fn widget_id(&self) -> Option<WidgetId> {
        self.text.widget_id()
    }

    fn layout(&self, ctx: &mut LayoutCtx, constraints: &LayoutConstraints, env: &Environment) -> Layout {
        self.text.layout(ctx, &constraints, env)
    }

    fn event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        self.text.route_event(ctx, event, env)
    }

    fn paint(&self, ctx: &mut PaintCtx) {
        self.text.paint(ctx)
    }
}
