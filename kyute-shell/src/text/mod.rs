//! Platform text services
use crate::{
    drawing::{Brush, Point, Rect, Size},
    error::Result,
    platform::Platform,
};
use std::{
    mem::MaybeUninit,
    ops::{Bound, Range, RangeBounds},
};

use crate::bindings::Windows::Win32::{
    Debug::WIN32_ERROR,
    DirectWrite::{
        IDWriteTextFormat, IDWriteTextLayout, DWRITE_FONT_STRETCH, DWRITE_FONT_STYLE,
        DWRITE_FONT_WEIGHT, DWRITE_HIT_TEST_METRICS, DWRITE_LINE_METRICS, DWRITE_TEXT_METRICS,
        DWRITE_TEXT_RANGE,
    },
    SystemServices::{BOOL, PWSTR},
};
use windows::{IUnknown, Interface, HRESULT};

/// Text drawing effects.
pub trait DrawingEffect {
    fn to_iunknown(&self) -> IUnknown;
}

impl DrawingEffect for Brush {
    fn to_iunknown(&self) -> IUnknown {
        self.to_base_brush().cast().unwrap()
    }
}

/// Font weight
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum FontWeight {
    Thin,
    ExtraLight,
    UltraLight,
    Light,
    SemiLight,
    Normal,
    Regular,
    Medium,
    DemiBold,
    SemiBold,
    Bold,
    ExtraBold,
    UltraBold,
    Black,
    Heavy,
    ExtraBlack,
    UltraBlack,
}

impl FontWeight {
    fn to_dwrite(self) -> DWRITE_FONT_WEIGHT {
        match self {
            FontWeight::Thin => DWRITE_FONT_WEIGHT::DWRITE_FONT_WEIGHT_THIN,
            FontWeight::ExtraLight => DWRITE_FONT_WEIGHT::DWRITE_FONT_WEIGHT_EXTRA_LIGHT,
            FontWeight::UltraLight => DWRITE_FONT_WEIGHT::DWRITE_FONT_WEIGHT_ULTRA_LIGHT,
            FontWeight::Light => DWRITE_FONT_WEIGHT::DWRITE_FONT_WEIGHT_LIGHT,
            FontWeight::SemiLight => DWRITE_FONT_WEIGHT::DWRITE_FONT_WEIGHT_SEMI_LIGHT,
            FontWeight::Normal => DWRITE_FONT_WEIGHT::DWRITE_FONT_WEIGHT_NORMAL,
            FontWeight::Regular => DWRITE_FONT_WEIGHT::DWRITE_FONT_WEIGHT_REGULAR,
            FontWeight::Medium => DWRITE_FONT_WEIGHT::DWRITE_FONT_WEIGHT_MEDIUM,
            FontWeight::DemiBold => DWRITE_FONT_WEIGHT::DWRITE_FONT_WEIGHT_DEMI_BOLD,
            FontWeight::SemiBold => DWRITE_FONT_WEIGHT::DWRITE_FONT_WEIGHT_SEMI_BOLD,
            FontWeight::Bold => DWRITE_FONT_WEIGHT::DWRITE_FONT_WEIGHT_BOLD,
            FontWeight::ExtraBold => DWRITE_FONT_WEIGHT::DWRITE_FONT_WEIGHT_EXTRA_BOLD,
            FontWeight::UltraBold => DWRITE_FONT_WEIGHT::DWRITE_FONT_WEIGHT_ULTRA_BOLD,
            FontWeight::Black => DWRITE_FONT_WEIGHT::DWRITE_FONT_WEIGHT_BLACK,
            FontWeight::Heavy => DWRITE_FONT_WEIGHT::DWRITE_FONT_WEIGHT_HEAVY,
            FontWeight::ExtraBlack => DWRITE_FONT_WEIGHT::DWRITE_FONT_WEIGHT_EXTRA_BLACK,
            FontWeight::UltraBlack => DWRITE_FONT_WEIGHT::DWRITE_FONT_WEIGHT_ULTRA_BLACK,
        }
    }
}

