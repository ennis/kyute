use kyute_shell::drawing::{Color, RectExt};
use crate::{BoxedWidget, Widget, BoxConstraints, Node, Layout, Visual, Point, Bounds};
use crate::widget::LayoutCtx;
use crate::visual::reconciliation::NodePlace;
use crate::renderer::Theme;
use crate::visual::{PaintCtx, EventCtx};
use std::any::Any;
use crate::event::Event;

/// A widget that draws a frame.
pub struct Frame<A> {
    pub border_color: Color,
    pub border_width: f64,
    pub fill_color: Color,
    pub inner: BoxedWidget<A>,
}

impl<A: 'static> Widget<A> for Frame<A> {
    fn layout<'a>(
        self,
        ctx: &mut LayoutCtx<A>,
        place: &'a mut dyn NodePlace,
        constraints: &BoxConstraints,
        theme: &Theme,
    ) -> &'a mut Node {
        place.reconcile(|_| {
            let mut visual = FrameVisual {
                border_color: self.border_color,
                border_width: self.border_width,
                fill_color: self.fill_color,
                inner: Node::dummy(),   // FIXME unnecessary allocation
            };
            self.inner.layout(ctx, &mut visual.inner, constraints, theme);
            let mut size = visual.inner.layout.size;
            let layout = Layout::new(size).with_baseline(visual.inner.layout.baseline);
            Node::new(layout, None, visual)
        })
    }
}

pub struct FrameVisual {
    border_color: Color,
    border_width: f64,
    fill_color: Color,
    inner: Box<Node>,
}

impl Visual for FrameVisual {
    fn paint(&mut self, ctx: &mut PaintCtx, theme: &Theme) {
        let rect = ctx.bounds();
        // box background
        ctx.fill_rectangle(
            rect.stroke_inset(self.border_width),
            self.fill_color,
        );
        // border
        ctx.draw_rectangle(
            rect.stroke_inset(self.border_width),
            self.border_color,
            self.border_width,
        );
        // child
        self.inner.paint(ctx, theme);
    }

    fn hit_test(&mut self, point: Point, bounds: Bounds) -> bool {
        unimplemented!()
    }
    fn event(&mut self, ctx: &mut EventCtx, event: &Event) {
        self.inner.event(ctx, event);
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
