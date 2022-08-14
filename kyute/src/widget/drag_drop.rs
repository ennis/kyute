//! Drag & drop widgets.

use crate::{shell::TypedData, widget::prelude::*};

pub struct DropTarget<Content> {
    id: WidgetId,
    content: Content,
}

impl<Content> DropTarget<Content> {
    #[composable]
    pub fn new(content: Content) -> DropTarget<Content> {
        DropTarget {
            id: WidgetId::here(),
            content,
        }
    }

    pub fn on_drop(self, f: impl FnOnce(&TypedData)) -> Self {
        // TODO
        self
    }
}

impl<Content: Widget + 'static> Widget for DropTarget<Content> {
    fn widget_id(&self) -> Option<WidgetId> {
        Some(self.id)
    }

    fn layout(&self, ctx: &mut LayoutCtx, params: &LayoutParams, env: &Environment) -> Geometry {
        self.content.layout(ctx, params, env)
    }

    fn event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        // when a file or anything else is dragged onto the window, the window receives a native event
        // containing a pointer to the payload.
        //

        todo!()
    }

    fn paint(&self, ctx: &mut PaintCtx) {
        self.content.paint(ctx)
    }
}
