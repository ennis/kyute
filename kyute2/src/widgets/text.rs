use kurbo::Size;
use skia_safe as sk;
use tracing::warn;
use tracy_client::span;

use crate::{
    drawing::ToSkia,
    text::{get_font_collection, TextSpan, TextStyle},
    Binding, BoxConstraints, Event, Geometry, HitTestResult, LayoutCtx, PaintCtx, Point, TreeCtx, Widget,
};

////////////////////////////////////////////////////////////////////////////////////////////////////

fn text_style_to_skia(style: &TextStyle) -> sk::textlayout::TextStyle {
    let mut sk_style = sk::textlayout::TextStyle::new();
    if let Some(ref color) = style.color {
        sk_style.set_color(color.to_skia().to_color());
    }
    if let Some(ref font_size) = style.font_size {
        sk_style.set_font_size(*font_size as sk::scalar);
    }
    if let Some(ref font_families) = style.font_families {
        sk_style.set_font_families(&font_families[..]);
    }
    sk_style
}

fn add_text_span(text_span: &TextSpan, paragraph: &mut sk::textlayout::ParagraphBuilder) {
    let has_style = !text_span.style.is_null();
    if has_style {
        let sk_style = text_style_to_skia(&text_span.style);
        paragraph.push_style(&sk_style);
    }

    if !text_span.text.is_empty() {
        paragraph.add_text(&text_span.text);
    }

    for child in text_span.children.iter() {
        add_text_span(child, paragraph);
    }

    if has_style {
        paragraph.pop();
    }
}

fn build_paragraph(text: &TextSpan) -> sk::textlayout::Paragraph {
    let _span = span!("build_paragraph");
    let font_collection = get_font_collection();
    let mut text_style = sk::textlayout::TextStyle::new();
    text_style.set_font_size(16.0 as sk::scalar); // TODO default font size
    let mut paragraph_style = sk::textlayout::ParagraphStyle::new();
    paragraph_style.set_text_style(&text_style);
    let mut builder = sk::textlayout::ParagraphBuilder::new(&paragraph_style, font_collection);
    add_text_span(text, &mut builder);
    builder.build()
}

pub struct Text {
    text: TextSpan,
    relayout: bool,
    paragraph: sk::textlayout::Paragraph,
}

impl Text {
    pub fn new(text: TextSpan) -> Text {
        let paragraph = build_paragraph(&text);
        Text {
            text,
            relayout: true,
            paragraph,
        }
    }
}

impl Widget for Text {
    fn update(&mut self, cx: &mut TreeCtx) {
        /*if self.text.update(cx) {
            self.relayout = true;
            self.paragraph = build_paragraph(&self.text.value());
            cx.mark_needs_layout();
        }*/
    }

    fn event(&mut self, _ctx: &mut TreeCtx, _event: &mut Event) {}

    fn hit_test(&mut self, _ctx: &mut HitTestResult, position: Point) -> bool {
        if self.relayout {
            warn!("hit_test called before layout");
        }
        let paragraph = &self.paragraph;
        let paragraph_size = Size {
            width: paragraph.longest_line() as f64,
            height: paragraph.height() as f64,
        };
        paragraph_size.to_rect().contains(position)
    }

    fn layout(&mut self, _ctx: &mut LayoutCtx, params: &BoxConstraints) -> Geometry {
        // layout paragraph in available space
        let _span = span!("text layout");

        // available space for layout
        let available_width = params.max.width;
        let _available_height = params.max.height;

        // We can reuse the previous layout if and only if:
        // - the new available width is >= the current paragraph width (otherwise new line breaks are necessary)
        // - the current layout is still valid (i.e. it hasn't been previously invalidated)

        let paragraph = &mut self.paragraph;

        if !self.relayout && paragraph.longest_line() <= available_width as f32 {
            let paragraph_size = Size {
                width: paragraph.longest_line() as f64,
                height: paragraph.height() as f64,
            };
            let size = params.constrain(paragraph_size);
            return Geometry {
                size,
                baseline: Some(paragraph.alphabetic_baseline() as f64),
                bounding_rect: paragraph_size.to_rect(),
                paint_bounding_rect: paragraph_size.to_rect(),
            };
        }

        paragraph.layout(available_width as sk::scalar);
        let w = paragraph.longest_line() as f64;
        let h = paragraph.height() as f64;
        let alphabetic_baseline = paragraph.alphabetic_baseline();
        let unconstrained_size = Size::new(w, h);
        let size = params.constrain(unconstrained_size);
        self.relayout = false;

        /*self.paragraph.max_width();
        self.paragraph.max_intrinsic_width();
        self.paragraph.height();
        self.paragraph.alphabetic_baseline();*/

        // update cached values
        //self.available_width = available_width;
        //self.available_height = available_height;
        //self.scale_factor = params.scale_factor;

        Geometry {
            size,
            baseline: Some(alphabetic_baseline as f64),
            bounding_rect: size.to_rect(),
            paint_bounding_rect: size.to_rect(),
        }
    }

    fn paint(&mut self, ctx: &mut PaintCtx) {
        ctx.with_canvas(|canvas| {
            self.paragraph.paint(canvas, Point::ZERO.to_skia());
        })
    }
}
