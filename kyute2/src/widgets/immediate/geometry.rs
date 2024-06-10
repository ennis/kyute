//! Geometry descriptions for immediate mode widgets.
use crate::{
    widgets::immediate::{
        linsys::{var, LinExpr},
        VarId, IMCTX,
    },
    WidgetPtr,
};

/// Describes a 2D point.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Point {
    pub x: VarId,
    pub y: VarId,
}

impl Point {
    pub fn new() -> Point {
        Point { x: var(), y: var() }
    }

    pub fn equals(&self, other: impl Into<Point>) {
        let other = other.into();
        self.x.equals(other.x);
        self.y.equals(other.y);
    }

    pub fn resolve(&self) -> kurbo::Point {
        kurbo::Point::new(self.x.resolve(), self.y.resolve())
    }
}

impl<A, B> From<(A, B)> for Point
where
    A: Into<LinExpr>,
    B: Into<LinExpr>,
{
    fn from(value: (A, B)) -> Self {
        Point {
            x: value.0.into().bind(),
            y: value.1.into().bind(),
        }
    }
}

/// Describes a 2D size.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Size {
    pub width: VarId,
    pub height: VarId,
}

impl Size {
    pub fn new() -> Size {
        Size {
            width: var(),
            height: var(),
        }
    }

    pub fn equals(&self, other: impl Into<Size>) {
        let other = other.into();
        self.width.equals(other.width);
        self.height.equals(other.height);
    }
}

impl<A, B> From<(A, B)> for Size
where
    A: Into<LinExpr>,
    B: Into<LinExpr>,
{
    fn from(value: (A, B)) -> Self {
        Size {
            width: value.0.into().bind(),
            height: value.1.into().bind(),
        }
    }
}
/// Describes a rectangle.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Rect {
    pub left: VarId,
    pub right: VarId,
    pub top: VarId,
    pub bottom: VarId,
    pub center: Point,
    pub width: VarId,
    pub height: VarId,
}

impl Rect {
    pub fn top_left(&self) -> Point {
        Point {
            x: self.left,
            y: self.top,
        }
    }

    pub fn top_right(&self) -> Point {
        Point {
            x: self.right,
            y: self.top,
        }
    }

    pub fn bottom_left(&self) -> Point {
        Point {
            x: self.left,
            y: self.bottom,
        }
    }

    pub fn bottom_right(&self) -> Point {
        Point {
            x: self.right,
            y: self.bottom,
        }
    }

    pub fn size(&self) -> Size {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    pub fn resolve(&self) -> kurbo::Rect {
        let left = self.left.resolve();
        let top = self.top.resolve();
        let right = self.right.resolve();
        let bottom = self.bottom.resolve();
        kurbo::Rect::new(left, top, right, bottom)
    }

    /// Constrains the aspect ratio of the rectangle.
    ///
    /// Equivalent to `self.width.equals(aspect_ratio * self.height)`.
    pub fn aspect_ratio(&self, aspect_ratio: f64) {
        self.width.equals(aspect_ratio * self.height);
    }
}

/// Describes a circle.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Circle {
    pub center: Point,
    pub radius: VarId,
    pub left: VarId,
    pub right: VarId,
    pub top: VarId,
    pub bottom: VarId,
}

impl Circle {
    pub fn resolve(&self) -> kurbo::Circle {
        kurbo::Circle {
            center: self.center.resolve(),
            radius: self.radius.resolve(),
        }
    }
}

pub fn circle() -> Circle {
    let center = point();
    let radius = var();
    let left = var();
    let right = var();
    let top = var();
    let bottom = var();

    left.equals(center.x - radius);
    right.equals(center.x + radius);
    top.equals(center.y - radius);
    bottom.equals(center.y + radius);

    Circle {
        center,
        radius,
        left,
        right,
        top,
        bottom,
    }
}

/// Undefined rectangle.
pub fn rect() -> Rect {
    let left = var();
    let right = var();
    let top = var();
    let bottom = var();
    let center = point();
    let width = var();
    let height = var();

    width.equals(right - left);
    height.equals(bottom - top);
    center.x.equals((left + right) * 0.5);
    center.y.equals((top + bottom) * 0.5);

    Rect {
        left,
        right,
        top,
        bottom,
        center,
        width,
        height,
    }
}

/// Rectangle by its top-left corner and size.
pub fn rect_xywh(x: impl Into<LinExpr>, y: impl Into<LinExpr>, w: impl Into<LinExpr>, h: impl Into<LinExpr>) -> Rect {
    let r = rect();
    r.left.equals(x);
    r.top.equals(y);
    r.width.equals(w);
    r.height.equals(h);
    r
}

/// Defines a rectangle by its center and size.
pub fn rect_center_size(center: impl Into<Point>, size: impl Into<Size>) -> Rect {
    let center = center.into();
    let size = size.into();
    let r = rect();
    r.center.equals(center);
    r.width.equals(size.width);
    r.height.equals(size.height);
    r
}

/// Defines a rectangle by its left, top, right and bottom edges.
pub fn rect_ltrb(l: impl Into<LinExpr>, t: impl Into<LinExpr>, r: impl Into<LinExpr>, b: impl Into<LinExpr>) -> Rect {
    let rect = rect();
    rect.left.equals(l);
    rect.top.equals(t);
    rect.right.equals(r);
    rect.bottom.equals(b);
    rect
}

/// Undefined point.
pub fn point() -> Point {
    Point::new()
}

pub fn point_xy(x: impl Into<LinExpr>, y: impl Into<LinExpr>) -> Point {
    let p = point();
    p.x.equals(x);
    p.y.equals(y);
    p
}

/// Two undefined points making a line segment.
pub fn line() -> (Point, Point) {
    let p1 = Point::new();
    let p2 = Point::new();
    (p1, p2)
}

/// Linear interpolation between two expressions.
pub fn lerp(a: impl Into<LinExpr>, b: impl Into<LinExpr>, t: f64) -> LinExpr {
    a.into() * (1.0 - t) + b.into() * t
}

pub fn min_width() -> f64 {
    IMCTX.with(|imctx| imctx.constraints.min.width)
}

pub fn max_width() -> f64 {
    IMCTX.with(|imctx| imctx.constraints.max.width)
}

pub fn min_height() -> f64 {
    IMCTX.with(|imctx| imctx.constraints.min.height)
}

pub fn max_height() -> f64 {
    IMCTX.with(|imctx| imctx.constraints.max.height)
}

pub fn width() -> VarId {
    IMCTX.with(|imctx| imctx.width)
}

pub fn height() -> VarId {
    IMCTX.with(|imctx| imctx.height)
}

pub fn baseline() -> VarId {
    IMCTX.with(|imctx| imctx.baseline)
}

#[derive(Copy, Clone, Debug)]
pub struct WidgetBox {
    pub rect: Rect,
}

/// Embeds another widget
pub fn widget_box(widget: WidgetPtr) -> WidgetBox {
    let rect = rect();
    IMCTX.with(|imctx| imctx.add_child_widget(widget, rect));
    WidgetBox { rect }
}
