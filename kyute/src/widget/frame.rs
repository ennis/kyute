use crate::event::Event;
use crate::renderer::Theme;
use crate::visual::{EventCtx, NodeArena, NodeCursor, PaintCtx};
use crate::widget::LayoutCtx;
use crate::{Bounds, BoxConstraints, BoxedWidget, Layout, NodeData, Point, Visual, Widget};
use generational_indextree::NodeId;
use kyute_shell::drawing::{Color, IntoBrush, RectExt};
use std::any::Any;

/// A widget that draws a frame.
pub struct Frame<A> {
    pub border_color: Color,
    pub border_width: f64,
    pub fill_color: Color,
    pub inner: BoxedWidget<A>,
}

impl<A: 'static> Widget<A> for Frame<A> {
    fn layout(
        self,
        ctx: &mut LayoutCtx<A>,
        nodes: &mut NodeArena,
        cursor: &mut NodeCursor,
        constraints: &BoxConstraints,
        theme: &Theme,
    ) -> NodeId {
        let Frame {
            border_color,
            border_width,
            fill_color,
            inner,
        } = self;

        let node_id = cursor.get_or_insert_with(nodes, || {
            NodeData::new(
                Layout::default(),
                None,
                FrameVisual {
                    border_color,
                    border_width,
                    fill_color,
                },
            )
        });

        let child_id = inner.layout_child(ctx, nodes, node_id, constraints, theme);

        let child_layout = nodes[child_id].get().layout;
        nodes[node_id].get_mut().layout = child_layout;
        node_id
    }
}

pub struct FrameVisual {
    border_color: Color,
    border_width: f64,
    fill_color: Color,
}

impl Default for FrameVisual {
    fn default() -> Self {
        FrameVisual {
            border_color: Color::new(0.0, 0.0, 0.0, 1.0),
            border_width: 0.0,
            fill_color: Color::new(1.0, 1.0, 1.0, 1.0),
        }
    }
}

impl Visual for FrameVisual {
    fn paint(&mut self, ctx: &mut PaintCtx, theme: &Theme) {
        let rect = ctx.bounds();
        let bg_brush = self.fill_color.into_brush(ctx);
        let border_brush = self.border_color.into_brush(ctx);
        // box background
        ctx.fill_rectangle(rect.stroke_inset(self.border_width), &bg_brush);
        // border
        if ctx.is_hovering() {
            ctx.draw_rectangle(
                rect.stroke_inset(self.border_width),
                &border_brush,
                self.border_width,
            );
        }
    }

    fn hit_test(&mut self, point: Point, bounds: Bounds) -> bool {
        unimplemented!()
    }
    fn event(&mut self, ctx: &mut EventCtx, event: &Event) {
        match event {
            Event::PointerOver(_) | Event::PointerOut(_) => {
                ctx.request_redraw()
            }
            _ => {}
        }
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
