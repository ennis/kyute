use crate::event::Event;
use crate::layout::{BoxConstraints, Layout, Size};
use crate::renderer::Theme;
use crate::visual::reconciliation::NodePlace;
use crate::visual::{EventCtx, Node, PaintCtx, Visual};
use crate::widget::LayoutCtx;
use crate::{Bounds, Point, Widget};
use kyute_shell::drawing::{Color, DrawTextOptions};
use kyute_shell::text::TextLayout;
use log::trace;
use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

pub struct TextVisual {
    text: String,
    text_layout: TextLayout,
}

impl Visual for TextVisual {
    fn paint(&mut self, ctx: &mut PaintCtx, theme: &Theme) {
        ctx.draw_text_layout(
            Point::origin(),
            &self.text_layout,
            Color::new(1.0, 1.0, 1.0, 1.0),
            DrawTextOptions::empty(),
        )
    }

    fn hit_test(&mut self, _point: Point, _bounds: Bounds) -> bool {
        false
    }
    fn event(&mut self, _ctx: &mut EventCtx, _event: &Event) {}
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// Text element.
pub struct Text {
    text: String,
}

impl<A: 'static> Widget<A> for Text {
    fn layout<'a>(
        self,
        ctx: &mut LayoutCtx<A>,
        place: &'a mut dyn NodePlace,
        constraints: &BoxConstraints,
        theme: &Theme,
    ) -> &'a mut Node {
        place.reconcile(|_prev: Option<Node<TextVisual>>| {
            // TODO check for changes instead of re-creating from scratch every time
            let text_layout = TextLayout::new(
                ctx.platform(),
                &self.text,
                &theme.label_text_format,
                constraints.biggest(),
            )
            .unwrap();

            let text_size = text_layout.metrics().bounds.size.ceil();

            let baseline = text_layout
                .line_metrics()
                .first()
                .map(|m| m.baseline.ceil() as f64);

            let layout = Layout::new(text_size).with_baseline(baseline);

            Node::new(
                layout,
                None,
                TextVisual {
                    text: self.text,
                    text_layout,
                },
            )
        })
    }
}

impl Text {
    pub fn new(text: impl Into<String>) -> Text {
        Text { text: text.into() }
    }
}
