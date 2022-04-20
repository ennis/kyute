mod paragraph;

use crate::{
    application::Application,
    text::{FontStyle, FontWeight, TextAlignment},
};
use kyute_common::Transform;
pub use paragraph::{GlyphRun, GlyphRunAnalysis, Paragraph};
use windows::Win32::Graphics::DirectWrite::{
    IDWriteFactory, DWRITE_FONT_STYLE, DWRITE_FONT_STYLE_ITALIC, DWRITE_FONT_STYLE_NORMAL, DWRITE_FONT_STYLE_OBLIQUE,
    DWRITE_FONT_WEIGHT, DWRITE_MATRIX, DWRITE_TEXT_ALIGNMENT, DWRITE_TEXT_ALIGNMENT_CENTER,
    DWRITE_TEXT_ALIGNMENT_JUSTIFIED, DWRITE_TEXT_ALIGNMENT_LEADING, DWRITE_TEXT_ALIGNMENT_TRAILING,
};

fn dwrite_factory() -> &'static IDWriteFactory {
    &Application::instance().backend.dwrite_factory.0
}

trait ToDirectWrite {
    type Target;
    fn to_dwrite(&self) -> Self::Target;
}

impl ToDirectWrite for FontWeight {
    type Target = DWRITE_FONT_WEIGHT;
    fn to_dwrite(&self) -> Self::Target {
        DWRITE_FONT_WEIGHT(self.0 as i32)
    }
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

impl ToDirectWrite for Transform {
    type Target = DWRITE_MATRIX;

    fn to_dwrite(&self) -> Self::Target {
        DWRITE_MATRIX {
            m11: self.m11 as f32,
            m12: self.m12 as f32,
            m21: self.m21 as f32,
            m22: self.m22 as f32,
            dx: self.m31 as f32,
            dy: self.m32 as f32,
        }
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