/// Font style.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum FontStyle {
    Normal,
    Oblique,
    Italic,
}

impl FontStyle {
    fn to_dwrite(self) -> DWRITE_FONT_STYLE {
        match self {
            FontStyle::Normal => DWRITE_FONT_STYLE::DWRITE_FONT_STYLE_NORMAL,
            FontStyle::Oblique => DWRITE_FONT_STYLE::DWRITE_FONT_STYLE_OBLIQUE,
            FontStyle::Italic => DWRITE_FONT_STYLE::DWRITE_FONT_STYLE_ITALIC,
        }
    }
}

/// Font stretch.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum FontStretch {
    Undefined,
    UltraCondensed,
    ExtraCondensed,
    Condensed,
    SemiCondensed,
    Normal,
    Medium,
    SemiExpanded,
    Expanded,
    ExtraExpanded,
    UltraExpanded,
}

impl FontStretch {
    fn to_dwrite(self) -> DWRITE_FONT_STRETCH {
        match self {
            FontStretch::Undefined => DWRITE_FONT_STRETCH::DWRITE_FONT_STRETCH_UNDEFINED,
            FontStretch::UltraCondensed => DWRITE_FONT_STRETCH::DWRITE_FONT_STRETCH_ULTRA_CONDENSED,
            FontStretch::ExtraCondensed => DWRITE_FONT_STRETCH::DWRITE_FONT_STRETCH_EXTRA_CONDENSED,
            FontStretch::Condensed => DWRITE_FONT_STRETCH::DWRITE_FONT_STRETCH_CONDENSED,
            FontStretch::SemiCondensed => DWRITE_FONT_STRETCH::DWRITE_FONT_STRETCH_SEMI_CONDENSED,
            FontStretch::Normal => DWRITE_FONT_STRETCH::DWRITE_FONT_STRETCH_NORMAL,
            FontStretch::Medium => DWRITE_FONT_STRETCH::DWRITE_FONT_STRETCH_MEDIUM,
            FontStretch::SemiExpanded => DWRITE_FONT_STRETCH::DWRITE_FONT_STRETCH_SEMI_EXPANDED,
            FontStretch::Expanded => DWRITE_FONT_STRETCH::DWRITE_FONT_STRETCH_EXPANDED,
            FontStretch::ExtraExpanded => DWRITE_FONT_STRETCH::DWRITE_FONT_STRETCH_EXTRA_EXPANDED,
            FontStretch::UltraExpanded => DWRITE_FONT_STRETCH::DWRITE_FONT_STRETCH_ULTRA_EXPANDED,
        }
    }
}

/*#[derive(Copy, Clone, Debug, PartialEq)]
pub struct TextFormatDesc<'a> {
    family: &'a str,
    weight: FontWeight,
    style: FontStyle,
    stretch: FontStretch,
    size: f32,
}*/

/// Text formatting options.
#[derive(Clone)]
pub struct TextFormat(IDWriteTextFormat);

impl TextFormat {
    /// Creates a new `TextFormatBuilder` to build a `TextFormat`.
    pub fn builder<'a>() -> TextFormatBuilder<'a> {
        TextFormatBuilder::new()
    }

    pub fn as_raw(&self) -> &IDWriteTextFormat {
        &self.0
    }

    /// Returns the font size in DIPs.
    pub fn font_size(&self) -> f32 {
        unsafe { self.0.GetFontSize() }
    }
}

/// Builder pattern for `TextFormat`.
pub struct TextFormatBuilder<'a> {
    family: &'a str,
    weight: FontWeight,
    style: FontStyle,
    stretch: FontStretch,
    size: f32,
}

