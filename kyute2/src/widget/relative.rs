//! Relative positioning widgets
use crate::{
    composable, elem_node::TransformNode, Alignment, ChangeFlags, Element, Environment, Event, EventCtx, Geometry,
    HitTestResult, LayoutCtx, LayoutParams, PaintCtx, RouteEventCtx, Size, TreeCtx, Vec2, Widget, WidgetId,
};
use kurbo::Point;
use std::any::Any;
use tracing::warn;

/*/// Place A relative to B.
pub struct Adjacent<A, B> {
    id: WidgetId,
    a: A,
    b: B,
}

impl<A: Widget + 'static, B: Widget + 'static> Adjacent<A, B> {
    #[composable]
    pub fn new(a: A, b: B) -> Adjacent<A, B> {
        let id = WidgetId::here();
        Adjacent { id, a, b }
    }
}

impl<A, B> Widget for Adjacent<A, B>
where
    A: Widget,
    B: Widget,
{
    type Element = AdjacentElement<A::Element, B::Element>;

    fn id(&self) -> WidgetId {
        self.id
    }

    fn build(self, cx: &mut TreeCtx, env: &Environment) -> Self::Element {
        AdjacentElement {
            id: self.id,
            a: TransformNode::new(cx.build(self.a, env)),
            b: TransformNode::new(cx.build(self.b, env)),
        }
    }

    fn update(self, cx: &mut TreeCtx, element: &mut Self::Element, env: &Environment) -> ChangeFlags {
        let mut change_flags = ChangeFlags::empty();
        change_flags |= element.a.update(cx, self.a, env);
        change_flags |= element.b.update(cx, self.b, env);
        change_flags
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

fn anchor_pos(anchor: Alignment, box_size: f64, baseline: Option<f64>) -> f64 {
    match anchor {
        Alignment::Relative(f) => f * box_size,
        Alignment::FirstBaseline => baseline.unwrap_or(0.0),
        Alignment::LastBaseline => {
            // TODO LastBaseline
            baseline.unwrap_or(box_size)
        }
    }
}

pub struct AdjacentElement<A, B> {
    id: WidgetId,
    a: TransformNode<A>,
    b: TransformNode<B>,
}

impl<A, B> Element for AdjacentElement<A, B>
where
    A: Element,
    B: Element,
{
    fn id(&self) -> WidgetId {
        self.id
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, params: &LayoutParams) -> Geometry {
        let layout_a = self.a.layout(ctx, params);
        let layout_b = self.b.layout(ctx, params);

        let size_a = layout_a.padding_box_size();
        let baseline_a = layout_a.padding_box_baseline();
        let size_b = layout_b.padding_box_size();
        let baseline_b = layout_b.padding_box_baseline();

        let anchor_a = Point::new(
            anchor_pos(layout_a.x_align, size_a.width, None),
            anchor_pos(layout_a.y_align, size_a.height, baseline_a),
        );

        let anchor_b = Point::new(
            anchor_pos(layout_b.x_align, size_b.width, None),
            anchor_pos(layout_b.y_align, size_b.height, baseline_b),
        );

        // offset_b + anchorPos(b) == offset_a + anchorPos(a)
        //      where offset_b, offset_a are outputs, either offset_b or offset_a == 0
        let (x_a, x_b) = if anchor_a.x < anchor_b.x {
            (anchor_b.x - anchor_a.x, 0.0)
        } else {
            (0.0, anchor_a.x - anchor_b.x)
        };
        let (y_a, y_b) = if anchor_a.y < anchor_b.y {
            (anchor_b.y - anchor_a.y, 0.0)
        } else {
            (0.0, anchor_a.y - anchor_b.y)
        };

        let width = f64::max(anchor_a.x, anchor_b.x) + f64::max(size_a.width - anchor_a.x, size_b.width - anchor_b.x);
        let height =
            f64::max(anchor_a.y, anchor_b.y) + f64::max(size_a.height - anchor_a.y, size_b.height - anchor_b.y);
        let size = Size::new(width, height);
        // keep baseline of the first element
        let baseline = baseline_a.map(|b| b + y_a);

        let a_offset = Vec2::new(x_a + layout_a.padding.x0, y_a + layout_a.padding.y0);
        let b_offset = Vec2::new(x_a + layout_a.padding.x0, y_a + layout_a.padding.y0);
        self.a.set_offset(a_offset);
        self.b.set_offset(b_offset);

        Geometry {
            x_align: Default::default(),
            y_align: Default::default(),
            padding: Default::default(),
            size,
            baseline,
            bounding_rect: (layout_a.bounding_rect + a_offset).union(layout_b.bounding_rect + b_offset),
            paint_bounding_rect: (layout_a.paint_bounding_rect + a_offset)
                .union(layout_b.paint_bounding_rect + b_offset),
        }
    }

    fn event(&mut self, _ctx: &mut EventCtx, _event: &mut Event) -> ChangeFlags {
        ChangeFlags::empty()
    }

    fn route_event(&mut self, ctx: &mut RouteEventCtx, child: WidgetId, event: &mut Event) -> ChangeFlags {
        if child == self.a.id() {
            ctx.route_event(&mut self.a, event)
        } else if child == self.b.id() {
            ctx.route_event(&mut self.b, event)
        } else {
            warn!("invalid child ID");
            ChangeFlags::empty()
        }
    }

    fn hit_test(&self, ctx: &mut HitTestResult, position: Point) -> bool {
        todo!()
    }

    fn paint(&mut self, ctx: &mut PaintCtx) {
        self.a.paint(ctx);
        self.b.paint(ctx);
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}*/
