use crate::{
    debug_util::DebugWriter,
    drawing::ToSkia,
    text::{get_font_collection, ChangeKind, TextSpan, TextStyle},
    widget::Axis,
    ChangeFlags, Element, Environment, Event, EventCtx, Geometry, HitTestResult, LayoutCtx, LayoutParams, PaintCtx,
    Point, RouteEventCtx, TreeCtx, Widget, WidgetId,
};
use kurbo::Size;
use skia_safe as sk;
use skia_safe::textlayout::ParagraphBuilder;
use std::any::Any;
use tracing::{trace_span, warn};

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

    fn id(&self) -> WidgetId {
        WidgetId::ANONYMOUS
    }

    fn build(self, cx: &mut TreeCtx, _env: &Environment) -> Self::Element {
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

    fn update(self, cx: &mut TreeCtx, element: &mut Self::Element, _env: &Environment) -> ChangeFlags {
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
    fn id(&self) -> WidgetId {
        WidgetId::ANONYMOUS
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, params: &LayoutParams) -> Geometry {
        // layout paragraph in available space
        let _span = trace_span!("text layout").entered();

        // available space for layout
        let available_width = params.max.width;
        let available_height = params.max.height;

        // We can reuse the previous layout if and only if:
        // - the new available width is >= the current paragraph width (otherwise new line breaks are necessary)
        // - the current layout is still valid (i.e. it hasn't been previously invalidated)

        if !self.relayout && self.paragraph.max_width() <= available_width as f32 {
            let paragraph_size = Size {
                width: self.paragraph.max_width() as f64,
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

        self.paragraph.layout(dbg!(available_width) as sk::scalar);
        let w = self.paragraph.max_width() as f64;
        let h = self.paragraph.height() as f64;
        let alphabetic_baseline = self.paragraph.alphabetic_baseline();
        let unconstrained_size = Size::new(w, h);
        let size = params.constrain(unconstrained_size);
        self.relayout = false;

        self.paragraph.max_width();
        self.paragraph.max_intrinsic_width();
        self.paragraph.height();
        self.paragraph.alphabetic_baseline();

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

    fn event(&mut self, ctx: &mut EventCtx, event: &mut Event) -> ChangeFlags {
        // this might change if there's dynamic text (e.g. hyperlinks)
        ChangeFlags::NONE
    }

    fn natural_size(&mut self, axis: Axis, params: &LayoutParams) -> f64 {
        match axis {
            Axis::Horizontal => self.paragraph.max_intrinsic_width() as f64,
            Axis::Vertical => {
                warn!("unimplemented: text element intrinsic height");
                dbg!(self.paragraph.alphabetic_baseline()) as f64
            }
        }
    }

    fn natural_baseline(&mut self, params: &LayoutParams) -> f64 {
        // this should work even before layout() is called
        // FIXME: yeah no it doesn't
        self.paragraph.alphabetic_baseline() as f64
    }

    fn hit_test(&self, ctx: &mut HitTestResult, position: Point) -> bool {
        if self.relayout {
            warn!("hit_test called before layout");
        }
        let paragraph_size = Size {
            width: self.paragraph.max_width() as f64,
            height: self.paragraph.height() as f64,
        };
        paragraph_size.to_rect().contains(position)
    }

    fn paint(&mut self, ctx: &mut PaintCtx) {
        self.paragraph
            .paint(ctx.surface.surface().canvas(), Point::ZERO.to_skia());
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn debug(&self, visitor: &mut DebugWriter) {
        visitor.type_name("TextElement");
        visitor.property("id", self.id());
        visitor.str_property("text", &self.text.text);
    }
}
