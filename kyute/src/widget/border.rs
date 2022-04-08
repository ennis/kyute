//! Baseline alignment.
use crate::{style, style::BorderPosition, widget::prelude::*, RoundToPixel, SideOffsets};

////////////////////////////////////////////////////////////////////////////////////////////////////
// Layer delegate
////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct BorderLayerDelegate {
    border: style::Border,
}

impl LayerDelegate for BorderLayerDelegate {
    fn draw(&self, ctx: &mut PaintCtx) {
        use skia_safe as sk;
        // TODO support non zero radius
        let radius = sk::Vector::new(0.0, 0.0);
        self.border.draw(ctx, ctx.bounds, [radius, radius, radius, radius]);
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Widget definition
////////////////////////////////////////////////////////////////////////////////////////////////////

/// Applies a border around a widget.
#[derive(Clone)]
pub struct Border<Inner> {
    layer: LayerHandle,
    border_layer: LayerHandle,
    inner: Inner,
    border: style::Border,
}

impl<Inner: Widget + 'static> Border<Inner> {
    #[composable]
    pub fn new(border: style::Border, inner: Inner) -> Border<Inner> {
        Border {
            layer: Layer::new(),
            border_layer: Layer::new(),
            inner,
            border,
        }
    }

    /// Returns a reference to the inner widget.
    pub fn inner(&self) -> &Inner {
        &self.inner
    }

    /// Returns a mutable reference to the inner widget.
    pub fn inner_mut(&mut self) -> &mut Inner {
        &mut self.inner
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// impl Widget
////////////////////////////////////////////////////////////////////////////////////////////////////

impl<Inner: Widget> Widget for Border<Inner> {
    fn widget_id(&self) -> Option<WidgetId> {
        self.inner.widget_id()
    }

    fn layer(&self) -> &LayerHandle {
        &self.layer
    }

    fn layout(&self, ctx: &mut LayoutCtx, constraints: BoxConstraints, env: &Environment) -> Measurements {
        let nat_width = constraints.finite_max_width().unwrap_or(0.0);
        let nat_height = constraints.finite_max_height().unwrap_or(0.0);
        let border_offsets = self
            .border
            .side_offsets(ctx.scale_factor, Size::new(nat_width, nat_height));
        let constraints = constraints.deflate(border_offsets);
        let mut m = self.inner.layout(ctx, constraints, env);
        m.size.width += border_offsets.horizontal();
        m.size.height += border_offsets.vertical();
        m.clip_bounds = m.clip_bounds.union(&m.local_bounds().outer_rect(border_offsets));
        m.baseline.map(|b| b + border_offsets.top);

        // update layers
        let child_layer = self.inner.layer();
        child_layer.set_offset(Offset::new(border_offsets.left, border_offsets.top));
        self.layer.set_size(m.size);
        self.border_layer.set_size(m.size);
        self.layer.add_child(child_layer);
        self.layer.add_child(&self.border_layer);

        m
    }

    fn event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        self.inner.route_event(ctx, event, env);
    }
}
