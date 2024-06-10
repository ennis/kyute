//! Drawing & interaction elements for immediate mode widgets.

use crate::{
    drawing::ToSkia,
    widgets::immediate::{
        api::{rect, Circle},
        geometry::Rect,
        ImCtx, VarId, IMCTX,
    },
    WidgetPtr,
};

pub enum DrawCommand {
    FillRect { rect: Rect, paint: skia_safe::Paint },
    FillCircle { circle: Circle, paint: skia_safe::Paint },
}

impl DrawCommand {
    pub fn resolve(&self) -> ResolvedDrawCommand {
        match self {
            DrawCommand::FillRect { rect, paint } => ResolvedDrawCommand::FillRect {
                rect: rect.resolve(),
                paint: paint.clone(),
            },
            DrawCommand::FillCircle { circle, paint } => ResolvedDrawCommand::FillCircle {
                circle: circle.resolve(),
                paint: paint.clone(),
            },
        }
    }
}

pub enum ResolvedDrawCommand {
    FillRect {
        rect: kurbo::Rect,
        paint: skia_safe::Paint,
    },
    FillCircle {
        circle: kurbo::Circle,
        paint: skia_safe::Paint,
    },
}

impl ResolvedDrawCommand {
    pub fn draw(&self, canvas: &mut skia_safe::Canvas) {
        match self {
            ResolvedDrawCommand::FillRect { rect, paint } => {
                let rect = rect.to_skia();
                canvas.draw_rect(rect, paint);
            }
            ResolvedDrawCommand::FillCircle { circle, paint } => {
                canvas.draw_circle(circle.center.to_skia(), circle.radius as f32, paint);
            }
        }
    }
}

pub fn fill_rect(rect: Rect, paint: &skia_safe::Paint) {
    IMCTX.with(|imctx| {
        imctx.add_draw_command(DrawCommand::FillRect {
            rect,
            paint: paint.clone(),
        });
    });
}

pub fn fill_circle(circle: Circle, paint: &skia_safe::Paint) {
    IMCTX.with(|imctx| {
        imctx.add_draw_command(DrawCommand::FillCircle {
            circle,
            paint: paint.clone(),
        });
    });
}
