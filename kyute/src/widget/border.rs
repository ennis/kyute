//! Baseline alignment.
use crate::{drawing, drawing::PaintCtxExt, style, widget::prelude::*, SideOffsets};
use std::cell::{Cell, RefCell};

////////////////////////////////////////////////////////////////////////////////////////////////////
// Widget definition
////////////////////////////////////////////////////////////////////////////////////////////////////

/// Applies a border around a widget.
pub struct Border<Inner> {
    //border_layer: WidgetPod<Null>,
    inner: WidgetPod<Inner>,
    border: style::Border,
    shape: style::Shape,
    /// Computed border widths
    computed_widths: Cell<[f64; 4]>,
    computed_shape: Cell<drawing::Shape>,
}

impl<Inner: Widget + 'static> Border<Inner> {
    // TODO radii should be propagated up during layout
    #[composable]
    pub fn new(border: style::Border, shape: style::Shape, inner: Inner) -> Border<Inner> {
        Border {
            //border_layer: WidgetPod::with_surface(Container::new(Null)),
            inner: WidgetPod::new(inner),
            border,
            shape,
            computed_widths: Cell::new([0.0; 4]),
            computed_shape: Cell::new(drawing::Shape::RoundedRect(drawing::RoundedRect::default())),
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
        let border_top = self.border.widths[0].compute(constraints);
        let border_right = self.border.widths[1].compute(constraints);
        let border_bottom = self.border.widths[2].compute(constraints);
        let border_left = self.border.widths[3].compute(constraints);

        let subconstraints =
            constraints.deflate(SideOffsets::new(border_top, border_right, border_bottom, border_left));
        let sublayout = self.inner.layout(ctx, &subconstraints, env);

        let mut size = sublayout.padding_box_size();
        size.width += border_right + border_left;
        size.height += border_top + border_bottom;
        let baseline = sublayout.padding_box_baseline().map(|x| x + border_top);

        if !ctx.speculative {
            // TODO
            let border_constraints = LayoutConstraints {
                min: sublayout.measurements.size,
                max: sublayout.measurements.size,
                ..*constraints
            };
            //self.border_layer.layout(ctx, &border_constraints, env);
            self.inner.set_offset(Offset::new(
                border_left + sublayout.padding_left,
                border_top + sublayout.padding_top,
            ));
            self.computed_widths
                .set([border_top, border_right, border_bottom, border_left]);

            match self.shape {
                style::Shape::RoundedRect { radii } => {
                    let radius_top_left = radii[0].compute(constraints);
                    let radius_top_right = radii[1].compute(constraints);
                    let radius_bottom_right = radii[2].compute(constraints);
                    let radius_bottom_left = radii[3].compute(constraints);
                    self.computed_shape.set(
                        drawing::RoundedRect {
                            rect: Rect::new(Point::origin(), size),
                            radii: [
                                Offset::new(radius_top_left, radius_top_left),
                                Offset::new(radius_top_right, radius_top_right),
                                Offset::new(radius_bottom_right, radius_bottom_right),
                                Offset::new(radius_bottom_left, radius_bottom_left),
                            ],
                        }
                        .into(),
                    );
                }
            }
        }

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

        let border = drawing::Border {
            widths: self.computed_widths.get(),
            paint: drawing::Paint::Color(self.border.color),
            line_style: self.border.line_style,
            blend_mode: drawing::BlendMode::SrcOver,
        };

        ctx.draw_border(&self.computed_shape.get(), &border);
    }
}
