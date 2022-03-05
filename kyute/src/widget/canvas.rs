use crate::{widget::prelude::*, Dip, Transform};
use std::sync::Arc;

pub struct Canvas {
    id: WidgetId,
    transform: Transform<Dip, Dip>,
    items: Vec<(Offset, Arc<WidgetPod>)>,
}

impl Canvas {
    #[composable]
    pub fn new() -> Canvas {
        Canvas {
            id: WidgetId::here(),
            transform: Transform::identity(),
            items: vec![],
        }
    }

    pub fn set_transform(&mut self, transform: Transform<Dip, Dip>) {
        self.transform = transform;
    }

    pub fn add_item(&mut self, offset: Offset, widget: impl Widget + 'static) {
        self.items.push((offset, Arc::new(WidgetPod::new(widget))));
    }
}

impl Widget for Canvas {
    fn widget_id(&self) -> Option<WidgetId> {
        Some(self.id)
    }

    fn event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        for (_, item) in self.items.iter() {}
    }

    fn layout(&self, ctx: &mut LayoutCtx, constraints: BoxConstraints, env: &Environment) -> Measurements {
        todo!()
    }

    fn paint(&self, ctx: &mut PaintCtx, bounds: Rect, env: &Environment) {
        todo!()
    }
}
