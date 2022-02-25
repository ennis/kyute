use crate::{resolve_range, Attribute, TextAlignment};
use kyute_common::Data;
use std::{
    cmp::Ordering,
    ops::{Range, RangeBounds},
    sync::Arc,
};

/// A run of text sharing the same text attributes.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct TextRun {
    pub(crate) range: Range<usize>,
    pub(crate) attributes: Vec<Attribute>,
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
pub(crate) struct TextRuns {
    pub(crate) runs: Vec<TextRun>,
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

    pub(crate) fn merge_attribute(&mut self, range: Range<usize>, attribute: &Attribute) {
        if range.is_empty() {
            return;
        }

        let (first_run, last_run) = self.split(range);
        for run in &mut self.runs[first_run..=last_run] {
            run.merge_attribute(attribute);
        }
    }
}

#[derive(Clone, Debug, Default, Data)]
pub struct ParagraphStyle {
    pub text_alignment: TextAlignment,
}

/// Text with formatting information.
#[derive(Clone, Data)]
pub struct FormattedText {
    pub plain_text: Arc<str>,
    pub(crate) runs: Arc<TextRuns>,
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

    /*/// Layouts the text into a paragraph.
    pub fn format(&self, width: f64) -> Paragraph {



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
    }*/
}
