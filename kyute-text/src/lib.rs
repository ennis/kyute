pub(crate) mod factory;
mod formatted_text;
mod paragraph;

use kyute_common::{Color, Data, Point, Rect};
use lazy_static::lazy_static;
use std::{
    cmp::Ordering,
    ops::{Bound, Deref, Range, RangeBounds},
    sync::Arc,
};
use windows::Win32::Graphics::DirectWrite::{
    DWriteCreateFactory, IDWriteFactory, IDWriteFactory7, IDWriteTextLayout3, DWRITE_FACTORY_TYPE_SHARED,
    DWRITE_FONT_STYLE, DWRITE_FONT_STYLE_ITALIC, DWRITE_FONT_STYLE_NORMAL, DWRITE_FONT_STYLE_OBLIQUE,
    DWRITE_FONT_WEIGHT, DWRITE_FONT_WEIGHT_EXTRA_BLACK, DWRITE_PARAGRAPH_ALIGNMENT, DWRITE_TEXT_ALIGNMENT,
    DWRITE_TEXT_ALIGNMENT_CENTER, DWRITE_TEXT_ALIGNMENT_JUSTIFIED, DWRITE_TEXT_ALIGNMENT_LEADING,
    DWRITE_TEXT_ALIGNMENT_TRAILING,
};

pub use formatted_text::FormattedText;
pub use paragraph::{
    FontFace, GlyphMaskData, GlyphMaskFormat, GlyphOffset, GlyphRun, GlyphRunAnalysis, GlyphRunDrawingEffects,
    HitTestMetrics, HitTestPoint, HitTestTextPosition, LineMetrics, Paragraph, RasterizationOptions, Renderer,
    TextMetrics,
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("OS error")]
    OsError(#[from] windows::core::Error),
}

trait ToDirectWrite {
    type Target;
    fn to_dwrite(&self) -> Self::Target;
}

/// Text selection.
///
/// Start is the start of the selection, end is the end. The caret is at the end of the selection.
/// Note that we don't necessarily have start <= end: a selection with start > end means that the
/// user started the selection gesture from a later point in the text and then went back
/// (right-to-left in LTR languages). In this case, the cursor will appear at the "beginning"
/// (i.e. left, for LTR) of the selection.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Data)]
pub struct Selection {
    pub start: usize,
    pub end: usize,
}

impl Selection {
    pub fn min(&self) -> usize {
        self.start.min(self.end)
    }
    pub fn max(&self) -> usize {
        self.start.max(self.end)
    }
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }
    pub fn empty(at: usize) -> Selection {
        Selection { start: at, end: at }
    }
}

impl Default for Selection {
    fn default() -> Self {
        Selection::empty(0)
    }
}

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

/// From [piet-direct2d](https://github.com/linebender/piet/blob/master/piet-direct2d/src/text.rs):
/// Counts the number of utf-16 code units in the given string.
/// from xi-editor
pub(crate) fn count_utf16(s: &str) -> usize {
    let mut utf16_count = 0;
    for &b in s.as_bytes() {
        if (b as i8) >= -0x40 {
            utf16_count += 1;
        }
        if b >= 0xf0 {
            utf16_count += 1;
        }
    }
    utf16_count
}

/// From [piet-direct2d](https://github.com/linebender/piet/blob/master/piet-direct2d/src/text.rs):
/// returns utf8 text position (code unit offset)
/// at the given utf-16 text position
pub(crate) fn count_until_utf16(s: &str, utf16_text_position: usize) -> usize {
    let mut utf16_count = 0;

    for (i, c) in s.char_indices() {
        utf16_count += c.len_utf16();
        if utf16_count > utf16_text_position {
            return i;
        }
    }

    s.len()
}

trait ToWString {
    fn to_wstring(&self) -> Vec<u16>;
}

impl ToWString for str {
    fn to_wstring(&self) -> Vec<u16> {
        self.encode_utf16().chain(std::iter::once(0)).collect()
    }
}

/// Describes a font weight.
///
/// It is a value between 1 and 1000, based on the CSS [`font-weight`](https://developer.mozilla.org/en-US/docs/Web/CSS/font-weight) property.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Data)]
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

impl ToDirectWrite for FontWeight {
    type Target = DWRITE_FONT_WEIGHT;
    fn to_dwrite(&self) -> Self::Target {
        DWRITE_FONT_WEIGHT(self.0 as i32)
    }
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
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Data)]
pub enum FontStyle {
    Normal,
    Italic,
    Oblique,
}

impl ToDirectWrite for FontStyle {
    type Target = DWRITE_FONT_STYLE;
    fn to_dwrite(&self) -> Self::Target {
        match *self {
            FontStyle::Normal => DWRITE_FONT_STYLE_NORMAL,
            FontStyle::Italic => DWRITE_FONT_STYLE_ITALIC,
            FontStyle::Oblique => DWRITE_FONT_STYLE_OBLIQUE,
        }
    }
}

/// Text alignment within a text paragraph.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Data)]
pub enum TextAlignment {
    Leading,
    Trailing,
    Center,
    Justified,
}

impl Default for TextAlignment {
    fn default() -> Self {
        TextAlignment::Leading
    }
}

impl ToDirectWrite for TextAlignment {
    type Target = DWRITE_TEXT_ALIGNMENT;
    fn to_dwrite(&self) -> Self::Target {
        match *self {
            TextAlignment::Leading => DWRITE_TEXT_ALIGNMENT_LEADING,
            TextAlignment::Trailing => DWRITE_TEXT_ALIGNMENT_TRAILING,
            TextAlignment::Center => DWRITE_TEXT_ALIGNMENT_CENTER,
            TextAlignment::Justified => DWRITE_TEXT_ALIGNMENT_JUSTIFIED,
        }
    }
}

/// Attributes that can be applied to text.
#[derive(Clone, Debug, PartialEq)]
pub enum Attribute {
    /// Font size in DIPs (1/96 inch).
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

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Data)]
pub enum TextAffinity {
    Upstream,
    Downstream,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Data)]
pub struct TextPosition {
    pub position: usize,
    pub affinity: TextAffinity,
}

/*
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
}*/

#[cfg(test)]
mod tests {
    use crate::{
        formatted_text::{TextRun, TextRuns},
        Attribute, FontStyle, FontWeight,
    };

    #[test]
    fn test_text_runs() {
        let mut tr = TextRuns { runs: vec![] };

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
