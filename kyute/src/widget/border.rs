//! Baseline alignment.
use crate::{
    style,
    style::BorderPosition,
    widget::{prelude::*, Container, Null},
    RoundToPixel, SideOffsets,
};

////////////////////////////////////////////////////////////////////////////////////////////////////
// Widget definition
////////////////////////////////////////////////////////////////////////////////////////////////////

/// Applies a border around a widget.
pub struct Border<Inner> {
    border_layer: WidgetPod<Container<Null>>,
    inner: WidgetPod<Inner>,
    border: style::Border,
}

impl<Inner: Widget + 'static> Border<Inner> {
    #[composable]
    pub fn new(border: style::Border, inner: Inner) -> Border<Inner> {
        Border {
            border_layer: WidgetPod::with_surface(Container::new(Null)),
            inner: WidgetPod::new(inner),
            border,
        }
    }

    /// Returns a reference to the inner widget.
    pub fn inner(&self) -> &Inner {
        self.inner.inner()
    }

    /// Returns a mutable reference to the inner widget.
    pub fn inner_mut(&mut self) -> &mut Inner {
        self.inner.inner_mut()
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// impl Widget
////////////////////////////////////////////////////////////////////////////////////////////////////

impl<Inner: Widget> Widget for Border<Inner> {
    fn widget_id(&self) -> Option<WidgetId> {
        self.inner.widget_id()
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
        // TODO clip bounds
        //m.clip_bounds = m.clip_bounds.union(&m.local_bounds().outer_rect(border_offsets));
        m.baseline.map(|b| b + border_offsets.top);

        // update layers
        if !ctx.speculative {
            self.border_layer.layout(ctx, BoxConstraints::tight(m.size), env);
            self.inner
                .set_offset(Offset::new(border_offsets.left, border_offsets.top));
        }
        m
    }

    fn event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        self.inner.route_event(ctx, event, env);
    }

    fn paint(&self, ctx: &mut PaintCtx) {
        self.inner.paint(ctx);
        self.border_layer.paint(ctx);
    }
}
