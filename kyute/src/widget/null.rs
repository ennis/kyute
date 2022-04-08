use crate::widget::prelude::*;

/// Null widget. Takes no space, ignores all events.
#[derive(Clone, Debug)]
pub struct Null {
    layer: LayerHandle,
}

impl Null {
    #[composable]
    pub fn new() -> Null {
        Null { layer: Layer::new() }
    }
}

impl Widget for Null {
    fn widget_id(&self) -> Option<WidgetId> {
        None
    }

    fn layer(&self) -> &LayerHandle {
        &self.layer
    }

    fn layout(&self, _ctx: &mut LayoutCtx, _constraints: BoxConstraints, _env: &Environment) -> Measurements {
        Measurements::default()
    }

    fn event(&self, _ctx: &mut EventCtx, _event: &mut Event, _env: &Environment) {}
}
