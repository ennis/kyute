//! Frame decorations

use crate::{
    drawing,
    drawing::{BoxShadow, Paint, Shape, ToSkia},
    element::TransformNode,
    skia,
    widget::{padding::PaddingElement, prelude::*},
    Color, PaintCtx,
};
use kurbo::{Affine, Insets, PathEl, Rect, RoundedRect, Vec2};
use smallvec::SmallVec;
use std::any::Any;
use tracing::warn;

////////////////////////////////////////////////////////////////////////////////////////////////////
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BorderStyle {
    None,
    Solid,
}

impl Default for BorderStyle {
    fn default() -> Self {
        BorderStyle::None
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
pub trait ShapeBorder {
    type Shape: Shape;
    fn dimensions(&self) -> Insets;
    fn inner_shape(&self, rect: Rect) -> Self::Shape;
    fn outer_shape(&self, rect: Rect) -> Self::Shape;
    fn paint(&self, ctx: &mut PaintCtx, rect: Rect);
}

pub struct RoundedRectBorder {
    pub color: Color,
    pub radius: f64,
    pub dimensions: Insets,
    pub style: BorderStyle,
}

impl Default for RoundedRectBorder {
    fn default() -> Self {
        RoundedRectBorder {
            color: Default::default(),
            radius: 0.0,
            dimensions: Default::default(),
            style: BorderStyle::None,
        }
    }
}

impl ShapeBorder for RoundedRectBorder {
    type Shape = RoundedRect;

    fn dimensions(&self) -> Insets {
        self.dimensions
    }

    fn inner_shape(&self, rect: Rect) -> Self::Shape {
        // FIXME: multiple radii
        RoundedRect::from_rect(rect - self.dimensions, self.radius - 0.5 * self.dimensions.x_value())
    }

    fn outer_shape(&self, rect: Rect) -> Self::Shape {
        RoundedRect::from_rect(rect, self.radius)
    }

    fn paint(&self, ctx: &mut PaintCtx, rect: Rect) {
        if self.style == BorderStyle::None {
            return;
        }

        let mut paint = Paint::Color(self.color).to_sk_paint(rect);
        paint.set_style(skia::paint::Style::Fill);

        ctx.with_canvas(|canvas| {
            //if self.radius == 0.0 {
            let outer_rrect = self.outer_shape(rect).to_skia();
            let inner_rrect = self.inner_shape(rect).to_skia();
            canvas.draw_drrect(outer_rrect, inner_rrect, &paint);
        });
    }
}

/// Applies border A, then border B.
pub struct CompoundBorder<Inner, Outer> {
    inner: Inner,
    outer: Outer,
}

impl<Inner, Outer, S> ShapeBorder for CompoundBorder<Inner, Outer>
where
    S: Shape,
    Inner: ShapeBorder<Shape = S>,
    Outer: ShapeBorder<Shape = S>,
{
    type Shape = S;

    fn dimensions(&self) -> Insets {
        let da = self.inner.dimensions();
        let db = self.outer.dimensions();
        Insets {
            x0: da.x0 + db.x0,
            y0: da.y0 + db.y0,
            x1: da.x1 + db.x1,
            y1: da.y1 + db.y1,
        }
    }

    fn inner_shape(&self, rect: Rect) -> Self::Shape {
        self.inner.inner_shape(rect - self.outer.dimensions())
    }

    fn outer_shape(&self, rect: Rect) -> Self::Shape {
        self.outer.outer_shape(rect)
    }

    fn paint(&self, ctx: &mut PaintCtx, rect: Rect) {
        self.inner.paint(ctx, rect - self.outer.dimensions());
        self.outer.paint(ctx, rect);
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

//impl<S,Inner> BoxShadow<> for InnerShadow<>

// TODO: a generic "decoration" trait? inset + outset + paint?
pub struct ShapeDecoration<Border> {
    pub fill: Paint,
    pub border: Border,
    pub shadows: SmallVec<[BoxShadow; 2]>,
}

impl ShapeDecoration<RoundedRectBorder> {
    pub fn new() -> ShapeDecoration<RoundedRectBorder> {
        ShapeDecoration {
            fill: Paint::default(),
            border: RoundedRectBorder {
                color: Default::default(),
                radius: 0.0,
                dimensions: Default::default(),
                style: BorderStyle::None,
            },
            shadows: Default::default(),
        }
    }
}

impl<B: ShapeBorder> ShapeDecoration<B> {
    pub fn border<C>(self, border: C) -> ShapeDecoration<CompoundBorder<B, C>> {
        ShapeDecoration {
            fill: self.fill,
            border: CompoundBorder {
                inner: self.border,
                outer: border,
            },

            shadows: self.shadows,
        }
    }

    pub fn box_shadow(mut self, shadow: BoxShadow) -> Self {
        self.shadows.push(shadow);
        self
    }

    pub fn insets(&self) -> Insets {
        self.border.dimensions()
    }

    pub fn paint(&self, ctx: &mut PaintCtx, rect: Rect) {
        let inner_shape = self.border.inner_shape(rect);
        let outer_shape = self.border.outer_shape(rect);

        // draw drop shadows
        ctx.with_canvas(|canvas| {
            for shadow in &self.shadows {
                if !shadow.inset {
                    if let Some(rect) = outer_shape.as_rect() {
                        drawing::draw_box_shadow(canvas, &rect.to_rounded_rect(0.0), shadow);
                    } else if let Some(rrect) = outer_shape.as_rounded_rect() {
                        drawing::draw_box_shadow(canvas, &rrect, shadow);
                    } else {
                        warn!("shadows are currently only implemented for rects and rounded rect shapes")
                    };
                }
            }
        });

        // fill
        let mut paint = self.fill.to_sk_paint(rect);
        paint.set_style(skia::paint::Style::Fill);
        ctx.with_canvas(|canvas| {
            if let Some(rect) = inner_shape.as_rect() {
                canvas.draw_rect(rect.to_skia(), &paint);
            } else if let Some(rrect) = inner_shape.as_rounded_rect() {
                canvas.draw_rrect(rrect.to_skia(), &paint);
            } else if let Some(line) = inner_shape.as_line() {
                todo!("line shape")
            } else {
                let mut sk_path: skia::Path = skia::Path::new();
                for elem in inner_shape.path_elements(0.1) {
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
                canvas.draw_path(&sk_path, &paint);
            };
        });

        // draw inset shadows
        ctx.with_canvas(|canvas| {
            for shadow in &self.shadows {
                if shadow.inset {
                    if let Some(rect) = inner_shape.as_rect() {
                        drawing::draw_box_shadow(canvas, &rect.to_rounded_rect(0.0), shadow);
                    } else if let Some(rrect) = inner_shape.as_rounded_rect() {
                        drawing::draw_box_shadow(canvas, &rrect, shadow);
                    } else {
                        warn!("shadows are currently only implemented for rects and rounded rect shapes")
                    };
                }
            }
        });

        // paint borders
        self.border.paint(ctx, rect);
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct DecoratedBoxElement<Border, E> {
    decoration: ShapeDecoration<Border>,
    content: PaddingElement<E>,
}

/*impl<Border: ShapeBorder, E> DecoratedBoxElement<Border, E> {
    pub fn new(decoration: ShapeDecoration<Border>, content: E) -> Self {
        let padding = decoration.insets();
        Self {
            decoration,
            content: PaddingElement {
                padding,
                size: Default::default(),
                content,
            },
        }
    }
}*/

impl<Border, E> Element for DecoratedBoxElement<Border, E>
where
    Border: ShapeBorder + 'static,
    E: Element,
{
    fn id(&self) -> ElementId {
        self.content.id()
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, constraints: &BoxConstraints) -> Geometry {
        let mut geometry = ctx.layout(&mut self.content, constraints);
        // assume that the decoration expands the paint bounds
        geometry.bounding_rect = geometry.bounding_rect.union(geometry.size.to_rect());
        geometry.paint_bounding_rect = geometry.paint_bounding_rect.union(geometry.size.to_rect());
        geometry
    }

    fn event(&mut self, ctx: &mut EventCtx, event: &mut Event) -> ChangeFlags {
        self.content.event(ctx, event)
    }

    fn natural_width(&mut self, height: f64) -> f64 {
        self.content.natural_width(height)
    }

    fn natural_height(&mut self, width: f64) -> f64 {
        self.content.natural_height(width)
    }

    fn natural_baseline(&mut self, params: &BoxConstraints) -> f64 {
        self.content.natural_baseline(params)
    }

    fn hit_test(&self, ctx: &mut HitTestResult, position: Point) -> bool {
        if self.content.hit_test(ctx, position) {
            return true;
        }
        if self.content.size.to_rect().contains(position) {
            ctx.add(self.id());
            return true;
        }
        return false;
    }

    fn paint(&mut self, ctx: &mut PaintCtx) {
        self.decoration.paint(ctx, self.content.size.to_rect());
        self.content.paint(ctx);
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn debug(&self, w: &mut DebugWriter) {
        w.type_name("DecoratedBoxElement");
        w.child("content", &self.content);
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
pub struct DecoratedBox<Border, W> {
    pub decoration: ShapeDecoration<Border>,
    pub content: W,
}

impl<Border, W> DecoratedBox<Border, W> {
    pub fn new(decoration: ShapeDecoration<Border>, content: W) -> Self {
        Self { decoration, content }
    }
}

impl<Border, W> Widget for DecoratedBox<Border, W>
where
    Border: ShapeBorder + 'static,
    W: Widget,
{
    type Element = DecoratedBoxElement<Border, W::Element>;

    fn build(self, cx: &mut TreeCtx, id: ElementId) -> Self::Element {
        let padding = self.decoration.insets();
        DecoratedBoxElement {
            decoration: self.decoration,
            content: PaddingElement {
                padding,
                content: cx.build(self.content),
                size: Default::default(),
            },
        }
    }

    fn update(self, cx: &mut TreeCtx, element: &mut Self::Element) -> ChangeFlags {
        let mut flags = ChangeFlags::empty();
        // TODO compare decorations
        let padding = self.decoration.insets();
        element.decoration = self.decoration;
        element.content.padding = padding;
        // TODO
        //flags |= ChangeFlags::GEOMETRY;
        flags | cx.update(self.content, &mut element.content.content)
    }
}
