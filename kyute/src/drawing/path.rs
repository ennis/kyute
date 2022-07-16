use crate::{
    drawing::{svg_path_to_skia, BoxShadow, Paint, ToSkia},
    PaintCtx, Rect, RectExt,
};
use skia_safe as sk;

/// Path visual.
pub struct Path {
    path: sk::Path,
    stroke: Option<Paint>,
    fill: Option<Paint>,
    box_shadow: Option<BoxShadow>,
}

impl Path {
    pub fn new(svg_path: &str) -> Path {
        Path {
            path: svg_path_to_skia(svg_path).expect("invalid path syntax"),
            stroke: None,
            fill: None,
            box_shadow: None,
        }
    }

    /// Sets the brush used to fill the path.
    pub fn fill(mut self, paint: impl Into<Paint>) -> Self {
        self.fill = Some(paint.into());
        self
    }

    /// Sets the brush used to stroke the path.
    pub fn stroke(mut self, paint: impl Into<Paint>) -> Self {
        self.fill = Some(paint.into());
        self
    }

    pub fn draw(&self, ctx: &mut PaintCtx, bounds: Rect) {
        // fill
        let canvas = ctx.surface.canvas();
        if let Some(ref brush) = self.fill {
            let mut paint = brush.to_sk_paint(bounds);
            paint.set_style(sk::PaintStyle::Fill);
            canvas.save();
            canvas.translate(bounds.top_left().to_skia());
            canvas.draw_path(&self.path, &paint);
            canvas.restore();
        }

        // stroke
        if let Some(ref stroke) = self.stroke {
            let mut paint = stroke.to_sk_paint(bounds);
            paint.set_style(sk::PaintStyle::Stroke);
            canvas.save();
            canvas.translate(bounds.top_left().to_skia());
            canvas.draw_path(&self.path, &paint);
            canvas.restore();
        }
    }
}
