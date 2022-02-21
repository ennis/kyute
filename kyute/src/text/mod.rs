use crate::{
    drawing::{FromSkia, ToSkia},
    Color, Data, Point, Rect,
};
use skia_safe as sk;
use std::{
    cmp::Ordering,
    ops::{Bound, Range, RangeBounds},
    sync::Arc,
};

/// Resolves a `RangeBounds` into a range in the range 0..len.
pub fn resolve_range(range: impl RangeBounds<usize>, len: usize) -> Range<usize> {
    let start = match range.start_bound() {
        Bound::Unbounded => 0,
        Bound::Included(n) => *n,
        Bound::Excluded(n) => *n + 1,
    };

    let end = match range.end_bound() {
        Bound::Unbounded => len,
        Bound::Included(n) => *n + 1,
        Bound::Excluded(n) => *n,
    };

    start.min(len)..end.min(len)
}

/// Describes a font weight.
///
/// It is a value between 1 and 1000, based on the CSS [`font-weight`](https://developer.mozilla.org/en-US/docs/Web/CSS/font-weight) property.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(transparent)]
pub struct FontWeight(pub u16);

impl FontWeight {
    pub const THIN: FontWeight = FontWeight(100);
    pub const EXTRA_LIGHT: FontWeight = FontWeight(200);
    pub const ULTRA_LIGHT: FontWeight = FontWeight(200);
    pub const LIGHT: FontWeight = FontWeight(300);
    pub const NORMAL: FontWeight = FontWeight(400);
    pub const REGULAR: FontWeight = FontWeight(400);
    pub const MEDIUM: FontWeight = FontWeight(500);
    pub const SEMI_BOLD: FontWeight = FontWeight(600);
    pub const DEMI_BOLD: FontWeight = FontWeight(600);
    pub const BOLD: FontWeight = FontWeight(700);
    pub const EXTRA_BOLD: FontWeight = FontWeight(800);
    pub const ULTRA_BOLD: FontWeight = FontWeight(800);
    pub const BLACK: FontWeight = FontWeight(900);
    pub const HEAVY: FontWeight = FontWeight(900);
}

/// Describes a font family.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FontFamily(Arc<str>);

impl FontFamily {
    pub fn new(name: impl Into<Arc<str>>) -> FontFamily {
        FontFamily(name.into())
    }
}

/// Font styling options (normal, italic, or oblique).
// NOTE: "style" may be a bit vague for what this represents: for example, skia uses "slant" instead.
// Still, we choose to follow the name of the CSS property ("font-style").
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum FontStyle {
    Normal,
    Italic,
    Oblique,
}

impl ToSkia for FontStyle {
    type Target = sk::font_style::Slant;
    fn to_skia(&self) -> Self::Target {
        match *self {
            FontStyle::Normal => sk::font_style::Slant::Upright,
            FontStyle::Italic => sk::font_style::Slant::Italic,
            FontStyle::Oblique => sk::font_style::Slant::Oblique,
        }
    }
}

/// Text alignment within a text paragraph.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum TextAlignment {
    Left,
    Right,
    Center,
    Justify,
    Start,
    End,
}

impl ToSkia for TextAlignment {
    type Target = sk::textlayout::TextAlign;
    fn to_skia(&self) -> Self::Target {
        match *self {
            TextAlignment::Left => sk::textlayout::TextAlign::Left,
            TextAlignment::Right => sk::textlayout::TextAlign::Right,
            TextAlignment::Center => sk::textlayout::TextAlign::Center,
            TextAlignment::Justify => sk::textlayout::TextAlign::Justify,
            TextAlignment::Start => sk::textlayout::TextAlign::Start,
            TextAlignment::End => sk::textlayout::TextAlign::End,
        }
    }
}

/// Attributes that can be applied to text.
#[derive(Clone, Debug, PartialEq)]
pub enum Attribute {
    /// Font size in points (1/72 inch).
    FontSize(f64),
    /// Font family.
    FontFamily(FontFamily),
    /// Font style (normal, italic, or oblique).
    FontStyle(FontStyle),
    /// Font weight.
    FontWeight(FontWeight),
    /// Color.
    Color(Color),
}

