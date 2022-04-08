use crate::{cache, widget::prelude::*, Window};
use kyute_shell::winit::window::WindowBuilder;

/// Pop-up window with contents.
#[derive(Clone)]
pub struct Popup {
    id: WidgetId,
    layer: LayerHandle,
    shown: cache::State<bool>,
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
            layer: Layer::new(),
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

    fn layer(&self) -> &LayerHandle {
        &self.layer
    }

    fn layout(&self, _ctx: &mut LayoutCtx, _constraints: BoxConstraints, _env: &Environment) -> Measurements {
        Measurements::default()
    }

    fn event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        if let Some(ref window) = self.window {
            window.route_event(ctx, event, env);
        }
    }
}
