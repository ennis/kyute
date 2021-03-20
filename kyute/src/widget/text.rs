use crate::{
    event::Event,
    layout::{BoxConstraints, Measurements},
    theme, Environment, EventCtx, LayoutCtx, PaintCtx, Point, Rect, TypedWidget, Visual, Widget,
};
use generational_indextree::NodeId;
use kyute_shell::{
    drawing::{Brush, Color, DrawTextOptions, IntoBrush},
    text::{TextFormatBuilder, TextLayout},
};
use log::trace;
use std::any::Any;

pub struct TextVisual {
    text: String,
    text_layout: TextLayout,
}

impl Visual for TextVisual {
    fn paint(&mut self, ctx: &mut PaintCtx, env: &Environment) {
        let text_brush = Brush::new_solid_color(ctx, Color::new(0.92, 0.92, 0.92, 1.0));
        ctx.draw_text_layout(
            Point::origin(),
            &self.text_layout,
            &text_brush,
            DrawTextOptions::empty(),
        )
    }

    fn hit_test(&mut self, _point: Point, _bounds: Rect) -> bool {
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

impl TypedWidget for Text {
    type Visual = TextVisual;

    fn key(&self) -> Option<u64> {
        None
    }

    fn layout(
        self,
        context: &mut LayoutCtx,
        previous_visual: Option<Box<Self::Visual>>,
        constraints: &BoxConstraints,
        env: Environment,
    ) -> (Box<Self::Visual>, Measurements) {
        let font_name = env.get(theme::FontName);
        let font_size = env.get(theme::FontSize);

        // TODO re-creating a TextFormat every time might be inefficient; verify the cost of
        // creating many TextFormats
        let text_format = TextFormatBuilder::new(context.platform())
            .size(font_size as f32)
            .family(font_name)
            .build()
            .expect("failed to create text format");

        // TODO check for changes instead of re-creating from scratch every time
        let text_layout = TextLayout::new(
            context.platform(),
            &self.text,
            &text_format,
            constraints.biggest(),
        )
        .unwrap();

        // round size to nearest device pixel
        let text_size = text_layout.metrics().bounds.size.ceil();

        let baseline = text_layout
            .line_metrics()
            .first()
            .map(|m| m.baseline as f64);

        let measurements = Measurements {
            size: text_size,
            baseline,
        };

        let visual = TextVisual {
            text: self.text,
            text_layout,
        };

        (Box::new(visual), measurements)
    }
}

impl Text {
    pub fn new(text: impl Into<String>) -> Text {
        Text { text: text.into() }
    }
}
