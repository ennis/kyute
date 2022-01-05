use keyboard_types::{Key, Modifiers};
use std::fmt::Write;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Shortcut {
    /// Modifier (e.g. the `Ctrl` in `Ctrl+Z`).
    pub modifiers: Modifiers,
    /// Non-modifier key.
    pub key: Key,
}

impl Shortcut {
    pub fn new(modifiers: Modifiers, key: Key) -> Shortcut {
        Shortcut { modifiers, key }
    }

    pub fn to_string(&self) -> String {
        let mut s = String::new();
        if self.modifiers.contains(Modifiers::CONTROL) {
            s.push_str("Ctrl+");
        }
        if self.modifiers.contains(Modifiers::ALT) {
            s.push_str("Alt+");
        }
        if self.modifiers.contains(Modifiers::SHIFT) {
            s.push_str("Shift+");
        }
        if self.modifiers.contains(Modifiers::META) {
            s.push_str("Windows+");
        }
        write!(s, "{}", self.key).unwrap();
        s
    }
}
