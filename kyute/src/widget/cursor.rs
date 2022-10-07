use crate::{
    composable,
    style::WidgetState,
    widget::{Clickable, WidgetExt},
    Environment, Event, EventCtx, Geometry, LayoutCtx, LayoutParams, PaintCtx, PointerEventKind, Widget, WidgetId,
};
use keyboard_types::{Key, KeyState, Modifiers};
use kyute_shell::winit;

pub struct CursorIcon<W> {
    id: WidgetId,
    inner: W,
    icon: winit::window::CursorIcon,
}

impl<W: Widget + 'static> CursorIcon<W> {
    #[composable]
    pub fn new(inner: W, icon: winit::window::CursorIcon) -> CursorIcon<W> {
        CursorIcon {
            id: WidgetId::here(),
            inner,
            icon,
        }
    }
}

impl<W: Widget + 'static> Widget for CursorIcon<W> {
    fn widget_id(&self) -> Option<WidgetId> {
        Some(self.id)
    }

    fn layout(&self, ctx: &mut LayoutCtx, params: &LayoutParams, env: &Environment) -> Geometry {
        self.inner.layout(ctx, params, env)
    }

    fn event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        match event {
            Event::Pointer(p) => match p.kind {
                PointerEventKind::PointerOver => ctx.set_cursor_icon(self.icon),
                PointerEventKind::PointerOut => ctx.set_cursor_icon(winit::window::CursorIcon::Default),
                _ => {}
            },
            _ => {}
        }

        self.inner.route_event(ctx, event, env);
    }

    fn paint(&self, ctx: &mut PaintCtx) {
        self.inner.paint(ctx)
    }
}
