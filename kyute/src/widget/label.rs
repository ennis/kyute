//! Text elements
use crate::{
    composable, drawing::FromSkia, env::Environment, event::Event, style::ColorRef, theme, BoxConstraints, EventCtx,
    LayoutCtx, Measurements, PaintCtx, Point, Rect, Widget, WidgetIdentity,
};
use skia_safe as sk;
use std::cell::RefCell;

/// Style of a text label.
#[derive(Copy, Clone, Debug)]
pub struct TextStyle {
    pub color: ColorRef,
}

/// Simple text label.
#[derive(Clone)]
pub struct Label {
    state: WidgetIdentity,
    style: TextStyle,
    text: String,
    text_blob: RefCell<Option<sk::TextBlob>>,
}

impl Label {
    /// Creates a new text label.
    #[composable(cached)]
    pub fn new(text: String) -> Label {
        Label {
            state: WidgetIdentity::new(),
            style: TextStyle {
                // by default, use LABEL_COLOR as the text color
                color: theme::keys::LABEL_COLOR.into(),
            },
            text,
            text_blob: RefCell::new(None),
        }
    }

    /// Sets the color of the label.
    pub fn color(mut self, color: impl Into<ColorRef>) -> Self {
        self.set_color(color);
        self
    }

    /// Sets the color of the label.
    pub fn set_color(&mut self, color: impl Into<ColorRef>) {
        self.style.color = color.into();
    }
}

impl Widget for Label {
    fn widget_identity(&self) -> Option<&WidgetIdentity> {
        Some(&self.state)
    }

    fn debug_name(&self) -> &str {
        std::any::type_name::<Self>()
    }

    fn event(&self, _ctx: &mut EventCtx, _event: &mut Event, _env: &Environment) {}

    fn layout(&self, ctx: &mut LayoutCtx, _constraints: BoxConstraints, env: &Environment) -> Measurements {
        //let font_name = "Consolas";
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
        }
    }

    fn paint(&self, ctx: &mut PaintCtx, _bounds: Rect, _env: &Environment) {
        let text_blob = self.text_blob.borrow();

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
        }
    }
}
