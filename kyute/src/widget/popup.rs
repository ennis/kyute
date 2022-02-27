use crate::{
    cache,
    event::{PointerButton, PointerEventKind},
    theme,
    widget::{prelude::*, Container, Label},
    Data, SideOffsets, Signal, State, Window,
};
use kyute_shell::winit::window::WindowBuilder;
use std::{convert::TryInto, fmt::Display};
use tracing::trace;

/// Pop-up window with contents.
#[derive(Clone)]
pub struct Popup {
    id: WidgetId,
    shown: cache::Key<bool>,
    window: Option<Window>,
}

impl Popup {
    /// Creates a new popup window.
    #[composable]
    pub fn new(content: impl Widget + 'static) -> Popup {
        let shown = cache::state(|| false);

        let window = if shown.get() {
            Some(Window::new(WindowBuilder::new().with_decorations(false), content, None))
        } else {
            None
        };

        Popup {
            id: WidgetId::here(),
            shown,
            window,
        }
    }

    /// Shows the popup.
    #[composable]
    pub fn show(&self) {
        // will trigger a recomp
        self.shown.set(true);
    }
}

impl Widget for Popup {
    fn widget_id(&self) -> Option<WidgetId> {
        Some(self.id)
    }

    fn event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        if let Some(ref window) = self.window {
            window.event(ctx, event, env);
        }
    }

    fn layout(&self, _ctx: &mut LayoutCtx, _constraints: BoxConstraints, _env: &Environment) -> Measurements {
        Measurements::default()
    }

    fn paint(&self, _ctx: &mut PaintCtx, _bounds: Rect, _env: &Environment) {}
}
