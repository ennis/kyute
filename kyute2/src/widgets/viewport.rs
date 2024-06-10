use kurbo::{Point, Rect, Size, Vec2};

use crate::{BoxConstraints, Ctx, Event, Geometry, HitTestResult, LayoutCtx, PaintCtx, Widget, WidgetPtr};

pub struct Viewport {
    size: Size,
    offset: Vec2,
    constrain_width: bool,
    constrain_height: bool,
    content: WidgetPtr,
}

impl Viewport {
    pub fn new(content: WidgetPtr) -> Self {
        Viewport {
            size: Size::ZERO,
            offset: Vec2::ZERO,
            constrain_width: false,
            constrain_height: false,
            content,
        }
    }

    pub fn set_x_offset(&mut self, x: f64) {
        self.offset.x = x;
    }

    pub fn set_y_offset(&mut self, y: f64) {
        self.offset.y = y;
    }

    /// Returns whether the viewport fully contains the given rectangle.
    pub fn contains_rect(&self, rect: Rect) -> bool {
        let viewport_rect = Rect::from_origin_size(self.offset.to_point(), self.size);
        // TODO maybe there's a better approach
        viewport_rect.union(rect) == viewport_rect
    }

    /// Sets the X offset of the viewport such that the given point (in the coordinate space inside the viewport) is in view.
    pub fn horizontal_scroll_to(&mut self, x: f64) {
        if x - self.offset.x > self.size.width {
            // pos overflow to the right
            self.offset.x = x - self.size.width;
        } else if x - self.offset.x < 0.0 {
            // pos overflow to the left
            self.offset.x = x;
        }
    }
}

/*
impl<Content: 'static> WeakWidget for Viewport<Content> {
    fn weak_self(&self) -> WeakWidgetPtr<Self> {
        self.weak.clone()
    }
}*/

impl Widget for Viewport {
    fn mount(&mut self, cx: &mut Ctx) {
        self.content.mount(cx)
    }

    fn hit_test(&mut self, result: &mut HitTestResult, position: Point) -> bool {
        if self.size.to_rect().contains(position) {
            self.content.hit_test(result, position)
        } else {
            false
        }
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, constraints: &BoxConstraints) -> Geometry {
        let mut child_constraints = BoxConstraints::default();
        if self.constrain_width {
            child_constraints.set_width_range(constraints.width_range());
        }
        if self.constrain_height {
            child_constraints.set_height_range(constraints.height_range());
        }

        let child_layout = self.content.layout(ctx, &child_constraints);
        self.content.set_offset(self.offset);

        // always take the maximum available space
        // if the constraints are unbounded in a direction, we use the child's size
        self.size.width = constraints.finite_max_width().unwrap_or(child_layout.size.width);
        self.size.height = constraints.finite_max_height().unwrap_or(child_layout.size.height);
        Geometry::new(self.size)
    }

    fn paint(&mut self, ctx: &mut PaintCtx) {
        let clip_rect = self.size.to_rect();
        ctx.with_clip_rect(clip_rect, |ctx| {
            self.content.paint(ctx);
        });
    }
}
