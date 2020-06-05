use crate::event::Event;
use crate::{
    Bounds, BoxConstraints, BoxedWidget, Environment, EventCtx, LayoutCtx, Measurements, PaintCtx,
    Point, TypedWidget, Visual, Widget,
};
use kyute_shell::drawing::{Color, IntoBrush, RectExt};
use std::any::Any;

pub enum WidgetType {
    Button,
    TextEdit,
    Slider,
}

/// A widget that draws a theme-specific frame (a box with borders).
pub struct Frame {
    pub border_color: Color,
    pub border_width: f64,
    pub fill_color: Color,
    pub inner: BoxedWidget,
}

impl TypedWidget for Frame {
    type Visual = FrameVisual;

    fn layout(
        self,
        context: &mut LayoutCtx,
        _previous_visual: Option<Box<Self::Visual>>,
        constraints: &BoxConstraints,
        env: Environment,
    ) -> (Box<Self::Visual>, Measurements) {
        let Frame {
            border_color,
            border_width,
            fill_color,
            inner,
        } = self;

        let visual = Box::new(FrameVisual {
            border_color,
            border_width,
            fill_color,
        });

        let (child_id, child_measurements) = context.emit_child(inner, constraints, env);
        (visual, child_measurements)
    }
}

pub struct FrameVisual {
    pub border_color: Color,
    pub border_width: f64,
    pub fill_color: Color,
}

impl Default for FrameVisual {
    fn default() -> Self {
        FrameVisual {
            border_color: Color::new(0.0, 0.0, 0.0, 1.0),
            border_width: 0.0,
            fill_color: Color::new(1.0, 1.0, 1.0, 1.0),
        }
    }
}

impl Visual for FrameVisual {
    fn paint(&mut self, ctx: &mut PaintCtx, env: &Environment) {
        let rect = ctx.bounds();
        let bg_brush = self.fill_color.into_brush(ctx);
        let border_brush = self.border_color.into_brush(ctx);
        // box background
        ctx.fill_rectangle(rect.stroke_inset(self.border_width), &bg_brush);
        // border
        if ctx.is_hovering() || ctx.is_focused() {
            ctx.draw_rectangle(
                rect.stroke_inset(self.border_width),
                &border_brush,
                self.border_width,
            );
        }
    }

    fn hit_test(&mut self, point: Point, bounds: Bounds) -> bool {
        unimplemented!()
    }
    fn event(&mut self, ctx: &mut EventCtx, event: &Event) {
        match event {
            Event::PointerOver(_) | Event::PointerOut(_) => ctx.request_redraw(),
            _ => {}
        }
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
