mod formatted_text;
mod paragraph;

pub use formatted_text::{FormattedText, FormattedTextExt, ParagraphStyle};
pub use paragraph::{
    GlyphRun, GlyphRunAnalysis, GlyphRunDrawingEffects, HitTestMetrics, HitTestPoint, HitTestTextPosition, LineMetrics,
    Paragraph, Renderer, TextMetrics,
};

use kyute_common::{Color, Data, SizeI};
use std::{
    ops::{Bound, Range, RangeBounds},
    sync::Arc,
};

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
pub(crate) fn resolve_range(range: impl RangeBounds<usize>, len: usize) -> Range<usize> {
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
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Data)]
#[repr(transparent)]
pub struct FontWeight(pub u16);

impl Default for FontWeight {
    fn default() -> Self {
        FontWeight::NORMAL
    }
}

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
pub struct FontFamily(pub(crate) Arc<str>);

impl FontFamily {
    pub fn new(name: impl Into<Arc<str>>) -> FontFamily {
        FontFamily(name.into())
    }

    pub fn name(&self) -> &str {
        &self.0
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

impl Default for FontStyle {
    fn default() -> Self {
        FontStyle::Normal
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

#[derive(Copy, Clone, Debug)]
pub struct GlyphOffset {
    pub advance_offset: f32,
    pub ascender_offset: f32,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum RasterizationOptions {
    Bilevel,
    Grayscale,
    Subpixel,
}

/// Format of a rasterized glyph mask.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum GlyphMaskFormat {
    // 3 bytes per pixel, RGB subpixel mask
    Rgb8,
    // one byte per pixel, alpha mask
    Gray8,
}

/// Pixel data of a rasterized glyph run.
#[derive(Debug)]
pub struct GlyphMaskData {
    pub size: SizeI,
    pub format: GlyphMaskFormat,
    pub data: Vec<u8>,
}

/*impl GlyphMaskData {
    /// Returns the size of this mask.
    pub fn size(&self) -> SizeI {
        self.size
    }

    /// Returns a reference to the pixel data.
    pub fn data(&self) -> &[u8] {
        self.data.as_slice()
    }

    /// Returns the format of the mask data.
    pub fn format(&self) -> GlyphMaskFormat {
        self.format
    }
}*/

#[cfg(test)]
mod tests {
    use super::{
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
