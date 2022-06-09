//! Live-editable numeric/string literals.
use crate::{cache, composable, util};
use lazy_static::lazy_static;
use parking_lot::Mutex;
use similar::{utils::TextDiffRemapper, Change, DiffOp, DiffTag};
use std::{
    cell::RefCell,
    collections::{hash_map::Entry, HashMap},
    fs, io,
    ops::Range,
    path::{Path, PathBuf},
    sync::Arc,
};

/// A type that can be parsed from a rust literal expression and supports live-editing.
pub trait LiveLiteral: Clone {
    fn parse(lit: &str) -> Option<Self>;
}

macro_rules! impl_numeric_live_literal {
    ($($t:ty)+) => {
        $(
            impl LiveLiteral for $t {
                fn parse(x: &str) -> Option<$t> {
                    x.parse().ok()
                }
            }
        )+
    };
}

// implementation for numeric literals
impl_numeric_live_literal!(u8 u16 u32 u64 u128 i8 i16 i32 i64 i128 usize isize bool f32 f64);

/// Live-editable strings.
impl LiveLiteral for &'static str {
    fn parse(lit: &str) -> Option<Self> {
        // TODO maybe remove the dependency on syn at some point?
        let lit: syn::LitStr = syn::parse_str(lit).ok()?;
        // since literal tweaking is a development feature, it's acceptable to leak there
        Some(Box::leak(lit.value().into_boxed_str()))
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
struct LineColumn {
    line: u32,
    col: u32,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
struct Span {
    start: LineColumn,
    end: LineColumn,
}

fn map_position(diff: &[DiffOp], pos: usize) -> (DiffTag, Option<usize>) {
    for op in diff {
        let (tag, old_range, new_range) = op.as_tag_tuple();
        if old_range.contains(&pos) {
            match tag {
                DiffTag::Equal => return (DiffTag::Equal, Some(pos - old_range.start + new_range.start)),
                DiffTag::Delete => return (DiffTag::Delete, None),
                DiffTag::Insert => {
                    unreachable!()
                }
                DiffTag::Replace => {
                    // TODO not sure about that
                    let offset = pos - old_range.start;
                    if offset < new_range.len() {
                        return (DiffTag::Replace, Some(offset + new_range.start));
                    } else {
                        return (DiffTag::Replace, None);
                    }
                }
            }
        }
    }
    (DiffTag::Delete, None)
}

/// Returns the byte offset in the given text for the specified line & column positions.
fn compute_byte_offset(text: &str, line: u32, column: u32) -> Option<usize> {
    assert!(line > 0);
    assert!(column > 0);
    let line = text.lines().nth((line - 1) as usize)?;
    let col_offset = (column - 1) as usize;
    if line.is_char_boundary(col_offset) {
        unsafe { Some(line.as_ptr().add(col_offset).offset_from(text.as_ptr()) as usize) }
    } else {
        None
    }
}

///
struct SourceMap {
    /// Path to the source file.
    source_path: PathBuf,
    /// Original source text.
    original: String,
    /// Last version of the source text.
    current: String,
    diff: Vec<DiffOp>,
    literal_ranges: RefCell<HashMap<Span, Range<usize>>>,
}

impl SourceMap {
    /// Creates a new source map for the specified source file.
    fn new<P: Into<PathBuf>>(source_path: P) -> io::Result<SourceMap> {
        let source_path = source_path.into();
        let original = fs::read_to_string(&source_path)?;
        let current = original.clone();
        Ok(SourceMap {
            source_path,
            original,
            current,
            diff: vec![],
            literal_ranges: RefCell::new(Default::default()),
        })
    }

    /// Returns the text in the given span, taking into account the modifications to the original source text.
    fn get_text(&self, span: Span) -> &str {
        let mut literal_ranges = self.literal_ranges.borrow_mut();
        let range = literal_ranges
            .entry(span)
            .or_insert_with(|| {
                let start_offset = compute_byte_offset(&self.original, span.start.line, span.start.col);
                let end_offset = compute_byte_offset(&self.original, span.end.line, span.end.col);
                if start_offset.is_none() || end_offset.is_none() {
                    warn!(
                        "({}) literal span ({}:{} -> {}:{}) not found in file",
                        self.source_path.display(),
                        span.start.line,
                        span.start.col,
                        span.end.line,
                        span.end.col
                    );
                    0..0
                } else {
                    start_offset.unwrap()..end_offset.unwrap()
                }
            })
            .clone();

        // remap the original range to the current version of the source text
        let (_, new_start) = map_position(&self.diff, range.start);
        let (_, new_end) = map_position(&self.diff, range.end);
        match (new_start, new_end) {
            (Some(start), Some(end)) if end > start => &self.current[start..end],
            _ => "",
        }
    }

    /// Should be called whenever the source file changes to update the text mapping.
    fn update(&mut self) -> io::Result<()> {
        // diff the new and prev sources
        self.current = fs::read_to_string(&self.source_path)?;
        self.diff = similar::TextDiff::from_chars(&self.original, &self.current)
            .ops()
            .to_vec();
        Ok(())
    }
}

lazy_static! {
    static ref SOURCE_MAPS: Mutex<HashMap<&'static str, SourceMap>> = Mutex::new(HashMap::new());
}

/// Returns the current value of a literal in a rust source file.
///
/// This composable function creates a state variable, initialized to `original_value`.
/// Whenever the specified rust source file changes, the function updates the state variable
/// by trying to parse a literal expression in the source (using `LiveLiteral::parse`), at the specified span (start/end line/column),
/// adjusted for file modifications.
///
/// If any of this fails (e.g. if the source file isn't accessible, or if the expression under the span is malformed)
/// the function leaves the state variable unchanged and returns its current value.
///
/// This function is intended for use by the `#[composable(live_literals)]` proc-macro, and shouldn't be called directly.
#[doc(hidden)]
#[composable]
pub fn live_literal<T: LiveLiteral + 'static>(
    source_file: &'static str,
    start_line: u32,
    start_column: u32,
    end_line: u32,
    end_column: u32,
    original_value: T,
) -> T {
    //assert!(line_number > 0);
    let value = cache::state(|| original_value);

    // create the SourceMap for the source file if not done already
    let mut sources = SOURCE_MAPS.lock();
    let source_map = match sources.entry(source_file) {
        Entry::Occupied(map) => map.into_mut(),
        Entry::Vacant(entry) => {
            match SourceMap::new(source_file) {
                Ok(map) => entry.insert(map),
                Err(err) => {
                    // Creation of the source map failed, possibly because the source file isn't accessible. Bail out.
                    warn!("({}) failed to create source map: {}", source_file, err);
                    return value.get();
                }
            }
        }
    };

    let span = Span {
        start: LineColumn {
            line: start_line,
            col: start_column,
        },
        end: LineColumn {
            line: end_line,
            col: end_column,
        },
    };

    // watch source changes
    if util::fs_watch::watch_path(source_file) {
        eprintln!("file {} changed", source_file);

        // update source map
        if let Err(err) = source_map.update() {
            warn!("({}) failed to update source map: {}", source_file, err);
            return value.get();
        }

        // read span in the new source text and try to parse it to a `T`
        let span_text = source_map.get_text(span);
        if span_text.is_empty() {
            // no text, bail out (get_text outputs a warning)
            return value.get();
        }
        if let Some(v) = T::parse(span_text) {
            eprintln!("({}) tweakable value changed: `{}`", source_file, span_text);
            value.set(v.clone());
        } else {
            error!("({}) failed to parse tweakable value: `{}`", source_file, span_text);
            return value.get();
        }
    }

    value.get()
}
