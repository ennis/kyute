//!

use crate::{
    style::WidgetState, Environment, Event, EventCtx, Geometry, LayoutCtx, LayoutParams, Measurements, PaintCtx, Size,
    Widget, WidgetId,
};
use std::cell::{Cell, RefCell};

pub struct Drawable<F> {
    draw_callback: F,
    size: Size,
    baseline: Option<f64>,
    env: RefCell<Environment>,
    state: Cell<WidgetState>,
}

impl<F> Drawable<F>
where
    F: Fn(&mut PaintCtx, WidgetState, &Environment),
{
    pub fn new(size: Size, baseline: Option<f64>, draw_callback: F) -> Drawable<F> {
        Drawable {
            draw_callback,
            size,
            baseline,
            env: RefCell::new(Default::default()),
            state: Cell::new(Default::default()),
        }
    }
}

impl<F> Widget for Drawable<F>
where
    F: Fn(&mut PaintCtx, WidgetState, &Environment),
{
    fn widget_id(&self) -> Option<WidgetId> {
        None
    }

    fn layout(&self, _ctx: &mut LayoutCtx, _params: &LayoutParams, env: &Environment) -> Geometry {
        self.env.replace(env.clone());
        Geometry {
            measurements: Measurements {
                size: self.size,
                clip_bounds: None,
                baseline: self.baseline,
            },
            ..Default::default()
        }
    }

    fn event(&self, _ctx: &mut EventCtx, _event: &mut Event, _env: &Environment) {}

    fn paint(&self, ctx: &mut PaintCtx) {
        (self.draw_callback)(ctx, self.state.get(), &self.env.borrow())
    }
}
