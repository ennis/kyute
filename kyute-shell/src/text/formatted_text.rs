use crate::text::{resolve_range, Attribute, FontStyle, FontWeight, TextAlignment};
use kyute_common::Data;
use std::{
    cmp::Ordering,
    ops::{Range, RangeBounds},
    sync::Arc,
};

/// A run of text sharing the same text attributes.
#[derive(Clone, Debug, PartialEq)]
pub struct TextRun {
    pub range: Range<usize>,
    pub attributes: Vec<Attribute>,
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
pub struct TextRuns {
    pub runs: Vec<TextRun>,
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

    pub fn merge_attribute(&mut self, range: Range<usize>, attribute: &Attribute) {
        if range.is_empty() {
            return;
        }

        let (first_run, last_run) = self.split(range);
        for run in &mut self.runs[first_run..=last_run] {
            run.merge_attribute(attribute);
        }
    }
}

#[derive(Clone, Debug, Data, Default)]
pub struct ParagraphStyle {
    pub text_alignment: Option<TextAlignment>,
    pub font_style: Option<FontStyle>,
    pub font_weight: Option<FontWeight>,
    pub font_size: Option<f64>,
    pub font_family: Option<String>,
}

/// Text with formatting information.
#[derive(Clone, Data)]
pub struct FormattedText {
    pub plain_text: Arc<str>,
    pub runs: Arc<TextRuns>,
    pub paragraph_style: ParagraphStyle,
}

impl Default for FormattedText {
    fn default() -> Self {
        FormattedText {
            plain_text: Arc::from(""),
            runs: Arc::new(TextRuns {
                runs: vec![TextRun {
                    range: 0..0,
                    attributes: vec![],
                }],
            }),
            paragraph_style: Default::default(),
        }
    }
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
    pub fn new(text: impl Into<Arc<str>>) -> FormattedText {
        FormattedText {
            plain_text: text.into(),
            runs: Arc::new(TextRuns { runs: vec![] }),
            paragraph_style: Default::default(),
        }
    }

    /// Adds the specified attribute on the range of characters.
    pub fn add_attribute(&mut self, range: impl RangeBounds<usize>, attribute: impl Into<Attribute>) {
        let range = resolve_range(range, self.plain_text.len());
        Arc::make_mut(&mut self.runs).merge_attribute(range, &attribute.into())
    }

    /// Sets the font size.
    pub fn set_font_size(&mut self, font_size: f64) {
        self.paragraph_style.font_size = Some(font_size);
    }

    /// Sets the font style.
    pub fn set_font_style(&mut self, font_style: FontStyle) {
        self.paragraph_style.font_style = Some(font_style);
    }

    /// Sets the font weight.
    pub fn set_font_weight(&mut self, font_weight: FontWeight) {
        self.paragraph_style.font_weight = Some(font_weight);
    }

    /// Sets the font weight.
    pub fn set_text_alignment(&mut self, alignment: TextAlignment) {
        self.paragraph_style.text_alignment = Some(alignment);
    }

    /// Sets the font family.
    pub fn set_font_family(&mut self, font_family: &str) {
        self.paragraph_style.font_family = Some(font_family.to_owned())
    }

    pub fn with_paragraph_style(mut self, style: ParagraphStyle) -> Self {
        self.set_paragraph_style(style);
        self
    }

    pub fn set_paragraph_style(&mut self, style: ParagraphStyle) {
        self.paragraph_style = style;
    }
}

pub trait FormattedTextExt {
    /// Returns a new formatted text object with the specified font size set.
    fn font_size(self, font_size: f64) -> FormattedText;
    /// Returns a new formatted text object with the specified font style set.
    fn font_style(self, font_style: FontStyle) -> FormattedText;
    /// Sets the font family.
    fn font_family(self, font_family: &str) -> FormattedText;
    /// Returns a new formatted text object with the specified font weight set.
    fn font_weight(self, font_weight: FontWeight) -> FormattedText;
    /// Returns a new formatted text object with the specified text alignment set.
    fn text_alignment(self, alignment: TextAlignment) -> FormattedText;
    /// Returns a new formatted text object with the specified attribute applied on the range of characters.
    fn attribute(self, range: impl RangeBounds<usize>, attribute: impl Into<Attribute>) -> FormattedText;
}

impl<T> FormattedTextExt for T
where
    T: Into<Arc<str>>,
{
    fn font_size(mut self, font_size: f64) -> FormattedText {
        FormattedText::new(self.into()).font_size(font_size)
    }

    fn font_style(self, font_style: FontStyle) -> FormattedText {
        FormattedText::new(self.into()).font_style(font_style)
    }

    fn font_family(self, font_family: &str) -> FormattedText {
        FormattedText::new(self.into()).font_family(font_family)
    }

    fn font_weight(self, font_weight: FontWeight) -> FormattedText {
        FormattedText::new(self.into()).font_weight(font_weight)
    }

    fn text_alignment(mut self, alignment: TextAlignment) -> FormattedText {
        FormattedText::new(self.into()).text_alignment(alignment)
    }

    fn attribute(mut self, range: impl RangeBounds<usize>, attribute: impl Into<Attribute>) -> FormattedText {
        FormattedText::new(self.into()).attribute(range, attribute)
    }
}

impl FormattedTextExt for FormattedText {
    fn font_size(mut self, font_size: f64) -> FormattedText {
        self.set_font_size(font_size);
        self
    }

    fn font_style(mut self, font_style: FontStyle) -> FormattedText {
        self.set_font_style(font_style);
        self
    }

    fn font_family(mut self, font_family: &str) -> FormattedText {
        self.set_font_family(font_family);
        self
    }

    fn font_weight(mut self, font_weight: FontWeight) -> FormattedText {
        self.set_font_weight(font_weight);
        self
    }

    fn text_alignment(mut self, alignment: TextAlignment) -> FormattedText {
        self.set_text_alignment(alignment);
        self
    }

    fn attribute(mut self, range: impl RangeBounds<usize>, attribute: impl Into<Attribute>) -> FormattedText {
        self.add_attribute(range, attribute);
        self
    }
}
