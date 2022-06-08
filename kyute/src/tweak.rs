//! Tweakable numeric literals.
use crate::{cache, composable, util};
use std::fs;

/// A type usable with `tweak!`.
pub trait Tweakable: Clone {
    fn parse(lit: &str) -> Option<Self>;
}

macro_rules! impl_numeric_tweakable {
    ($($t:ty)+) => {
        $(
            impl Tweakable for $t {
                fn parse(x: &str) -> Option<$t> {
                    x.parse().ok()
                }
            }
        )+
    };
}

impl_numeric_tweakable!(u8 u16 u32 u64 u128 i8 i16 i32 i64 i128 usize isize bool f32 f64);

impl Tweakable for &'static str {
    fn parse(lit: &str) -> Option<Self> {
        let lit: syn::LitStr = syn::parse_str(lit).ok()?;
        Some(Box::leak(lit.value().into_boxed_str()))
    }
}

struct Entry {
    orig_line: u32,
    orig_col: u32,
    index: u32,
}

struct SourceFile {
    //
    fixups: Vec<(usize, usize)>,
}

impl SourceFile {
    fn setup(&mut self) {
        // open the source, assign indices
    }
}

struct TweakablesMap {}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct LineColumn {
    pub line: usize,
    pub col: usize,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Span {
    pub start: LineColumn,
    pub end: LineColumn,
}

#[composable]
pub fn tweak<T: Tweakable + 'static>(source_file_path: &str, line_number: u32, original_value: T) -> T {
    assert!(line_number > 0);
    let value = cache::state(|| original_value);

    if util::fs_watch::watch_path(source_file_path) {
        eprintln!("file {} changed (L{})", source_file_path, line_number);
        // file has changed, get the value of the tweakable from the file.
        let file_contents = if let Ok(str) = fs::read_to_string(source_file_path) {
            str
        } else {
            error!("could not read source file");
            return value.get();
        };
        // find this tweakable at the specified line
        // FIXME: we assume two things:
        // 1. that there is only one instance of the "tweak!" substring per line (implying that there can be only one tweakable per line)
        // 2. that the same tweakable stays at the same line.
        //
        // (1) implies that there can be only one tweakable per line.
        // (2) means that we can't insert newlines in the source file while the application is running.
        //
        // This might be impractical with formatters that run automatically on save. They might move things to the same line, or on separate lines if the tokens inside the macros
        // are longer or shorter.
        let line = if let Some(line) = file_contents.lines().nth(line_number as usize - 1) {
            line
        } else {
            error!("could not find tweak line");
            return value.get();
        };
        let tweak_macro_pos = if let Some(p) = line.find("tweak!") {
            p
        } else {
            error!("could not find start of tweak macro");
            return value.get();
        };
        let line = &line[tweak_macro_pos + 6..];
        let start_brace = if let Some(p) = line.chars().nth(0) {
            p
        } else {
            error!("could not find start brace");
            return value.get();
        };
        let matching_brace = match start_brace {
            '{' => '}',
            '(' => ')',
            '[' => ']',
            _ => {
                error!("unexpected macro brace");
                return value.get();
            }
        };
        let end_brace_pos = if let Some(p) = line.find(matching_brace) {
            p
        } else {
            error!("could not find end brace");
            return value.get();
        };
        let contents = &line[1..end_brace_pos].trim();

        if let Some(v) = T::parse(contents) {
            eprintln!("tweak value changed : {} L{}", source_file_path, line_number);
            value.set(v.clone());
        } else {
            error!("failed to parse value: {}", contents);
            return value.get();
        }
    }

    value.get()
}

#[macro_export]
macro_rules! tweak {
    ($e:expr) => {
        $crate::tweak(file!(), line!(), $e)
    };
}
