//! Stateful widgets.

use crate::{
    cache, composable, DebugNode, Environment, Event, EventCtx, Geometry, LayerPaintCtx, LayoutCtx, LayoutParams,
    PaintCtx, Widget, WidgetId,
};
use kyute_common::Transform;
use kyute_shell::animation::Layer;
use parking_lot::Mutex;
use std::{cell::RefCell, sync::Arc};

/// Widgets whose internal state is kept across recompositions.
pub trait RetainedWidget {
    /// The type of the arguments passed to the constructor and update function.
    type Args;

    /// Creates a new instance of the widget state.
    fn new(args: &Self::Args) -> Self;

    /// Updates the state with the given arguments.
    fn update(&mut self, args: &Self::Args);

    // ------ Widget interface ------

    fn widget_id(&self) -> Option<WidgetId>;

    fn speculative_layout(&mut self, ctx: &mut LayoutCtx, params: &LayoutParams, env: &Environment) -> Geometry {
        let was_speculative = ctx.speculative;
        ctx.speculative = true;
        let layout = self.layout(ctx, params, env);
        ctx.speculative = was_speculative;
        layout
    }

    ///
    fn layout(&mut self, ctx: &mut LayoutCtx, params: &LayoutParams, env: &Environment) -> Geometry;

    ///
    fn event(&mut self, ctx: &mut EventCtx, event: &mut Event, env: &Environment);

    ///
    fn paint(&mut self, ctx: &mut PaintCtx);

    ///
    fn layer_paint(&mut self, ctx: &mut LayerPaintCtx, layer: &Layer, scale_factor: f64) {
        ctx.paint_layer(layer, scale_factor, |ctx| self.paint(ctx))
    }

    ///
    fn debug_name(&mut self) -> &str {
        std::any::type_name::<Self>()
    }

    ///
    fn debug_node(&mut self) -> DebugNode {
        DebugNode { content: None }
    }
}

pub struct Retained<W> {
    widget: Arc<Mutex<W>>,
}

impl<W: RetainedWidget> Widget for Retained<W> {
    fn widget_id(&self) -> Option<WidgetId> {
        self.widget.lock().widget_id()
    }

    fn layout(&self, ctx: &mut LayoutCtx, params: &LayoutParams, env: &Environment) -> Geometry {
        self.widget.lock().layout(ctx, params, env)
    }

    fn event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        self.widget.lock().event(ctx, event, env)
    }

    fn paint(&self, ctx: &mut PaintCtx) {
        self.widget.lock().paint(ctx)
    }

    fn layer_paint(&self, ctx: &mut LayerPaintCtx, layer: &Layer, scale_factor: f64) {
        self.widget.lock().layer_paint(ctx, layer, scale_factor)
    }
}

impl<W: RetainedWidget + 'static> Retained<W> {
    #[composable]
    pub fn new(args: &W::Args) -> Retained<W> {
        let mut created = false;
        let w = cache::state(|| {
            created = true;
            Arc::new(Mutex::new(W::new(args)))
        })
        .get();

        if !created {
            w.lock().update(args);
        }

        Retained { widget: w }
    }
}
