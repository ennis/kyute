//! Text elements
use crate::{
    composable, env::Environment, event::Event, theme, widget::Text, BoxConstraints, Color, Data, EventCtx, LayoutCtx,
    Measurements, PaintCtx, Rect, Widget, WidgetId,
};
use kyute_text::FormattedText;

/// Style of a text label.
#[derive(Copy, Clone, Debug)]
pub struct TextStyle {}

/// Simple text label.
#[derive(Clone)]
pub struct Label {
    text: Text,
    color: Color,
}

impl Label {
    /// Creates a new text label.
    #[composable(cached)]
    pub fn new(text: impl Into<String> + Data) -> Label {
        let text = text.into();
        let color = theme::keys::LABEL_COLOR.get().unwrap();
        Label {
            text: Text::new(FormattedText::new(text.into())),
            color,
        }
    }

    /// Sets the color of the label.
    pub fn color(mut self, color: Color) -> Self {
        self.set_color(color);
        self
    }

    /// Sets the color of the label.
    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }
}

impl Widget for Label {
    fn widget_id(&self) -> Option<WidgetId> {
        self.text.widget_id()
    }

    fn event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        self.text.event(ctx, event, env)
    }

    fn layout(&self, ctx: &mut LayoutCtx, constraints: BoxConstraints, env: &Environment) -> Measurements {
        self.text.layout(ctx, constraints, env)

        /*//let font_name = "Consolas";
        let font_size = env.get(theme::LABEL_FONT_SIZE).unwrap().to_dips(ctx.scale_factor);

        let mut font: sk::Font = sk::Font::new(sk::Typeface::default(), Some(font_size as f32));
        font.set_subpixel(true);
        font.set_hinting(sk::FontHinting::Full);
        font.set_edging(sk::font::Edging::SubpixelAntiAlias);
        let text_blob = sk::TextBlob::from_str(&self.text, &font).unwrap();
        let paint: sk::Paint = sk::Paint::new(sk::Color4f::new(0.0, 0.0, 0.0, 1.0), None);
        let (_, bounds) = font.measure_str(&self.text, Some(&paint));
        let bounds = Rect::from_skia(bounds);

        // round size to nearest device pixel
        let size = bounds.size.ceil();
        let baseline = -bounds.origin.y;

        self.text_blob.replace(Some(text_blob));
        Measurements {
            bounds: Rect::new(Point::origin(), size),
            baseline: Some(baseline), // TODO
        }*/
    }

    fn paint(&self, ctx: &mut PaintCtx, bounds: Rect, env: &Environment) {
        self.text.paint(ctx, bounds, env);

        /*let text_blob = self.text_blob.borrow();
        if let Some(ref text_blob) = &*text_blob {
            let mut paint: sk::Paint = sk::Paint::new(sk::Color4f::new(1.0, 1.0, 1.0, 1.0), None);
            paint.set_anti_alias(true);
            ctx.canvas.draw_text_blob(
                &text_blob,
                sk::Point::new(0.0, ctx.measurements().baseline.unwrap_or(0.0) as f32),
                &paint,
            );
        } else {
            tracing::warn!("text layout wasn't calculated before paint")
        }*/
    }
}
