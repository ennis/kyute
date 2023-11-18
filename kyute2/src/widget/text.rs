use std::any::Any;

use kurbo::Size;
use skia_safe as sk;
use tracing::warn;
use tracy_client::span;

use crate::{
    debug_util::DebugWriter,
    drawing::ToSkia,
    text::{get_font_collection, ChangeKind, TextSpan, TextStyle},
    BoxConstraints, ChangeFlags, Element, ElementId, Event, EventCtx, Geometry, HitTestResult, LayoutCtx, PaintCtx,
    Point, TreeCtx, Widget,
};

/// A simple text label.
#[derive(Clone, Default)]
pub struct Text {
    text: Option<TextSpan>,
}

impl Text {
    pub fn new(text: TextSpan) -> Text {
        Text { text: Some(text) }
    }
}

impl Widget for Text {
    type Element = TextElement;

    fn build(self, _cx: &mut TreeCtx, _element_id: ElementId) -> Self::Element {
        let text = self.text.unwrap_or_default();
        let paragraph = build_paragraph(&text);
        TextElement {
            text,
            available_width: 0.0,
            available_height: 0.0,
            scale_factor: 0.0,
            relayout: true,
            paragraph,
        }
    }

    fn update(self, _cx: &mut TreeCtx, element: &mut Self::Element) -> ChangeFlags {
        if let Some(text) = self.text {
            let change = text.compare_to(&element.text);
            match change {
                ChangeKind::Identical | ChangeKind::Metadata => ChangeFlags::NONE,
                ChangeKind::Paint => {
                    element.paragraph = build_paragraph(&text);
                    element.text = text;
                    ChangeFlags::PAINT
                }
                ChangeKind::Layout => {
                    element.relayout = true;
                    element.paragraph = build_paragraph(&text);
                    element.text = text;
                    ChangeFlags::GEOMETRY
                }
            }
        } else {
            ChangeFlags::NONE
        }
    }
}

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

pub struct TextElement {
    text: TextSpan,
    available_width: f64,
    available_height: f64,
    scale_factor: f64,
    relayout: bool,
    paragraph: sk::textlayout::Paragraph,
}

impl TextElement {
    /*fn ensure_paragraph(&mut self) {
        if self.paragraph.is_none() {
            let font_collection = get_font_collection();
            let mut builder = sk::textlayout::ParagraphBuilder::new(
                &sk::textlayout::ParagraphStyle::new(),
                font_collection,
            );
            add_text_span(&self.text, &mut builder);
            self.paragraph = Some(builder.build());
        }
    }*/
}

impl Element for TextElement {
    fn id(&self) -> ElementId {
        ElementId::ANONYMOUS
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

        if !self.relayout && self.paragraph.longest_line() <= available_width as f32 {
            let paragraph_size = Size {
                width: self.paragraph.longest_line() as f64,
                height: self.paragraph.height() as f64,
            };
            let size = params.constrain(paragraph_size);
            return Geometry {
                size,
                baseline: Some(self.paragraph.alphabetic_baseline() as f64),
                bounding_rect: paragraph_size.to_rect(),
                paint_bounding_rect: paragraph_size.to_rect(),
            };
        }

        self.paragraph.layout(available_width as sk::scalar);
        let w = self.paragraph.longest_line() as f64;
        let h = self.paragraph.height() as f64;
        let alphabetic_baseline = self.paragraph.alphabetic_baseline();
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

    fn event(&mut self, _ctx: &mut EventCtx, _event: &mut Event) -> ChangeFlags {
        // this might change if there's dynamic text (e.g. hyperlinks)
        ChangeFlags::NONE
    }

    fn natural_width(&mut self, _height: f64) -> f64 {
        self.paragraph.max_intrinsic_width() as f64
    }

    fn natural_height(&mut self, _width: f64) -> f64 {
        warn!("unimplemented: text element intrinsic height");
        dbg!(self.paragraph.alphabetic_baseline()) as f64
    }

    fn natural_baseline(&mut self, _params: &BoxConstraints) -> f64 {
        // this should work even before layout() is called
        // FIXME: yeah no it doesn't
        self.paragraph.alphabetic_baseline() as f64
    }

    fn hit_test(&self, _ctx: &mut HitTestResult, position: Point) -> bool {
        if self.relayout {
            warn!("hit_test called before layout");
        }
        let paragraph_size = Size {
            width: self.paragraph.longest_line() as f64,
            height: self.paragraph.height() as f64,
        };
        paragraph_size.to_rect().contains(position)
    }

    fn paint(&mut self, ctx: &mut PaintCtx) {
        span!("text paint");
        ctx.with_canvas(|canvas| {
            self.paragraph.paint(canvas, Point::ZERO.to_skia());
        })
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn debug(&self, w: &mut DebugWriter) {
        w.type_name("TextElement");
        w.property("id", self.id());
        w.property("baseline", self.paragraph.alphabetic_baseline());
        w.property("longest_line", self.paragraph.longest_line());
        w.property("max_width", self.paragraph.max_width());
        w.property("height", self.paragraph.height());
        w.str_property("text", &self.text.text);
    }
}