impl From<FontFamily> for Attribute {
    fn from(ff: FontFamily) -> Self {
        Attribute::FontFamily(ff)
    }
}

impl From<FontStyle> for Attribute {
    fn from(fs: FontStyle) -> Self {
        Attribute::FontStyle(fs)
    }
}

impl From<FontWeight> for Attribute {
    fn from(fw: FontWeight) -> Self {
        Attribute::FontWeight(fw)
    }
}

#[derive(Clone, Debug, PartialEq)]
struct TextRun {
    range: Range<usize>,
    attributes: Vec<Attribute>,
}

impl TextRun {
    fn merge_attribute(&mut self, attribute: &Attribute) {
        let mut found = false;
        for attr in self.attributes.iter_mut() {
            match (attr, attribute) {
                (Attribute::FontSize(fs), Attribute::FontSize(new_fs)) => {
                    *fs = *new_fs;
                    found = true;
                    break;
                }
                (Attribute::FontFamily(ff), Attribute::FontFamily(new_ff)) => {
                    *ff = new_ff.clone();
                    found = true;
                    break;
                }
                (Attribute::FontStyle(fs), Attribute::FontStyle(new_fs)) => {
                    *fs = *new_fs;
                    found = true;
                    break;
                }
                (Attribute::FontWeight(fw), Attribute::FontWeight(new_fw)) => {
                    *fw = *new_fw;
                    found = true;
                    break;
                }
                (Attribute::Color(c), Attribute::Color(new_color)) => {
                    *c = *new_color;
                    found = true;
                    break;
                }
                _ => {}
            }
        }

        if !found {
            self.attributes.push(attribute.clone())
        }
    }
}

#[derive(Clone, Debug)]
struct TextRuns {
    runs: Vec<TextRun>,
}

impl TextRuns {
    /*fn new() -> TextRuns {
        TextRuns { runs: vec![] }
    }*/

    fn search_run(&self, text_pos: usize) -> Result<usize, usize> {
        self.runs.binary_search_by(|run| {
            if run.range.contains(&text_pos) {
                Ordering::Equal
            } else if text_pos < run.range.start {
                Ordering::Greater
            } else {
                Ordering::Less
            }
        })
    }

    fn split(&mut self, range: Range<usize>) -> (usize, usize) {
        let Range { start, end } = range;
        let start_run = match self.search_run(start) {
            Ok(i) => {
                if start != self.runs[i].range.start {
                    let mut run = self.runs[i].clone();
                    run.range.end = start;
                    self.runs[i].range.start = start;
                    self.runs.insert(i, run);
                    i + 1
                } else {
                    i
                }
            }
            Err(i) => {
                let next_start = if i < self.runs.len() {
                    self.runs[i].range.start
                } else {
                    end
                };
                self.runs.insert(
                    i,
                    TextRun {
                        range: start..next_start,
                        attributes: vec![],
                    },
                );
                i
            }
        };

        let end_run = match self.search_run(end - 1) {
            Ok(i) => {
                if end != self.runs[i].range.end {
                    let mut run = self.runs[i].clone();
                    run.range.end = end;
                    self.runs[i].range.start = end;
                    self.runs.insert(i, run);
                }
                i
            }
            Err(i) => {
                let prev_end = if i > 0 { self.runs[i - 1].range.end } else { start };
                self.runs.insert(
                    i,
                    TextRun {
                        range: prev_end..end,
                        attributes: vec![],
                    },
                );
                i
            }
        };

        (start_run, end_run)
    }

    fn merge_attribute(&mut self, range: Range<usize>, attribute: &Attribute) {
        if range.is_empty() {
            return;
        }

        let (first_run, last_run) = self.split(range);
        for run in &mut self.runs[first_run..=last_run] {
            run.merge_attribute(attribute);
        }
    }
}