impl<'a> TextFormatBuilder<'a> {
    pub fn new() -> TextFormatBuilder<'a> {
        TextFormatBuilder {
            family: "",
            weight: FontWeight::Normal,
            style: FontStyle::Normal,
            stretch: FontStretch::Normal,
            size: 12.0,
        }
    }

    pub fn size(mut self, size: f32) -> TextFormatBuilder<'a> {
        self.size = size;
        self
    }

    pub fn family(mut self, family: &'a str) -> TextFormatBuilder<'a> {
        self.family = family;
        self
    }

    pub fn build(self) -> Result<TextFormat> {
        let platform = Platform::instance();

        unsafe {
            let mut text_format = None;
            let text_format = platform
                .0
                .dwrite_factory
                .CreateTextFormat(
                    self.family,
                    None, // collection
                    self.weight.to_dwrite(),
                    self.style.to_dwrite(),
                    self.stretch.to_dwrite(),
                    self.size,
                    "en-US", // TODO
                    &mut text_format,
                )
                .and_some(text_format)?;
            Ok(TextFormat(text_format))
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct TextMetrics {
    pub bounds: Rect,
    pub width_including_trailing_whitespace: f32,
    pub line_count: u32,
    pub max_bidi_reordering_depth: u32,
}

impl From<DWRITE_TEXT_METRICS> for TextMetrics {
    fn from(m: DWRITE_TEXT_METRICS) -> Self {
        TextMetrics {
            bounds: Rect::new(
                Point::new(m.left as f64, m.top as f64),
                Size::new(m.width as f64, m.height as f64),
            ),
            width_including_trailing_whitespace: m.widthIncludingTrailingWhitespace,
            max_bidi_reordering_depth: m.maxBidiReorderingDepth,
            line_count: m.lineCount,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct LineMetrics {
    pub length: u32,
    pub trailing_whitespace_length: u32,
    pub newline_length: u32,
    pub height: f64,
    pub baseline: f64,
    pub is_trimmed: bool,
}

impl From<DWRITE_LINE_METRICS> for LineMetrics {
    fn from(m: DWRITE_LINE_METRICS) -> Self {
        LineMetrics {
            length: m.length,
            trailing_whitespace_length: m.trailingWhitespaceLength,
            newline_length: m.newlineLength,
            height: m.height as f64,
            baseline: m.baseline as f64,
            is_trimmed: m.isTrimmed.as_bool(),
        }
    }
}

/// From [piet-direct2d](https://github.com/linebender/piet/blob/master/piet-direct2d/src/text.rs):
/// Counts the number of utf-16 code units in the given string.
/// from xi-editor
fn count_utf16(s: &str) -> usize {
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
fn count_until_utf16(s: &str, utf16_text_position: usize) -> usize {
    let mut utf16_count = 0;

    for (i, c) in s.char_indices() {
        utf16_count += c.len_utf16();
        if utf16_count > utf16_text_position {
            return i;
        }
    }

    s.len()
}

/// Text hit-test metrics.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct HitTestMetrics {
    /// Text position in UTF-8 code units (bytes).
    pub text_position: usize,
    pub length: usize,
    pub bounds: Rect,
}

impl HitTestMetrics {
    pub(crate) fn from_dwrite(metrics: &DWRITE_HIT_TEST_METRICS, text: &str) -> HitTestMetrics {
        // convert utf16 code unit offset to utf8
        //dbg!(metrics.textPosition);
        let text_position = count_until_utf16(text, metrics.textPosition as usize);
        let length = count_until_utf16(&text[text_position..], metrics.length as usize);
        HitTestMetrics {
            text_position,
            length,
            bounds: Rect::new(
                Point::new(metrics.left as f64, metrics.top as f64),
                Size::new(metrics.width as f64, metrics.height as f64),
            ),
        }
    }
}

/// Return value of [TextLayout::hit_test_point].
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct HitTestPoint {
    pub is_trailing_hit: bool,
    pub metrics: HitTestMetrics,
}

/// Return value of [TextLayout::hit_test_text_position].
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct HitTestTextPosition {
    pub point: Point,
    pub metrics: HitTestMetrics,
}

/// Text layout.
#[derive(Clone)]
pub struct TextLayout {
    text_layout: IDWriteTextLayout,
    text: String,
}

impl TextLayout {
    pub fn new(text: &str, format: &TextFormat, layout_box_size: Size) -> Result<TextLayout> {
        let platform = Platform::instance();
        let mut wtext: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();

        unsafe {
            let mut text_layout = None;
            let text_layout = platform
                .0
                .dwrite_factory
                .CreateTextLayout(
                    PWSTR(wtext.as_mut_ptr()), // oversight?
                    wtext.len() as u32,
                    &format.0,
                    layout_box_size.width as f32,
                    layout_box_size.height as f32,
                    &mut text_layout,
                )
                .and_some(text_layout)?;
            Ok(TextLayout {
                text_layout,
                text: text.to_owned(),
            })
        }
    }

    pub fn hit_test_point(&self, point: Point) -> Result<HitTestPoint> {
        unsafe {
            let mut is_trailing_hit = BOOL::default();
            let mut is_inside = BOOL::default();
            let mut metrics = MaybeUninit::<DWRITE_HIT_TEST_METRICS>::uninit();
            self.text_layout
                .HitTestPoint(
                    point.x as f32,
                    point.y as f32,
                    &mut is_trailing_hit,
                    &mut is_inside,
                    metrics.as_mut_ptr(),
                )
                .ok()?;

            Ok(HitTestPoint {
                is_trailing_hit: is_trailing_hit.as_bool(),
                metrics: HitTestMetrics::from_dwrite(&metrics.assume_init(), &self.text),
            })
        }
    }

    /// Returns the layout maximum size.
    pub fn max_size(&self) -> Size {
        unsafe {
            let w = self.text_layout.GetMaxWidth();
            let h = self.text_layout.GetMaxHeight();
            Size::new(w as f64, h as f64)
        }
    }

    pub fn hit_test_text_position(&self, text_position: usize) -> Result<HitTestTextPosition> {
        // convert the text position to an utf-16 offset (inspired by piet-direct2d).
        let pos_utf16 = count_utf16(&self.text[0..text_position]);

        unsafe {
            let mut point_x = 0.0f32;
            let mut point_y = 0.0f32;
            let mut metrics = MaybeUninit::<DWRITE_HIT_TEST_METRICS>::uninit();
            self.text_layout
                .HitTestTextPosition(
                    pos_utf16 as u32,
                    false,
                    &mut point_x,
                    &mut point_y,
                    metrics.as_mut_ptr(),
                )
                .ok()?;

            Ok(HitTestTextPosition {
                metrics: HitTestMetrics::from_dwrite(&metrics.assume_init(), &self.text),
                point: Point::new(point_x as f64, point_y as f64),
            })
        }
    }

    pub fn hit_test_text_range(
        &self,
        text_range: Range<usize>,
        origin: &Point,
    ) -> Result<Vec<HitTestMetrics>> {
        unsafe {
            // convert range to UTF16
            let text_position = count_utf16(&self.text[0..text_range.start]);
            let text_length = count_utf16(&self.text[text_range]);

            // first call to determine the count
            let mut text_metrics = MaybeUninit::<DWRITE_TEXT_METRICS>::uninit();
            self.text_layout
                .GetMetrics(text_metrics.as_mut_ptr())
                .ok()?;
            let text_metrics = text_metrics.assume_init();

            // "A good value to use as an initial value for maxHitTestMetricsCount
            // may be calculated from the following equation:
            // maxHitTestMetricsCount = lineCount * maxBidiReorderingDepth"
            // (https://docs.microsoft.com/en-us/windows/win32/api/dwrite/nf-dwrite-idwritetextlayout-hittesttextrange)
            let mut max_metrics_count =
                text_metrics.lineCount * text_metrics.maxBidiReorderingDepth;
            let mut actual_metrics_count = 0;
            let mut metrics = Vec::with_capacity(max_metrics_count as usize);

            let hr = self.text_layout.HitTestTextRange(
                text_position as u32,
                text_length as u32,
                origin.x as f32,
                origin.y as f32,
                metrics.as_mut_ptr(),
                max_metrics_count,
                &mut actual_metrics_count,
            );
            if hr == HRESULT::from_win32(WIN32_ERROR::ERROR_INSUFFICIENT_BUFFER.0) {
                // reallocate with sufficient space
                metrics = Vec::with_capacity(actual_metrics_count as usize);
                max_metrics_count = actual_metrics_count;
                self.text_layout
                    .HitTestTextRange(
                        text_position as u32,
                        text_length as u32,
                        origin.x as f32,
                        origin.y as f32,
                        metrics.as_mut_ptr(),
                        max_metrics_count,
                        &mut actual_metrics_count,
                    )
                    .ok()?;
            }

            metrics.set_len(actual_metrics_count as usize);
            Ok(metrics
                .into_iter()
                .map(|m| HitTestMetrics::from_dwrite(&m, &self.text))
                .collect())
        }
    }

    pub fn metrics(&self) -> TextMetrics {
        unsafe {
            let mut metrics = MaybeUninit::<DWRITE_TEXT_METRICS>::uninit();
            self.text_layout.GetMetrics(metrics.as_mut_ptr()).unwrap();
            metrics.assume_init().into()
        }
    }

    pub fn line_metrics(&self) -> Vec<LineMetrics> {
        unsafe {
            let mut line_count = 1;
            let mut metrics = Vec::with_capacity(line_count as usize);
            let hr =
                self.text_layout
                    .GetLineMetrics(metrics.as_mut_ptr(), line_count, &mut line_count);

            if hr == HRESULT::from_win32(WIN32_ERROR::ERROR_INSUFFICIENT_BUFFER.0) {
                // reallocate with sufficient space
                metrics = Vec::with_capacity(line_count as usize);
                self.text_layout
                    .GetLineMetrics(metrics.as_mut_ptr(), line_count, &mut line_count)
                    .unwrap();
            }

            metrics.set_len(line_count as usize);
            metrics.into_iter().map(|m| m.into()).collect()
        }
    }

    /// Returns (start, len).
    fn to_utf16_text_range<R>(&self, range: R) -> DWRITE_TEXT_RANGE
    where
        R: RangeBounds<usize>,
    {
        let start = match range.start_bound() {
            Bound::Included(&n) => n,
            Bound::Excluded(&n) => n + 1,
            Bound::Unbounded => 0,
        };
        let end = match range.end_bound() {
            Bound::Included(&n) => n + 1,
            Bound::Excluded(&n) => n,
            Bound::Unbounded => self.text.len(),
        };

        let utf16_start = count_utf16(&self.text[0..start]);
        let utf16_len = count_utf16(&self.text[start..end]);

        DWRITE_TEXT_RANGE {
            startPosition: utf16_start as u32,
            length: utf16_len as u32,
        }
    }

    pub fn set_font_weight<R>(&mut self, weight: FontWeight, range: R)
    where
        R: RangeBounds<usize>,
    {
        let range = self.to_utf16_text_range(range);
        unsafe {
            self.text_layout
                .SetFontWeight(weight.to_dwrite(), range)
                .unwrap();
        }
    }

    pub fn set_drawing_effect<R>(&mut self, effect: &impl DrawingEffect, range: R)
    where
        R: RangeBounds<usize>,
    {
        let range = self.to_utf16_text_range(range);
        unsafe {
            self.text_layout
                .SetDrawingEffect(effect.to_iunknown(), range)
                .unwrap();
        }
    }

    pub fn as_raw(&self) -> &IDWriteTextLayout {
        &self.text_layout
    }
}
