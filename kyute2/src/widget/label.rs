use crate::{
    drawing::ToSkia,
    length::LengthResolutionParams,
    text::{get_font_collection, ChangeKind, TextSpan, TextStyle},
    ChangeFlags, Element, Environment, Event, EventCtx, Geometry, HitTestResult, LayoutCtx, LayoutParams, PaintCtx,
    Point, RouteEventCtx, TreeCtx, Widget, WidgetId,
};
use kurbo::Size;
use skia_safe as sk;
use std::any::Any;
use tracing::warn;

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
        TextElement {
            text: self.text.unwrap_or_default(),
            available_width: 0.0,
            available_height: 0.0,
            scale_factor: 0.0,
            parent_font_size: 0.0,
            relayout: true,
            paragraph: None,
        }
    }

    fn update(self, cx: &mut TreeCtx, element: &mut Self::Element, _env: &Environment) -> ChangeFlags {
        if let Some(text) = self.text {
            let change = text.compare_to(&element.text);
            match change {
                ChangeKind::Identical | ChangeKind::Metadata => ChangeFlags::NONE,
                ChangeKind::Paint => {
                    element.text = text;
                    ChangeFlags::PAINT
                }
                ChangeKind::Layout => {
                    element.relayout = true;
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

fn text_style_to_skia(
    style: &TextStyle,
    parent_font_size: f64,
    parent_height: f64,
    scale_factor: f64,
) -> sk::textlayout::TextStyle {
    let length_resolution_params = LengthResolutionParams {
        scale_factor,
        font_size: parent_font_size,
        container_size: parent_height,
    };
    let mut sk_style = sk::textlayout::TextStyle::new();
    if let Some(ref color) = style.color {
        sk_style.set_color(color.to_skia().to_color());
    }
    if let Some(ref font_size) = style.font_size {
        sk_style.set_font_size(*font_size/*.to_dips(&length_resolution_params)*/ as sk::scalar);
    }
    if let Some(ref font_families) = style.font_families {
        sk_style.set_font_families(&font_families[..]);
    }
    sk_style
}

fn add_text_span(
    text_span: &TextSpan,
    parent_font_size: f64,
    parent_height: f64,
    scale_factor: f64,
    paragraph: &mut sk::textlayout::ParagraphBuilder,
) {
    let has_style = !text_span.style.is_null();
    if has_style {
        let sk_style = text_style_to_skia(&text_span.style, parent_font_size, parent_height, scale_factor);
        paragraph.push_style(&sk_style);
    }

    if !text_span.text.is_empty() {
        paragraph.add_text(&text_span.text);
    }

    for child in text_span.children.iter() {
        add_text_span(child, parent_font_size, parent_height, scale_factor, paragraph);
    }

    if has_style {
        paragraph.pop();
    }
}

pub struct TextElement {
    text: TextSpan,
    available_width: f64,
    available_height: f64,
    parent_font_size: f64,
    scale_factor: f64,
    relayout: bool,
    paragraph: Option<sk::textlayout::Paragraph>,
}

impl Element for TextElement {
    fn id(&self) -> WidgetId {
        WidgetId::ANONYMOUS
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, params: &LayoutParams) -> Geometry {
        // layout paragraph in available space

        // available space for layout
        let available_width = params.max.width;
        let available_height = params.max.height;

        // We can reuse the previous layout if and only if:
        // - the scale factor is the same (font metrics can be specified in physical pixels, which depend on the scale factor)
        // - the new available width is >= the current paragraph width (otherwise new line breaks are necessary)
        // - the parent font size hasn't changed (font metrics can be relative to the parent font size)
        // - the current layout is still valid (i.e. it hasn't been previously invalidated)

        if !self.relayout
            && self.paragraph.as_ref().unwrap().max_width() <= params.max.width as f32
            && self.scale_factor == params.scale_factor
        {
            let paragraph = self.paragraph.as_ref().unwrap();
            let paragraph_size = Size {
                width: paragraph.max_width() as f64,
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

        self.paragraph = None;

        // default style
        let mut text_style = sk::textlayout::TextStyle::new();
        text_style.set_font_size(params.font_size as sk::scalar);
        let mut paragraph_style = sk::textlayout::ParagraphStyle::new();
        paragraph_style.set_text_style(&text_style);

        // create paragraph
        let font_collection = get_font_collection();
        let mut builder = sk::textlayout::ParagraphBuilder::new(&paragraph_style, font_collection);
        add_text_span(
            &self.text,
            params.font_size,
            available_height,
            params.scale_factor,
            &mut builder,
        );
        let paragraph = builder.build();
        let paragraph_size = Size {
            width: paragraph.max_width() as f64,
            height: paragraph.height() as f64,
        };
        let alphabetic_baseline = paragraph.alphabetic_baseline();
        let size = params.constrain(paragraph_size);
        self.paragraph = Some(paragraph);

        // update cached values
        self.available_width = available_width;
        self.available_height = available_height;
        self.scale_factor = params.scale_factor;
        self.parent_font_size = params.font_size;
        self.relayout = false;

        Geometry {
            size,
            baseline: Some(alphabetic_baseline as f64),
            bounding_rect: paragraph_size.to_rect(),
            paint_bounding_rect: paragraph_size.to_rect(),
        }
    }

    fn event(&mut self, ctx: &mut EventCtx, event: &mut Event) -> ChangeFlags {
        // this might change if there's dynamic text (e.g. hyperlinks)
        ChangeFlags::NONE
    }

    fn hit_test(&self, ctx: &mut HitTestResult, position: Point) -> bool {
        if let Some(ref paragraph) = self.paragraph {
            let paragraph_size = Size {
                width: paragraph.max_width() as f64,
                height: paragraph.height() as f64,
            };
            paragraph_size.to_rect().contains(position)
        } else {
            warn!("hit_test called before layout");
            false
        }
    }

    fn paint(&mut self, ctx: &mut PaintCtx) {
        todo!()
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
