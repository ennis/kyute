use crate::{
    drawing,
    drawing::{BlendMode, BorderStyle, Paint, PaintCtxExt, Shape, ToSkia},
    widget::prelude::*,
    Color, RectExt,
};

/// Behavior-less placeholder for widgets not yet implemented.
#[derive(Copy, Clone, Debug)]
pub struct Placeholder;

impl Widget for Placeholder {
    fn widget_id(&self) -> Option<WidgetId> {
        None
    }

    fn layout(&self, ctx: &mut LayoutCtx, constraints: &LayoutParams, env: &Environment) -> Layout {
        Layout::new(constraints.min)
    }

    fn event(&self, _ctx: &mut EventCtx, _event: &mut Event, _env: &Environment) {}

    fn paint(&self, ctx: &mut PaintCtx) {
        use skia_safe as sk;

        let canvas = ctx.surface.canvas();
        let mut paint = skia_safe::Paint::new(Color::from_hex("#3fb1fc").to_skia(), None);
        paint.set_stroke(false);
        paint.set_alpha(128);
        canvas.draw_rect(ctx.bounds.to_skia(), &paint);

        paint.set_stroke(true);
        paint.set_alpha(255);
        paint.set_stroke_width(1.0);
        canvas.draw_line(
            ctx.bounds.top_left().to_skia(),
            ctx.bounds.bottom_right().to_skia(),
            &paint,
        );
        canvas.draw_line(
            ctx.bounds.top_right().to_skia(),
            ctx.bounds.bottom_left().to_skia(),
            &paint,
        );
        canvas.draw_rect(ctx.bounds.to_skia(), &paint);
    }
}
