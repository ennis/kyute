use kurbo::{Insets, Vec2};

use crate::{
    core::{WeakWidget, WeakWidgetPtr, WidgetCtx},
    layout::place_into,
    prelude::*,
    Alignment, WidgetPtrAny,
};

pub struct Align {
    weak: WeakWidgetPtr<Self>,
    pub x: Alignment,
    pub y: Alignment,
    pub width_factor: Option<f64>,
    pub height_factor: Option<f64>,
    offset: Vec2,
    pub content: WidgetPtrAny,
}

impl Align {
    pub fn new(x: Alignment, y: Alignment, content: WidgetPtr<impl Widget>) -> WidgetPtr<Self> {
        WidgetPod::new_cyclic(move |weak| Align {
            weak,
            x,
            y,
            width_factor: None,
            height_factor: None,
            offset: Default::default(),
            content,
        })
    }
}

impl WeakWidget for Align {
    fn weak_self(&self) -> WeakWidgetPtr<Self> {
        self.weak.clone()
    }
}

impl Widget for Align {
    fn mount(&mut self, cx: &mut Ctx) {
        self.content.mount(cx)
    }

    fn hit_test(&mut self, ctx: &mut HitTestResult, position: Point) -> bool {
        ctx.test_with_offset(self.offset, position, |result, position| {
            self.content.hit_test(result, position)
        })
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, constraints: &BoxConstraints) -> Geometry {
        let child_geometry = self.content.layout(ctx, &constraints.loosen());

        // first, size to max available width/height
        let mut size = Size {
            width: if constraints.max.width.is_finite() {
                constraints.max.width
            } else {
                child_geometry.size.width
            },
            height: if constraints.max.height.is_finite() {
                constraints.max.height
            } else {
                child_geometry.size.height
            },
        };

        // If width/height factors are present, override size according to them,
        // but don't let it go below the child's size.
        // Setting a factor to <1.0 can be used to make sure that the widget won't expand.
        if let Some(width) = self.width_factor {
            size.width = child_geometry.size.width * width.max(1.0);
        }
        if let Some(height) = self.height_factor {
            size.height = child_geometry.size.height * height.max(1.0);
        }

        // Apply parent constraints. The size might be below the minimum constraint, this
        // will push them back to the minimum accepted size.
        size = constraints.constrain(size);

        let offset = place_into(
            child_geometry.size,
            child_geometry.baseline,
            size,
            None,
            self.x,
            self.y,
            &Insets::ZERO,
        );
        self.offset = offset;

        Geometry {
            size,
            baseline: child_geometry.baseline.map(|baseline| baseline + offset.y),
            bounding_rect: child_geometry.bounding_rect + offset,
            paint_bounding_rect: child_geometry.paint_bounding_rect + offset,
        }
    }

    fn paint(&mut self, ctx: &mut PaintCtx) {
        ctx.with_offset(self.offset, |ctx| self.content.paint(ctx))
    }
}
