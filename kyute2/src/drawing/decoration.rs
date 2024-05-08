use crate::{
    drawing,
    drawing::{BorderStyle, BoxShadow, Paint, ToSkia},
    skia, Color,
};
use kurbo::{Insets, PathEl, Rect, RoundedRect, Shape};
use skia_safe as sk;
use smallvec::SmallVec;
use tracing::warn;

/// Represents a decoration that can be applied to a widget.
///
/// TODO document
pub trait Decoration: PartialEq {
    /// Insets for the content of the widget to which this decoration is applied.
    fn insets(&self) -> Insets;
    /// Draws the decoration.
    fn paint(&self, canvas: &mut sk::Canvas, rect: Rect);
}

////////////////////////////////////////////////////////////////////////////////////////////////////

/// Represents a border with a specific shape.
///
///
pub trait ShapeBorder: PartialEq {
    /// The shape of the border.
    type Shape: Shape;
    /// Width of the border on each side.
    fn dimensions(&self) -> Insets;
    /// Inner shape of the border.
    fn inner_shape(&self, rect: Rect) -> Self::Shape;
    /// Outer shape of the border.
    fn outer_shape(&self, rect: Rect) -> Self::Shape;
    /// Draws the border.
    fn paint(&self, canvas: &mut sk::Canvas, rect: Rect);
}

/// Border with a rounded rectangle shape.
#[derive(Clone, PartialEq)]
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

    fn paint(&self, canvas: &mut sk::Canvas, rect: Rect) {
        if self.style == BorderStyle::None {
            return;
        }

        let mut paint = Paint::Color(self.color).to_sk_paint(rect);
        paint.set_style(skia::paint::Style::Fill);

        let outer_rrect = self.outer_shape(rect).to_skia();
        let inner_rrect = self.inner_shape(rect).to_skia();
        canvas.draw_drrect(outer_rrect, inner_rrect, &paint);
    }
}

/// Border that is the combination of an inner border and an outer border.
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

    fn paint(&self, canvas: &mut sk::Canvas, rect: Rect) {
        self.inner.paint(canvas, rect - self.outer.dimensions());
        self.outer.paint(canvas, rect);
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

/// Decoration composed of a background fill, a border, and box shadows.
#[derive(Clone, PartialEq)]
pub struct ShapeDecoration<Border> {
    pub fill: Paint,
    pub border: Border,
    pub shadows: SmallVec<[BoxShadow; 2]>,
}

impl ShapeDecoration<RoundedRectBorder> {
    /// Creates a new shape decoration with default parameters (i.e. invisible).
    ///
    /// The decoration has a transparent fill and no shadows.
    /// The default border type is a rounded rectangle but the border width is 0.
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
    /// Changes the border of this `ShapeDecoration`.
    #[must_use]
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

    /// Adds a box shadow to this `ShapeDecoration`.
    #[must_use]
    pub fn box_shadow(mut self, shadow: BoxShadow) -> Self {
        self.shadows.push(shadow);
        self
    }
}

impl<B: ShapeBorder> Decoration for ShapeDecoration<B> {
    fn insets(&self) -> Insets {
        self.border.dimensions()
    }

    fn paint(&self, canvas: &mut sk::Canvas, rect: Rect) {
        let inner_shape = self.border.inner_shape(rect);
        let outer_shape = self.border.outer_shape(rect);

        // draw drop shadows
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

        // fill
        let mut paint = self.fill.to_sk_paint(rect);
        paint.set_style(skia::paint::Style::Fill);
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

        // draw inset shadows
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

        // paint borders
        self.border.paint(canvas, rect);
    }
}
