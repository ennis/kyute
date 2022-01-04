use std::iter;

mod menu;

pub use menu::Menu;

/// Converts a string to a sequence of UTF-16 characters.
pub(crate) fn str_to_wstr(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(iter::once(0)).collect()
}
