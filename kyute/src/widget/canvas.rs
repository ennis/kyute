use crate::{widget::prelude::*, Dip, Transform};
use std::sync::Arc;

#[derive(Clone)]
pub struct Canvas {
    id: WidgetId,
    transform: Transform,
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

    pub fn set_transform(&mut self, transform: Transform) {
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
        for (_, item) in self.items.iter() {
            item.event(ctx, event, env)
        }
    }

    fn layout(&self, ctx: &mut LayoutCtx, constraints: BoxConstraints, env: &Environment) -> Measurements {
        // a canvas always takes the maximum available space
        let width = if constraints.max.width.is_finite() {
            constraints.max.width
        } else {
            0.0
        };

        let height = if constraints.max.height.is_finite() {
            constraints.max.height
        } else {
            0.0
        };

        // place the items in the canvas
        for (offset, item) in self.items.iter() {
            item.layout(ctx, BoxConstraints::new(.., ..), env);
            let transform = offset.to_transform().then(&self.transform);
            item.set_transform(transform);
        }

        //trace!("canvas size: {}x{}", width, height);

        Measurements::new(Rect::new(Point::origin(), Size::new(width, height)))
    }

    fn paint(&self, ctx: &mut PaintCtx, bounds: Rect, transform: Transform, env: &Environment) {
        for (_, item) in self.items.iter() {
            item.paint(ctx, bounds, transform, env)
        }
    }
}
