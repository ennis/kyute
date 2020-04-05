use crate::drawing::{Point, Rect, Size};
use crate::error::{self, Result};
use crate::platform::Platform;
use std::mem::MaybeUninit;
use std::ops::{Bound, Range, RangeBounds};
use std::ptr;
use winapi::shared::minwindef::TRUE;
use winapi::shared::winerror::{ERROR_INSUFFICIENT_BUFFER, HRESULT_FROM_WIN32, SUCCEEDED};
use winapi::um::dwrite::*;
use wio::com::ComPtr;
use wio::wide::ToWide;

///
#[derive(Clone)]
pub struct TextFormat(ComPtr<IDWriteTextFormat>);

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

impl Into<DWRITE_FONT_WEIGHT> for FontWeight {
    fn into(self) -> DWRITE_FONT_WEIGHT {
        match self {
            FontWeight::Thin => DWRITE_FONT_WEIGHT_THIN,
            FontWeight::ExtraLight => DWRITE_FONT_WEIGHT_EXTRA_LIGHT,
            FontWeight::UltraLight => DWRITE_FONT_WEIGHT_ULTRA_LIGHT,
            FontWeight::Light => DWRITE_FONT_WEIGHT_LIGHT,
            FontWeight::SemiLight => DWRITE_FONT_WEIGHT_SEMI_LIGHT,
            FontWeight::Normal => DWRITE_FONT_WEIGHT_NORMAL,
            FontWeight::Regular => DWRITE_FONT_WEIGHT_REGULAR,
            FontWeight::Medium => DWRITE_FONT_WEIGHT_MEDIUM,
            FontWeight::DemiBold => DWRITE_FONT_WEIGHT_DEMI_BOLD,
            FontWeight::SemiBold => DWRITE_FONT_WEIGHT_SEMI_BOLD,
            FontWeight::Bold => DWRITE_FONT_WEIGHT_BOLD,
            FontWeight::ExtraBold => DWRITE_FONT_WEIGHT_EXTRA_BOLD,
            FontWeight::UltraBold => DWRITE_FONT_WEIGHT_ULTRA_BOLD,
            FontWeight::Black => DWRITE_FONT_WEIGHT_BLACK,
            FontWeight::Heavy => DWRITE_FONT_WEIGHT_HEAVY,
            FontWeight::ExtraBlack => DWRITE_FONT_WEIGHT_EXTRA_BLACK,
            FontWeight::UltraBlack => DWRITE_FONT_WEIGHT_ULTRA_BLACK,
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

impl Into<DWRITE_FONT_STYLE> for FontStyle {
    fn into(self) -> DWRITE_FONT_STYLE {
        match self {
            FontStyle::Normal => DWRITE_FONT_STYLE_NORMAL,
            FontStyle::Oblique => DWRITE_FONT_STYLE_OBLIQUE,
            FontStyle::Italic => DWRITE_FONT_STYLE_ITALIC,
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

impl Into<DWRITE_FONT_STRETCH> for FontStretch {
    fn into(self) -> DWRITE_FONT_STRETCH {
        match self {
            FontStretch::Undefined => DWRITE_FONT_STRETCH_UNDEFINED,
            FontStretch::UltraCondensed => DWRITE_FONT_STRETCH_ULTRA_CONDENSED,
            FontStretch::ExtraCondensed => DWRITE_FONT_STRETCH_EXTRA_CONDENSED,
            FontStretch::Condensed => DWRITE_FONT_STRETCH_CONDENSED,
            FontStretch::SemiCondensed => DWRITE_FONT_STRETCH_SEMI_CONDENSED,
            FontStretch::Normal => DWRITE_FONT_STRETCH_NORMAL,
            FontStretch::Medium => DWRITE_FONT_STRETCH_MEDIUM,
            FontStretch::SemiExpanded => DWRITE_FONT_STRETCH_SEMI_EXPANDED,
            FontStretch::Expanded => DWRITE_FONT_STRETCH_EXPANDED,
            FontStretch::ExtraExpanded => DWRITE_FONT_STRETCH_EXTRA_EXPANDED,
            FontStretch::UltraExpanded => DWRITE_FONT_STRETCH_ULTRA_EXPANDED,
        }
    }
}

pub struct TextFormatBuilder<'a> {
    factory: &'a ComPtr<IDWriteFactory>,
    family: &'a str,
    weight: FontWeight,
    style: FontStyle,
    stretch: FontStretch,
    size: f32,
}

impl<'a> TextFormatBuilder<'a> {
    pub fn new(platform: &'a Platform) -> TextFormatBuilder<'a> {
        TextFormatBuilder {
            factory: &platform.0.dwrite_factory,
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
        let family = self.family.to_wide_null();
        let locale = "en-US".to_wide_null();

        unsafe {
            let mut ptr = ptr::null_mut();
            let hr = self.factory.CreateTextFormat(
                family.as_ptr(),
                ptr::null_mut(), // collection
                self.weight.into(),
                self.style.into(),
                self.stretch.into(),
                self.size,
                locale.as_ptr(),
                &mut ptr,
            );

            error::wrap_hr(hr, || TextFormat(ComPtr::from_raw(ptr)))
        }
    }
}

#[derive(Copy, Clone, Debug)]
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

#[derive(Copy, Clone, Debug)]
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
            is_trimmed: m.isTrimmed != 0,
        }
    }
}

/// Text hit-test metrics.
pub struct HitTestMetrics {
    /// Text position in UTF-8 code units (bytes).
    pub text_position: usize,
    pub length: u32,
    pub bounds: Rect,
}

impl HitTestMetrics {
    pub(crate) fn from_dwrite(metrics: &DWRITE_HIT_TEST_METRICS, text: &str) -> HitTestMetrics {
        // convert utf16 code unit offset to utf8
        let text_position =
            count_until_utf16(text, metrics.textPosition as usize).expect("invalid UTF-16 offset");
        HitTestMetrics {
            text_position,
            length: metrics.length,
            bounds: Rect::new(
                Point::new(metrics.left as f64, metrics.top as f64),
                Size::new(metrics.width as f64, metrics.height as f64),
            ),
        }
    }
}

/// Return value of [TextLayout::hit_test_text_position].
pub struct HitTestTextPosition {
    pub point: Point,
    pub metrics: HitTestMetrics,
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
pub(crate) fn count_until_utf16(s: &str, utf16_text_position: usize) -> Option<usize> {
    let mut utf8_count = 0;
    let mut utf16_count = 0;
    for &b in s.as_bytes() {
        if (b as i8) >= -0x40 {
            utf16_count += 1;
        }
        if b >= 0xf0 {
            utf16_count += 1;
        }

        if utf16_count > utf16_text_position {
            return Some(utf8_count);
        }

        utf8_count += 1;
    }

    None
}

/// Text layout.
#[derive(Clone)]
pub struct TextLayout {
    ptr: ComPtr<IDWriteTextLayout>,
    text: String,
}

impl TextLayout {
    pub fn new(
        platform: &Platform,
        text: &str,
        format: &TextFormat,
        layout_box_size: Size,
    ) -> Result<TextLayout> {
        let wtext = text.to_wide();

        unsafe {
            let mut ptr = ptr::null_mut();
            let hr = platform.0.dwrite_factory.CreateTextLayout(
                wtext.as_ptr(),
                wtext.len() as u32,
                format.0.as_raw(),
                layout_box_size.width as f32,
                layout_box_size.height as f32,
                &mut ptr,
            );

            error::wrap_hr(hr, || TextLayout {
                ptr: ComPtr::from_raw(ptr),
                text: text.to_owned(),
            })
        }
    }

    pub fn hit_test_point(&self, point: Point) -> Result<HitTestMetrics> {
        unsafe {
            let mut is_trailing_hit = 0;
            let mut is_inside = 0;
            let mut metrics = MaybeUninit::<DWRITE_HIT_TEST_METRICS>::uninit();
            let hr = self.ptr.HitTestPoint(
                point.x as f32,
                point.y as f32,
                &mut is_trailing_hit,
                &mut is_inside,
                metrics.as_mut_ptr(),
            );

            error::wrap_hr(hr, || {
                HitTestMetrics::from_dwrite(&metrics.assume_init(), &self.text)
            })
        }
    }

    /// Returns the layout maximum size.
    pub fn max_size(&self) -> Size {
        unsafe {
            let w = self.ptr.GetMaxWidth();
            let h = self.ptr.GetMaxHeight();
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
            let hr = self.ptr.HitTestTextPosition(
                pos_utf16 as u32,
                TRUE,
                &mut point_x,
                &mut point_y,
                metrics.as_mut_ptr(),
            );

            error::wrap_hr(hr, || HitTestTextPosition {
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
            let hr = self.ptr.GetMetrics(text_metrics.as_mut_ptr());
            assert!(SUCCEEDED(hr));
            let text_metrics = text_metrics.assume_init();

            // "A good value to use as an initial value for maxHitTestMetricsCount
            // may be calculated from the following equation:
            // maxHitTestMetricsCount = lineCount * maxBidiReorderingDepth"
            // (https://docs.microsoft.com/en-us/windows/win32/api/dwrite/nf-dwrite-idwritetextlayout-hittesttextrange)
            let mut max_metrics_count =
                text_metrics.lineCount * text_metrics.maxBidiReorderingDepth;
            let mut actual_metrics_count = 0;
            let mut metrics = Vec::with_capacity(max_metrics_count as usize);

            let mut hr = self.ptr.HitTestTextRange(
                text_position as u32,
                text_length as u32,
                origin.x as f32,
                origin.y as f32,
                metrics.as_mut_ptr(),
                max_metrics_count,
                &mut actual_metrics_count,
            );
            if hr == HRESULT_FROM_WIN32(ERROR_INSUFFICIENT_BUFFER) {
                // reallocate with sufficient space
                metrics = Vec::with_capacity(actual_metrics_count as usize);
                max_metrics_count = actual_metrics_count;
                hr = self.ptr.HitTestTextRange(
                    text_position as u32,
                    text_length as u32,
                    origin.x as f32,
                    origin.y as f32,
                    metrics.as_mut_ptr(),
                    max_metrics_count,
                    &mut actual_metrics_count,
                );
            }

            error::wrap_hr(hr, || {
                metrics.set_len(actual_metrics_count as usize);
                metrics
                    .into_iter()
                    .map(|m| HitTestMetrics::from_dwrite(&m, &self.text))
                    .collect()
            })
        }
    }

    pub fn metrics(&self) -> TextMetrics {
        unsafe {
            let mut metrics = MaybeUninit::<DWRITE_TEXT_METRICS>::uninit();
            let hr = self.ptr.GetMetrics(metrics.as_mut_ptr());
            assert!(SUCCEEDED(hr));
            metrics.assume_init().into()
        }
    }

    pub fn line_metrics(&self) -> Vec<LineMetrics> {
        unsafe {
            let mut line_count = 1;
            let mut metrics = Vec::with_capacity(line_count as usize);
            let mut hr = self
                .ptr
                .GetLineMetrics(metrics.as_mut_ptr(), line_count, &mut line_count);

            if hr == HRESULT_FROM_WIN32(ERROR_INSUFFICIENT_BUFFER) {
                // reallocate with sufficient space
                metrics = Vec::with_capacity(line_count as usize);
                hr = self
                    .ptr
                    .GetLineMetrics(metrics.as_mut_ptr(), line_count, &mut line_count);
            }

            assert!(SUCCEEDED(hr));

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

        let start = count_utf16(&self.text[0..start]);
        let len = count_utf16(&self.text[start..end]);

        DWRITE_TEXT_RANGE {
            startPosition: start as u32,
            length: len as u32,
        }
    }

    pub fn set_font_weight<R>(&mut self, weight: FontWeight, range: R)
    where
        R: RangeBounds<usize>,
    {
        let range = self.to_utf16_text_range(range);
        unsafe {
            self.ptr.SetFontWeight(weight.into(), range);
        }
    }

    pub fn as_raw(&self) -> *mut IDWriteTextLayout {
        self.ptr.as_raw()
    }
}
