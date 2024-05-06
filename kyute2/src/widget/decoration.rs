//! Frame decorations

use std::{any::Any, cell::Cell};

use kurbo::{Insets, PathEl, Rect, RoundedRect};
use smallvec::SmallVec;
use tracing::warn;

use crate::{
    drawing,
    drawing::{BoxShadow, Paint, Shape, ToSkia},
    skia,
    widget::{prelude::*, Padding},
    Color, PaintCtx,
};

/// Represents a decoration that can be applied to a widget.
///
/// TODO document
pub trait Decoration: PartialEq {
    fn insets(&self) -> Insets;
    fn paint(&self, ctx: &mut PaintCtx, rect: Rect);
}

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
pub trait ShapeBorder: PartialEq {
    type Shape: Shape;
    fn dimensions(&self) -> Insets;
    fn inner_shape(&self, rect: Rect) -> Self::Shape;
    fn outer_shape(&self, rect: Rect) -> Self::Shape;
    fn paint(&self, ctx: &mut PaintCtx, rect: Rect);
}

#[derive(PartialEq)]
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
#[derive(PartialEq)]
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

#[derive(PartialEq)]
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
}

impl<B: ShapeBorder> Decoration for ShapeDecoration<B> {
    fn insets(&self) -> Insets {
        self.border.dimensions()
    }

    fn paint(&self, ctx: &mut PaintCtx, rect: Rect) {
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
            } else if let Some(_line) = inner_shape.as_line() {
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

pub struct DecoratedBox<D> {
    decoration: D,
    size: Cell<Size>,
    content: WidgetPtr,
}

impl<D: Decoration> DecoratedBox<D> {
    pub fn new(decoration: D, content: impl Widget + 'static) -> Self {
        let padding = decoration.insets();
        Self {
            decoration,
            size: Default::default(),
            content: WidgetPod::new(Padding::new(padding, content)),
        }
    }
}

impl<D> Widget for DecoratedBox<D>
where
    D: Decoration + 'static,
{
    fn update(&self, cx: &mut TreeCtx) {
        self.content.update(cx)
    }

    /*fn natural_width(&mut self, height: f64) -> f64 {
        self.content.natural_width(height)
    }

    fn natural_height(&mut self, width: f64) -> f64 {
        self.content.natural_height(width)
    }

    fn natural_baseline(&mut self, params: &BoxConstraints) -> f64 {
        self.content.natural_baseline(params)
    }*/

    fn event(&self, ctx: &mut TreeCtx, event: &mut Event) {
        self.content.event(ctx, event)
    }

    fn hit_test(&self, ctx: &mut HitTestResult, position: Point) -> bool {
        self.content.hit_test(ctx, position) || self.size.get().to_rect().contains(position)
    }

    fn layout(&self, ctx: &mut LayoutCtx, constraints: &BoxConstraints) -> Geometry {
        let mut geometry = self.content.layout(ctx, constraints);
        // assume that the decoration expands the paint bounds
        geometry.bounding_rect = geometry.bounding_rect.union(geometry.size.to_rect());
        geometry.paint_bounding_rect = geometry.paint_bounding_rect.union(geometry.size.to_rect());
        self.size.set(geometry.size);
        geometry
    }

    fn paint(&self, ctx: &mut PaintCtx) {
        self.decoration.paint(ctx, self.size.get().to_rect());
        self.content.paint(ctx);
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

/*
// TODO this should take a "Decoration" directly, not a Border
pub struct DecoratedBox<D, W> {
    pub decoration: D,
    pub content: W,
}

impl<D, W> DecoratedBox<D, W> {
    pub fn new(decoration: D, content: W) -> Self {
        Self { decoration, content }
    }
}

impl<D, W> Widget for DecoratedBox<D, W>
where
    D: Decoration + 'static,
    W: Widget,
{
    type Element = DecoratedBoxElement<D, W::Element>;

    fn build(self, cx: &mut TreeCtx, _id: ElementId) -> Self::Element {
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
        if element.decoration != self.decoration {
            let padding = self.decoration.insets();
            if element.content.padding != padding {
                element.content.padding = padding;
                flags |= ChangeFlags::GEOMETRY;
            }
            element.decoration = self.decoration;
            flags |= ChangeFlags::PAINT;
        }
        flags | cx.update(self.content, &mut element.content.content)
    }
}
*/
