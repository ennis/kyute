//! Text elements
use crate::{
    composable,
    env::Environment,
    event::Event,
    theme,
    widget::{prelude::*, Text},
    Color, Data, ValueRef,
};
use kyute_text::FormattedText;

/// Simple text label.
#[derive(Clone)]
pub struct Label {
    text: Text,
    color: ValueRef<Color>,
}

impl Label {
    /// Creates a new text label.
    #[composable(cached)]
    pub fn new(text: impl Into<String> + Data) -> Label {
        let text = text.into();
        Label {
            text: Text::new(FormattedText::new(text.into())),
            color: theme::keys::LABEL_COLOR.into(),
        }
    }

    /// Sets the color of the label.
    pub fn color(mut self, color: Color) -> Self {
        self.set_color(color);
        self
    }

    /// Sets the color of the label.
    pub fn set_color(&mut self, color: Color) {
        self.color = color.into();
    }
}

impl Widget for Label {
    fn widget_id(&self) -> Option<WidgetId> {
        self.text.widget_id()
    }

    fn layer(&self) -> &LayerHandle {
        self.text.layer()
    }

    fn layout(&self, ctx: &mut LayoutCtx, constraints: BoxConstraints, env: &Environment) -> Measurements {
        self.text.layout(ctx, constraints, env)
    }

    fn event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        self.text.route_event(ctx, event, env)
    }
}
