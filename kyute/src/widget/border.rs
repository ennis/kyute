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

    fn layout(&self, ctx: &mut LayoutCtx, constraints: &LayoutConstraints, env: &Environment) -> Layout {
        let border_top = constraints.resolve_height(self.border.widths[0]);
        let border_right = constraints.resolve_width(self.border.widths[1]);
        let border_bottom = constraints.resolve_height(self.border.widths[2]);
        let border_left = constraints.resolve_width(self.border.widths[3]);

        let subconstraints =
            constraints.deflate(SideOffsets::new(border_top, border_right, border_bottom, border_left));
        let sublayout = self.inner.layout(ctx, &subconstraints, env);

        if !ctx.speculative {
            // TODO
            let border_constraints = LayoutConstraints {
                min: sublayout.measurements.size,
                max: sublayout.measurements.size,
                ..*constraints
            };
            self.border_layer.layout(ctx, &border_constraints, env);
            self.inner.set_offset(Offset::new(
                border_left + sublayout.padding_left,
                border_top + sublayout.padding_top,
            ));
        }

        let mut size = sublayout.padding_box_size();
        size.width += border_right + border_left;
        size.height += border_top + border_bottom;
        let baseline = sublayout.padding_box_baseline().map(|x| x + border_top);

        Layout {
            padding_left: 0.0,
            padding_top: 0.0,
            padding_right: 0.0,
            padding_bottom: 0.0,
            measurements: Measurements {
                size,
                clip_bounds: None,
                baseline,
            },
            ..sublayout
        }
    }

    fn event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        self.inner.route_event(ctx, event, env);
    }

    fn paint(&self, ctx: &mut PaintCtx) {
        self.inner.paint(ctx);
        //self.border_layer.paint(ctx);
    }
}
