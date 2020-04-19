use crate::event::Event;
use crate::renderer::Theme;
use crate::visual::reconciliation::NodePlace;
use crate::visual::{EventCtx, PaintCtx};
use crate::widget::LayoutCtx;
use crate::{Bounds, BoxConstraints, BoxedWidget, Layout, Node, Point, Visual, Widget};
use kyute_shell::drawing::{Color, RectExt};
use std::any::Any;

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
    ) -> &'a mut Node
    {
        let mut node = place.get_or_insert_default::<FrameVisual>();
        node.visual.border_color = self.border_color;
        node.visual.border_width = self.border_width;
        node.visual.fill_color = self.fill_color;
        self.inner
            .layout(ctx, &mut node.visual.inner, constraints, theme);
        node.layout = node.visual.inner.layout;
        node
    }
}

pub struct FrameVisual {
    border_color: Color,
    border_width: f64,
    fill_color: Color,
    inner: Box<Node>,
}

impl Default for FrameVisual {
    fn default() -> Self {
        FrameVisual {
            border_color: Color::new(0.0, 0.0, 0.0, 1.0),
            border_width: 0.0,
            fill_color: Color::new(1.0, 1.0, 1.0, 1.0),
            inner: Node::dummy()
        }
    }
}

impl Visual for FrameVisual {
    fn paint(&mut self, ctx: &mut PaintCtx, theme: &Theme) {
        let rect = ctx.bounds();
        // box background
        ctx.fill_rectangle(rect.stroke_inset(self.border_width), self.fill_color);
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
