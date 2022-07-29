use crate::{drawing, drawing::PaintCtxExt, style, widget::prelude::*, LengthOrPercentage, Rect};
use std::{
    cell::{Cell, RefCell},
    convert::TryInto,
};

/// Shape widget.
pub struct Shape {
    shape: style::Shape,
    paint: style::Image,
    computed_shape: Cell<drawing::Shape>,
    computed_paint: RefCell<drawing::Paint>,
}

impl Shape {
    pub fn new(shape: style::Shape, paint: style::Image) -> Shape {
        Shape {
            shape,
            paint,
            computed_shape: Default::default(),
            computed_paint: Default::default(),
        }
    }
}

impl Widget for Shape {
    fn widget_id(&self) -> Option<WidgetId> {
        None
    }

    fn layout(&self, ctx: &mut LayoutCtx, constraints: &LayoutConstraints, env: &Environment) -> Layout {
        // resolve lengths
        // fill the available space
        let size = constraints.max;

        if !ctx.speculative {
            // TODO deduplicate this code, it's the same in border.rs
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

            self.computed_paint.replace(self.paint.compute_paint(env));
        }

        Layout::new(size)
    }

    fn event(&self, _ctx: &mut EventCtx, _event: &mut Event, _env: &Environment) {
        // shouldn't receive events
    }

    fn paint(&self, ctx: &mut PaintCtx) {
        let paint = self.computed_paint.borrow();
        ctx.fill_shape(&self.computed_shape.get(), &*paint);
    }
}
