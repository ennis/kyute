use crate::{
    drawing,
    drawing::{BoxShadow, Paint, ToSkia},
    skia,
    widget::prelude::*,
    Color,
};
use kurbo::{Arc, BezPath, Circle, CircleSegment, Ellipse, Insets, PathEl, RoundedRect, Shape};
use skia_safe::utils::shadow_utils::draw_shadow;
use smallvec::SmallVec;
use std::any::Any;
use tracing::warn;

////////////////////////////////////////////////////////////////////////////////////////////////////

/*
/// Shape widget.
pub struct Shape<S, Ops = NullOp<S>> {
    shape: S,
    ops: Ops,
}

impl<S, Ops> Shape<S, Ops> {
    pub fn border(self, width: f64, paint: impl Into<Paint>) -> Shape<S, BorderOp<Ops>> {
        Shape {
            shape: self.shape,
            ops: BorderOp {
                width,
                paint: paint.into(),
            },
        }
    }

    pub fn drop_shadow(self) -> Shape<S> {
        Shape {
            shape: self.shape,
            ops: OpThen {
                first: self.ops,
                second: DropShadowOp,
            },
        }
    }
}

impl<S> Shape<S> {
    pub fn new(shape: S) -> Shape<S> {
        Shape {
            shape,
            fill: Default::default(),
            stroke: Default::default(),
            stroke_width: 0.0,
        }
    }

    pub fn fill(mut self, paint: impl Into<Paint>) -> Self {
        self.fill = paint.into();
        self
    }

    pub fn stroke(mut self, paint: impl Into<Paint>, width: f64) -> Self {
        self.stroke = paint.into();
        self.stroke_width = width;
        self
    }
}

impl<S: drawing::Shape + 'static> Widget for Shape<S> {
    type Element = Self; // no need for a separate type

    fn build(self, cx: &mut TreeCtx, element_id: ElementId) -> Self::Element {
        self
    }

    fn update(self, cx: &mut TreeCtx, element: &mut Self::Element) -> ChangeFlags {
        *element = self;
        ChangeFlags::GEOMETRY | ChangeFlags::PAINT
    }
}

impl<S: drawing::Shape + 'static> Element for Shape<S> {
    fn id(&self) -> ElementId {
        ElementId::ANONYMOUS
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, params: &LayoutParams) -> Geometry {
        Geometry::new(params.constrain(self.shape.bounding_box().size()))
    }

    fn event(&mut self, ctx: &mut EventCtx, event: &mut Event) -> ChangeFlags {
        ChangeFlags::NONE
    }

    fn natural_size(&mut self, axis: Axis, params: &LayoutParams) -> f64 {
        match axis {
            Axis::Horizontal => self.shape.bounding_box().width(),
            Axis::Vertical => self.shape.bounding_box().height(),
        }
    }

    fn natural_baseline(&mut self, params: &LayoutParams) -> f64 {
        0.0
    }

    fn hit_test(&self, ctx: &mut HitTestResult, position: Point) -> bool {
        false
    }

    fn paint(&mut self, ctx: &mut PaintCtx) {
        let mut surface = ctx.surface.surface();
        let mut canvas = surface.canvas();
        let bounding_box = self.shape.bounding_box();

        let mut stroke_paint = self.stroke.to_sk_paint(bounding_box);
        stroke_paint.set_style(skia::paint::Style::Stroke);
        stroke_paint.set_stroke_width(self.stroke_width as f32);
        let mut fill_paint = self.fill.to_sk_paint(bounding_box);
        fill_paint.set_style(skia::paint::Style::Fill);

        if let Some(rect) = self.shape.as_rect() {
            canvas.draw_rect(rect.to_skia(), &stroke_paint);
            canvas.draw_rect(rect.to_skia(), &fill_paint);
        } else if let Some(rrect) = self.shape.as_rounded_rect() {
            let rrect = rrect.to_skia();
            canvas.draw_rrect(rrect, &stroke_paint);
            canvas.draw_rrect(rrect, &fill_paint);
        } else if let Some(line) = self.shape.as_line() {
            todo!("line shape")
        } else {
            let mut sk_path: skia::Path = skia::Path::new();
            for elem in self.shape.path_elements(0.1) {
                match elem {
                    PathEl::MoveTo(p) => {
                        sk_path.move_to(p.to_skia());
                    }
                    PathEl::LineTo(p) => {
                        sk_path.line_to(p.to_skia());
                    }
                    PathEl::QuadTo(a, b) => {
                        sk_path.quad_to(a.to_skia(), b.to_skia());
                    }
                    PathEl::CurveTo(a, b, c) => {
                        sk_path.cubic_to(a.to_skia(), b.to_skia(), c.to_skia());
                    }
                    PathEl::ClosePath => {
                        sk_path.close();
                    }
                }
            }
            canvas.draw_path(&sk_path, &stroke_paint);
        };
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
*/
