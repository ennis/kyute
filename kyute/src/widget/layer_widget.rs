use crate::{
    application::AppCtx,
    cache,
    core::{DebugNode, EventResult, FocusState, PaintDamage, WindowPaintCtx},
    widget::{prelude::*, WidgetWrapper},
    GpuFrameCtx, InternalEvent,
};
use kyute_common::SizeI;
use kyute_shell::animation::Layer;
use skia_safe as sk;
use std::cell::Cell;

/// A widget that draws its contents on a separate composition layer.
#[derive(Clone)]
pub struct LayerWidget<W> {
    id: WidgetId,
    layer: Layer,
    measurements: LayoutCache<Measurements>,
    /// Damage done to the contents of the layer.
    paint_damage: Cell<PaintDamage>,
    contents: W,
}

impl<W: Widget> LayerWidget<W> {
    /// Creates a new LayerWidget.
    #[composable]
    pub fn new(contents: W) -> LayerWidget<W> {
        let layer = cache::once(Layer::new);
        LayerWidget {
            id: WidgetId::here(),
            layer,
            measurements: Default::default(),
            paint_damage: Cell::new(PaintDamage::Repaint),
            contents,
        }
    }

    /// Returns the layer.
    pub fn layer(&self) -> &Layer {
        &self.layer
    }

    pub(crate) fn repaint(&self, skia_direct_context: sk::gpu::DirectContext) -> bool {
        assert!(self.measurements.is_valid(), "repaint called before layout");
        let repainted = if let Some(paint_damage) = self.paint_damage.get() {
            match paint_damage {
                PaintDamage::Repaint => {
                    // straight recursive repaint
                    let _span = trace_span!("Layer repaint", id=?self.id).entered();

                    self.layer.remove_all_children();
                    let scale_factor = self.measurements.get_cached_scale_factor();
                    let mut ctx = PaintCtx::new(&self.layer, scale_factor, skia_direct_context);
                    ctx.surface.canvas().clear(sk::Color4f::new(0.0, 0.0, 0.0, 0.0));
                    self.contents.paint(&mut ctx);
                    ctx.finish();
                }
                PaintDamage::SubLayers => {
                    let _span = trace_span!("Layer update", id=?self.id).entered();
                    self.update_child_layers(skia_direct_context);
                }
            }
            true
        } else {
            false
        };
        self.paint_damage.set(PaintDamage::Repaint);
        repainted
    }
}

impl<W: Widget> Widget for LayerWidget<W> {
    fn widget_id(&self) -> Option<WidgetId> {
        Some(self.id)
    }

    fn layout(&self, ctx: &mut LayoutCtx, constraints: BoxConstraints, env: &Environment) -> Measurements {
        self.measurements.update(ctx, constraints, |ctx| {
            let m = self.contents.layout(ctx, constraints, env);
            if !ctx.speculative {
                let size = SizeI::new(
                    (m.clip_bounds.size.width * ctx.scale_factor) as i32,
                    (m.clip_bounds.size.height * ctx.scale_factor) as i32,
                );
                if !size.is_empty() {
                    self.layer.set_size(size);
                }
            }

            // FIXME: here we need to differentiate between two cases:
            // 1. we recalculated because the cached value has been invalidated because a child requested a relayout during eval
            // 2. we recalculated because constraints have changed
            //
            // If 2., then we can skip repaint if the resulting measurements are the same.

            // TODO technically we can call layout and end up with the exact same measurements
            // so a repaint may not be always necessary.
            self.paint_damage.set(PaintDamage::Repaint);
            m
        })
    }

    fn event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        // handle explicit layer update requests
        if let Event::Internal(InternalEvent::UpdateLayers { skia_direct_context }) = event {
            self.repaint(skia_direct_context.clone());
        } else {
            self.contents.route_event(ctx, event, env)
        }
    }

    fn paint(&self, ctx: &mut PaintCtx) {}

    fn route_event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        let event_result = ctx.default_route_event(
            self,
            event,
            &Transform::identity(),
            if self.measurements.is_valid() {
                Some(self.measurements.get_cached().clone())
            } else {
                None
            },
            env,
        );
        if let Some(mut event_result) = event_result {
            if event_result.relayout {
                self.measurements.invalidate();
            }
            match (self.paint_damage.get(), event_result.paint_damage) {
                (None, _) => self.paint_damage.set(event_result.paint_damage),
                (Some(PaintDamage::SubLayers), Some(PaintDamage::Repaint)) => {
                    self.paint_damage.set(event_result.paint_damage)
                }
                _ => {}
            }
            if event_result.paint_damage == PaintDamage::Repaint {
                // downgrade `Repaint` to `SubLayers`: if the contents of a layer need to be redrawn,
                // its parent doesn't necessarily need to.
                event_result.paint_damage = PaintDamage::SubLayers;
            }
            ctx.merge_event_result(event_result);
        }
    }

    /// Implement to give a debug name to your widget. Used only for debugging.
    fn debug_node(&self) -> DebugNode {
        DebugNode {
            content: Some(format!("{:?} px", self.layer.size())),
        }
    }
}
