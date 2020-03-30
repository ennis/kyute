use crate::layout::{PaintLayout, BoxConstraints, Layout, Size};
use crate::renderer::{Painter, Renderer, TextLayout};
use crate::visual::{Node, Visual, Cursor, PaintCtx};
use crate::{Widget, Point};
use crate::event::{Event, EventCtx};
use crate::widget::LayoutCtx;
use std::any::Any;
use log::trace;

pub struct TextVisual {
    text: String,
    text_layout: TextLayout,
}

impl Visual for TextVisual {
    fn paint(&mut self, ctx: &mut PaintCtx) {
        trace!("Painting text into {}", ctx.bounds);
        ctx.painter.draw_text(ctx.bounds.origin, &self.text_layout)
    }

    fn hit_test(&mut self, _point: Point, _layout: &PaintLayout) -> bool {
        // TODO
        false
    }

    fn event(&mut self, _event_ctx: &EventCtx, _event: &Event) {
    }

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

impl<A> Widget<A> for Text
{
    fn layout(self, ctx: &mut LayoutCtx<A>, cursor: &mut Cursor, constraints: &BoxConstraints)
    {
        let text = self.text;

        let text_layout = ctx.renderer.layout_text(&text, constraints.biggest());
        let text_size = Size::new(
            text_layout.metrics().width as f64,
            text_layout.metrics().height as f64,
        )
        .ceil();

        let baseline = text_layout
            .line_metrics()
            .first()
            .map(|m| m.baseline.ceil() as f64);

        let layout = Layout::new(text_size).with_baseline(baseline);
        trace!("Text layout {:?}", layout);
        cursor.overwrite(None, layout, TextVisual {
            text, text_layout
        });
    }
}

impl Text {
    pub fn new(text: impl Into<String>) -> Text {
        Text { text: text.into() }
    }
}