/// Text with formatting information.
#[derive(Clone, Data)]
pub struct FormattedText {
    pub plain_text: Arc<str>,
    runs: Arc<TextRuns>,
    // FIXME: multiple paragraphs?
    // FIXME: Data impl blocked on skia_safe issue
    #[data(ignore)]
    pub(crate) paragraph_style: ParagraphStyle,
}

impl<T> From<T> for FormattedText
where
    T: Into<Arc<str>>,
{
    fn from(s: T) -> Self {
        let plain_text = s.into();
        let len = plain_text.len();
        FormattedText {
            plain_text,
            runs: Arc::new(TextRuns {
                runs: vec![TextRun {
                    range: 0..len,
                    attributes: vec![],
                }],
            }),
            paragraph_style: Default::default(),
        }
    }
}

impl FormattedText {
    pub fn with_attribute(mut self, range: impl RangeBounds<usize>, attribute: impl Into<Attribute>) -> FormattedText {
        self.add_attribute(range, attribute);
        self
    }

    pub fn add_attribute(&mut self, range: impl RangeBounds<usize>, attribute: impl Into<Attribute>) {
        let range = resolve_range(range, self.plain_text.len());
        Arc::make_mut(&mut self.runs).merge_attribute(range, &attribute.into())
    }

    pub fn with_paragraph_style(mut self, style: ParagraphStyle) -> Self {
        self.set_paragraph_style(style);
        self
    }

    pub fn set_paragraph_style(&mut self, style: ParagraphStyle) {
        self.paragraph_style = style;
    }

    pub fn format(&self, width: f64) -> FormattedTextParagraph {
        let default_font_manager = sk::FontMgr::default();
        let mut font_collection = sk::textlayout::FontCollection::new();
        font_collection.set_default_font_manager(default_font_manager, "Consolas");
        let mut builder = sk::textlayout::ParagraphBuilder::new(&self.paragraph_style.0, font_collection);

        // computed text style
        let mut text_style = sk::textlayout::TextStyle::new();
        for run in self.runs.runs.iter() {
            text_style.clone_from(&self.paragraph_style.0.text_style());
            for attr in run.attributes.iter() {
                match *attr {
                    Attribute::FontSize(fs) => {
                        text_style.set_font_size(fs as sk::scalar);
                    }
                    Attribute::FontFamily(ref family) => {
                        text_style.set_font_families(&[family.0.as_ref()]);
                    }
                    Attribute::FontStyle(fs) => {
                        let current_font_style = text_style.font_style();
                        text_style.set_font_style(sk::FontStyle::new(
                            current_font_style.weight(),
                            current_font_style.width(),
                            fs.to_skia(),
                        ));
                    }
                    Attribute::FontWeight(fw) => {
                        let current_font_style = text_style.font_style();
                        text_style.set_font_style(sk::FontStyle::new(
                            sk::font_style::Weight::from(fw.0 as i32),
                            current_font_style.width(),
                            current_font_style.slant(),
                        ));
                    }
                    Attribute::Color(color) => {
                        text_style.set_color(color.to_skia().to_color());
                    }
                }
            }
            builder.push_style(&text_style);
            builder.add_text(&self.plain_text[run.range.start..run.range.end]);
            builder.pop();
        }

        let mut paragraph = builder.build();
        // layout the paragraph
        paragraph.layout(width as sk::scalar);
        FormattedTextParagraph(paragraph)
    }
}

#[derive(Clone, Debug, Default)]
pub struct TextStyle(sk::textlayout::TextStyle);

impl TextStyle {
    /// Sets the font color.
    pub fn color(mut self, color: Color) -> Self {
        self.0.set_color(color.to_skia().to_color());
        self
    }

    /// Sets the font family.
    pub fn font_family(mut self, family: &str) -> Self {
        self.0.set_font_families(&[family]);
        self
    }

    /// Sets the font size (TODO in what units?)
    pub fn font_size(mut self, size: f64) -> Self {
        self.0.set_font_size(size as sk::scalar);
        self
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct TextBox {
    pub rect: Rect,
    pub direction: sk::textlayout::TextDirection,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Data)]
pub enum TextAffinity {
    Upstream,
    Downstream,
}

impl FromSkia for TextAffinity {
    type Source = sk::textlayout::Affinity;

