//! Relative positioning widgets
use crate::widget::prelude::*;

/// Place A relative to B.
pub struct Adjacent<A, B> {
    a: WidgetPod<A>,
    b: WidgetPod<B>,
}

impl<A: Widget + 'static, B: Widget + 'static> Adjacent<A, B> {
    pub fn new(a: A, b: B) -> Adjacent<A, B> {
        Adjacent { a: a.pod(), b: b.pod() }
    }
}

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

impl<A, B> Widget for Adjacent<A, B>
where
    A: Widget,
    B: Widget,
{
    fn widget_id(&self) -> Option<WidgetId> {
        None
    }

    fn layout(&self, ctx: &mut LayoutCtx, params: &LayoutParams, env: &Environment) -> Geometry {
        let layout_a = self.a.layout(ctx, params, env);
        let layout_b = self.b.layout(ctx, params, env);

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

        self.a
            .set_offset(Offset::new(x_a + layout_a.padding_left, y_a + layout_a.padding_top));
        self.b
            .set_offset(Offset::new(x_b + layout_b.padding_left, y_b + layout_b.padding_top));

        Geometry {
            x_align: Default::default(),
            y_align: Default::default(),
            padding_left: 0.0,
            padding_top: 0.0,
            padding_right: 0.0,
            padding_bottom: 0.0,
            measurements: Measurements {
                size,
                clip_bounds: None,
                baseline,
            },
        }
    }

    fn event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        self.a.event(ctx, event, env);
        self.b.event(ctx, event, env);
    }

    fn paint(&self, ctx: &mut PaintCtx) {
        self.a.paint(ctx);
        self.b.paint(ctx);
    }
}
