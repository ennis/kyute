use std::{cell::OnceCell, sync::Arc};

use skia_safe as sk;
use skia_safe::{textlayout::FontCollection, FontMgr};
use tracy_client::span;

use crate::{drawing::ToSkia, Color};

thread_local! {
    static FONT_COLLECTION: OnceCell<FontCollection> = OnceCell::new();
}

/// Returns the FontCollection for the current thread.
///
/// FontCollections (and other objects that reference them, e.g. Paragraph)
/// are bound to the thread in which they were created.
pub(crate) fn get_font_collection() -> FontCollection {
    // Ideally I'd like to have only one font collection for all threads.
    // However, FontCollection isn't Send or Sync, and `Paragraphs` hold a reference to a FontCollection,
    // so, to be able to create Paragraphs from different threads, there must be one FontCollection
    // per thread.
    //
    // See also https://github.com/rust-skia/rust-skia/issues/537
    FONT_COLLECTION.with(|fc| {
        fc.get_or_init(|| {
            let mut font_collection = FontCollection::new();
            font_collection.set_default_font_manager(FontMgr::new(), None);
            font_collection
        })
        .clone()
    })
}

#[derive(Copy, Clone, Debug, PartialOrd, Ord, PartialEq, Eq)]
pub enum ChangeKind {
    Identical,
    Metadata,
    Paint,
    Layout,
}

#[derive(Default)]
pub struct TextStyle {
    /// Font size in DIPs.
    pub font_size: Option<f64>,
    pub font_families: Option<Vec<String>>,
    pub color: Option<Color>,
}

impl PartialEq for TextStyle {
    fn eq(&self, other: &Self) -> bool {
        self.compare_to(other) == ChangeKind::Identical
    }
}

impl Eq for TextStyle {}

impl TextStyle {
    pub fn new() -> TextStyle {
        TextStyle {
            font_size: None,
            font_families: None,
            color: None,
        }
    }

    pub fn color(mut self, color: Color) -> TextStyle {
        self.color = Some(color);
        self
    }

    pub fn font_size(mut self, font_size: f64) -> TextStyle {
        self.font_size = Some(font_size);
        self
    }

    pub fn font_family(mut self, font_family: impl Into<String>) -> TextStyle {
        self.font_families = Some(vec![font_family.into()]);
        self
    }

    pub fn font_families(mut self, families: impl Into<Vec<String>>) -> TextStyle {
        self.font_families = Some(families.into());
        self
    }

    pub fn is_null(&self) -> bool {
        self == &TextStyle::default()
    }

    pub fn compare_to(&self, other: &TextStyle) -> ChangeKind {
        if self.font_size != other.font_size || self.font_families != other.font_families {
            return ChangeKind::Layout; // return early since it's the most drastic change that can happen
        }
        if self.color != other.color {
            return ChangeKind::Paint;
        }
        ChangeKind::Identical
    }

    pub fn to_skia(&self) -> sk::textlayout::TextStyle {
        let mut sk_style = sk::textlayout::TextStyle::new();
        if let Some(ref color) = self.color {
            sk_style.set_color(color.to_skia().to_color());
        }
        if let Some(ref font_size) = self.font_size {
            sk_style.set_font_size(*font_size as sk::scalar);
        }
        if let Some(ref font_families) = self.font_families {
            sk_style.set_font_families(&font_families[..]);
        }
        sk_style
    }
}

// TODO paragraphs
#[derive(Clone)]
pub struct TextSpan {
    pub text: String,
    pub style: Arc<TextStyle>,
    pub children: Vec<TextSpan>,
}

impl Default for TextSpan {
    fn default() -> Self {
        TextSpan {
            text: "".to_string(),
            style: Arc::new(Default::default()),
            children: vec![],
        }
    }
}

impl TextSpan {
    pub fn new(text: impl Into<String>, style: Arc<TextStyle>) -> TextSpan {
        TextSpan {
            text: text.into(),
            style,
            children: vec![],
        }
    }

    pub fn compare_to(&self, other: &TextSpan) -> ChangeKind {
        if self.text != other.text {
            return ChangeKind::Layout;
        }
        if self.children.len() != other.children.len() {
            // TODO this is too pessimistic but for now assume that the layout changes if new spans are added or removed
            return ChangeKind::Layout;
        }
        let mut change_kind = ChangeKind::Identical;
        for (child, other_child) in self.children.iter().zip(other.children.iter()) {
            change_kind = change_kind.max(child.compare_to(other_child));
            if change_kind == ChangeKind::Layout {
                break;
            }
        }

        return change_kind;
    }

    fn add_to_paragraph(&self, paragraph: &mut sk::textlayout::ParagraphBuilder) {
        let has_style = !self.style.is_null();
        if has_style {
            let sk_style = self.style.to_skia();
            paragraph.push_style(&sk_style);
        }

        if !self.text.is_empty() {
            paragraph.add_text(&self.text);
        }

        for child in self.children.iter() {
            child.add_to_paragraph(paragraph);
        }

        if has_style {
            paragraph.pop();
        }
    }

    /// Builds the skia `Paragraph` for this TextSpan.
    pub fn build_paragraph(&self) -> sk::textlayout::Paragraph {
        let _span = span!("build_paragraph");
        let font_collection = get_font_collection();
        let mut text_style = sk::textlayout::TextStyle::new();
        text_style.set_font_size(16.0 as sk::scalar); // TODO default font size
        let mut paragraph_style = sk::textlayout::ParagraphStyle::new();
        paragraph_style.set_text_style(&text_style);
        let mut builder = sk::textlayout::ParagraphBuilder::new(&paragraph_style, font_collection);
        self.add_to_paragraph(&mut builder);
        builder.build()
    }
}

/// Text selection.
///
/// Start is the start of the selection, end is the end. The caret is at the end of the selection.
/// Note that we don't necessarily have start <= end: a selection with start > end means that the
/// user started the selection gesture from a later point in the text and then went back
/// (right-to-left in LTR languages). In this case, the cursor will appear at the "beginning"
/// (i.e. left, for LTR) of the selection.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
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