    fn from_skia(value: Self::Source) -> Self {
        match value {
            sk::textlayout::Affinity::Upstream => TextAffinity::Upstream,
            sk::textlayout::Affinity::Downstream => TextAffinity::Downstream,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct TextPosition {
    pub position: usize,
    pub affinity: TextAffinity,
}

// FIXME: this is useless since it's not guaranteed to be laid out
pub struct FormattedTextParagraph(pub(crate) sk::textlayout::Paragraph);

impl FormattedTextParagraph {
    /// Returns a list of enclosing rects for the specified text range.
    pub fn rects_for_range(&self, range: Range<usize>) -> Vec<TextBox> {
        let boxes = self.0.get_rects_for_range(
            range,
            sk::textlayout::RectHeightStyle::IncludeLineSpacingMiddle,
            sk::textlayout::RectWidthStyle::Tight,
        );
        boxes
            .iter()
            .map(|tb| TextBox {
                rect: Rect::from_skia(tb.rect),
                direction: tb.direct,
            })
            .collect()
    }

    /// Returns the text position closest to the given point.
    pub fn glyph_text_position(&self, point: Point) -> TextPosition {
        let tp = self.0.get_glyph_position_at_coordinate(point.to_skia());
        TextPosition {
            position: tp.position as usize,
            affinity: TextAffinity::from_skia(tp.affinity),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct ParagraphStyle(pub(crate) sk::textlayout::ParagraphStyle);

impl ParagraphStyle {
    pub fn new() -> ParagraphStyle {
        ParagraphStyle(sk::textlayout::ParagraphStyle::new())
    }

    /// Sets the default text style of this paragraph (?)
    pub fn text_style(mut self, text_style: &TextStyle) -> Self {
        self.0.set_text_style(&text_style.0);
        self
    }

    /// Sets the text alignment.
    pub fn text_alignment(mut self, align: sk::textlayout::TextAlign) -> Self {
        self.0.set_text_align(align);
        self
    }
}

impl Data for ParagraphStyle {
    fn same(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_runs() {
        let mut tr = TextRuns::new();

        tr.merge_attribute(0..10, &Attribute::FontSize(40.0));
        assert_eq!(
            tr.runs,
            vec![TextRun {
                range: 0..10,
                attributes: vec![Attribute::FontSize(40.0)]
            },]
        );

        tr.merge_attribute(1..5, &Attribute::FontStyle(FontStyle::Italic));
        assert_eq!(
            tr.runs,
            vec![
                TextRun {
                    range: 0..1,
                    attributes: vec![Attribute::FontSize(40.0)]
                },
                TextRun {
                    range: 1..5,
                    attributes: vec![Attribute::FontSize(40.0), Attribute::FontStyle(FontStyle::Italic)]
                },
                TextRun {
                    range: 5..10,
                    attributes: vec![Attribute::FontSize(40.0)]
                },
            ]
        );

        tr.merge_attribute(5..7, &Attribute::FontWeight(FontWeight::BOLD));
        assert_eq!(
            tr.runs,
            vec![
                TextRun {
                    range: 0..1,
                    attributes: vec![Attribute::FontSize(40.0)]
                },
                TextRun {
                    range: 1..5,
                    attributes: vec![Attribute::FontSize(40.0), Attribute::FontStyle(FontStyle::Italic)]
                },
                /*TextRun {
                    range: 4..5,
                    attributes: vec![
                        Attribute::FontSize(40.0),
                        Attribute::FontStyle(FontStyle::Italic),
                        Attribute::FontWeight(FontWeight::BOLD)
                    ]
                },*/
                TextRun {
                    range: 5..7,
                    attributes: vec![Attribute::FontSize(40.0), Attribute::FontWeight(FontWeight::BOLD)]
                },
                TextRun {
                    range: 7..10,
                    attributes: vec![Attribute::FontSize(40.0)]
                }
            ]
        );
    }
}
