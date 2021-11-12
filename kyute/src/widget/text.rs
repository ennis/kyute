//! Text elements
use crate::{
    composable, env::Environment, event::Event, BoxConstraints, EventCtx, LayoutCtx, LayoutItem,
    Measurements, PaintCtx, Point, Rect, Size, Widget, WidgetPod,
};
use kyute_shell::{
    skia as sk,
    text::{TextFormatBuilder, TextLayout},
};
use std::cell::RefCell;

#[derive(Clone)]
pub struct Text {
    text: String,
    text_blob: RefCell<Option<sk::TextBlob>>,
}

impl Text {
    #[composable]
    pub fn new(text: String) -> WidgetPod<Text> {
        WidgetPod::new(Text {
            text,
            text_blob: RefCell::new(None),
        })
    }
}

impl Widget for Text {
    fn debug_name(&self) -> &str {
        std::any::type_name::<Self>()
    }

    fn event(&self, _ctx: &mut EventCtx, _event: &Event) {}

    fn layout(
        &self,
        _ctx: &mut LayoutCtx,
        constraints: BoxConstraints,
        _env: &Environment,
    ) -> Measurements {
        //let font_name = "Consolas";
        let font_size = 12;

        let mut font: sk::Font = sk::Font::new(sk::Typeface::default(), Some(font_size));
        font.set_subpixel(true);
        font.set_hinting(sk::FontHinting::Full);
        font.set_edging(sk::font::Edging::SubpixelAntiAlias);
        let text_blob = sk::TextBlob::from_str(&self.text, &font).unwrap();
        let paint: sk::Paint = sk::Paint::new(sk::Color4f::new(0.0, 0.0, 0.0, 1.0), None);
        let (_, bounds) = font.measure_str(&self.text, Some(&paint));
        let bounds = Rect {
            origin: Point::new(bounds.left, bounds.top),
            size: Size::new(bounds.right - bounds.left, bounds.bottom - bounds.top),
        };

        // round size to nearest device pixel
        let size = bounds.size.ceil();

        // TODO baseline
        /*let baseline = text_layout
        .line_metrics()
        .first()
        .map(|m| m.baseline as f64);*/

        self.text_blob.replace(Some(text_blob));
        Measurements {
            size,
            baseline: None, // TODO
            is_window: false,
        }
    }

    fn paint(&self, ctx: &mut PaintCtx, _bounds: Rect, _env: &Environment) {
        let text_blob = self.text_blob.borrow();

        if let Some(ref text_blob) = &*text_blob {
            let mut paint: sk::Paint = sk::Paint::new(sk::Color4f::new(1.0, 1.0, 1.0, 1.0), None);
            paint.set_anti_alias(true);
            ctx.canvas
                .draw_text_blob(&text_blob, sk::Point::new(0.0, 0.0), &paint);
            ctx.canvas.clear(sk::Color4f::new(0.1, 0.2, 0.7, 1.0));
        } else {
            tracing::warn!("text layout wasn't calculated before paint")
        }
    }
}
