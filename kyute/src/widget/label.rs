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
    pub fn new(text: impl Into<String>) -> Label {
        let text = text.into();
        Label {
            text: Text::new(text),
            color: EnvRef::Inline(Color::new(1.0, 1.0, 1.0, 1.0)),
        }
    }
}

impl Widget for Label {
    fn widget_id(&self) -> Option<WidgetId> {
        self.text.widget_id()
    }

    fn layout(&self, ctx: &mut LayoutCtx, constraints: &LayoutParams, env: &Environment) -> Geometry {
        self.text.layout(ctx, &constraints, env)
    }

    fn event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        self.text.route_event(ctx, event, env)
    }

    fn paint(&self, ctx: &mut PaintCtx) {
        self.text.paint(ctx)
    }
}
