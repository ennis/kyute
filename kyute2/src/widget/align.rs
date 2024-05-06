////////////////////////////////////////////////////////////////////////////////////////////////////

use crate::{
    layout::place_into,
    widget::{prelude::*, TransformNode},
    Alignment,
};
use kurbo::Insets;

pub struct Align<W> {
    pub x: Alignment,
    pub y: Alignment,
    pub width_factor: Option<f64>,
    pub height_factor: Option<f64>,
    // TODO a simple offset would be enough
    pub content: TransformNode<W>,
}

impl<W> Align<W> {
    pub fn new(x: Alignment, y: Alignment, content: W) -> Self {
        Self {
            x,
            y,
            width_factor: None,
            height_factor: None,
            content: TransformNode::new(content),
        }
    }
}

impl<W> Widget for Align<W>
where
    W: Widget,
{
    fn update(&self, cx: &mut TreeCtx) {
        self.content.update(cx)
    }

    fn event(&self, ctx: &mut TreeCtx, event: &mut Event) {
        self.content.event(ctx, event)
    }

    fn hit_test(&self, ctx: &mut HitTestResult, position: Point) -> bool {
        self.content.hit_test(ctx, position)
    }

    fn layout(&self, ctx: &mut LayoutCtx, constraints: &BoxConstraints) -> Geometry {
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
        self.content.set_offset(offset);
        Geometry {
            size,
            baseline: child_geometry.baseline.map(|baseline| baseline + offset.y),
            bounding_rect: child_geometry.bounding_rect + offset,
            paint_bounding_rect: child_geometry.paint_bounding_rect + offset,
        }
    }

    fn paint(&self, ctx: &mut PaintCtx) {
        self.content.paint(ctx)
    }
}

/*
////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct Align<W> {
    pub x: Alignment,
    pub y: Alignment,
    pub width_factor: Option<f64>,
    pub height_factor: Option<f64>,
    pub content: W,
}

impl<W> Align<W> {
    pub fn new(x: Alignment, y: Alignment, content: W) -> Self {
        Self {
            x,
            y,
            width_factor: None,
            height_factor: None,
            content,
        }
    }
}

impl<W> Widget for Align<W>
where
    W: Widget,
{
    type Element = AlignElement<W::Element>;

    fn build(self, cx: &mut TreeCtx, _id: ElementId) -> Self::Element {
        AlignElement {
            x: self.x,
            y: self.y,
            width_factor: self.width_factor,
            height_factor: self.height_factor,
            content: TransformNode::new(cx.build(self.content)),
        }
    }

    fn update(self, cx: &mut TreeCtx, element: &mut Self::Element) -> ChangeFlags {
        let mut flags = ChangeFlags::empty();
        if element.x != self.x
            || element.y != self.y
            || element.width_factor != self.width_factor
            || element.height_factor != self.height_factor
        {
            element.x = self.x;
            element.y = self.y;
            element.width_factor = self.width_factor;
            element.height_factor = self.height_factor;
            flags |= ChangeFlags::GEOMETRY;
        }
        flags | cx.update(self.content, &mut element.content.content)
    }
}*/
