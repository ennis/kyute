use keyboard_types::Modifiers;
use std::{fmt::Write, ops::Range};

/// Subset of `Key`s usable as the last key in a shortcut.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum ShortcutKey {
    // We don't support arbitrary strings because we need to be const
    Character(char),
    Enter,
    Tab,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    ArrowUp,
    End,
    Home,
    PageDown,
    PageUp,
    Backspace,
    Delete,
    Insert,
    Attn,
    Escape,
    PrintScreen,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
}

impl ShortcutKey {
    pub const fn from_str(s: &[u8]) -> ShortcutKey {
        match s {
            b"Enter" => ShortcutKey::Enter,
            b"Tab" => ShortcutKey::Tab,
            b"ArrowDown" => ShortcutKey::ArrowDown,
            b"ArrowLeft" => ShortcutKey::ArrowLeft,
            b"ArrowRight" => ShortcutKey::ArrowRight,
            b"ArrowUp" => ShortcutKey::ArrowUp,
            b"End" => ShortcutKey::End,
            b"Home" => ShortcutKey::Home,
            b"PageDown" => ShortcutKey::PageDown,
            b"PageUp" => ShortcutKey::PageUp,
            b"Backspace" => ShortcutKey::Backspace,
            b"Delete" => ShortcutKey::Delete,
            b"Insert" => ShortcutKey::Insert,
            b"Attn" => ShortcutKey::Attn,
            b"Escape" => ShortcutKey::Escape,
            b"PrintScreen" => ShortcutKey::PrintScreen,
            b"F1" => ShortcutKey::F1,
            b"F2" => ShortcutKey::F2,
            b"F3" => ShortcutKey::F3,
            b"F4" => ShortcutKey::F4,
            b"F5" => ShortcutKey::F5,
            b"F6" => ShortcutKey::F6,
            b"F7" => ShortcutKey::F7,
            b"F8" => ShortcutKey::F8,
            b"F9" => ShortcutKey::F9,
            b"F10" => ShortcutKey::F10,
            b"F11" => ShortcutKey::F11,
            b"F12" => ShortcutKey::F12,
            other => {
                assert!(other.len() == 1);
                ShortcutKey::Character(other[0] as char)
            }
        }
    }

    pub fn to_key(&self) -> keyboard_types::Key {
        match *self {
            ShortcutKey::Character(c) => keyboard_types::Key::Character(c.to_string()),
            ShortcutKey::Enter => keyboard_types::Key::Enter,
            ShortcutKey::Tab => keyboard_types::Key::Tab,
            ShortcutKey::ArrowDown => keyboard_types::Key::ArrowDown,
            ShortcutKey::ArrowLeft => keyboard_types::Key::ArrowLeft,
            ShortcutKey::ArrowRight => keyboard_types::Key::ArrowRight,
            ShortcutKey::ArrowUp => keyboard_types::Key::ArrowUp,
            ShortcutKey::End => keyboard_types::Key::End,
            ShortcutKey::Home => keyboard_types::Key::Home,
            ShortcutKey::PageDown => keyboard_types::Key::PageDown,
            ShortcutKey::PageUp => keyboard_types::Key::PageUp,
            ShortcutKey::Backspace => keyboard_types::Key::Backspace,
            ShortcutKey::Delete => keyboard_types::Key::Delete,
            ShortcutKey::Insert => keyboard_types::Key::Insert,
            ShortcutKey::Attn => keyboard_types::Key::Attn,
            ShortcutKey::Escape => keyboard_types::Key::Escape,
            ShortcutKey::PrintScreen => keyboard_types::Key::PrintScreen,
            ShortcutKey::F1 => keyboard_types::Key::F1,
            ShortcutKey::F2 => keyboard_types::Key::F2,
            ShortcutKey::F3 => keyboard_types::Key::F3,
            ShortcutKey::F4 => keyboard_types::Key::F4,
            ShortcutKey::F5 => keyboard_types::Key::F5,
            ShortcutKey::F6 => keyboard_types::Key::F6,
            ShortcutKey::F7 => keyboard_types::Key::F7,
            ShortcutKey::F8 => keyboard_types::Key::F8,
            ShortcutKey::F9 => keyboard_types::Key::F9,
            ShortcutKey::F10 => keyboard_types::Key::F10,
            ShortcutKey::F11 => keyboard_types::Key::F11,
            ShortcutKey::F12 => keyboard_types::Key::F12,
        }
    }
}

const fn const_subslice<T>(mut s: &[T], range: Range<usize>) -> &[T] {
    assert!(range.end >= range.start);

    let mut i = 0;
    let mut j = s.len();

    while i < range.start {
        match s {
            [_, xs @ ..] => {
                i += 1;
                s = xs;
            }
            _ => break,
        }
    }

    while j > range.end {
        match s {
            [xs @ .., _] => {
                j -= 1;
                s = xs;
            }
            _ => break,
        }
    }

    //constfn_assert!(i == range.start);
    //constfn_assert!(j == range.end);
    //constfn_assert!(s.len() == j - i);
    s
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct Shortcut {
    /// Modifier (e.g. the `Ctrl` in `Ctrl+Z`).
    pub modifiers: Modifiers,
    /// Non-modifier key.
    pub key: ShortcutKey,
}

impl Shortcut {
    pub const fn new(modifiers: Modifiers, key: ShortcutKey) -> Shortcut {
        Shortcut { modifiers, key }
    }

    pub const fn from_str(str: &str) -> Shortcut {
        let s = str.as_bytes();
        let mut p = 0;
        let mut modifiers = Modifiers::empty();

        // Ctrl+
        if matches!(const_subslice(s, p..p + 5), b"Ctrl+") {
            p += 5;
            modifiers = modifiers.union(Modifiers::CONTROL);
        }

        if matches!(const_subslice(s, p..p + 4), b"Alt+") {
            p += 4;
            modifiers = modifiers.union(Modifiers::ALT);
        };

        if matches!(const_subslice(s, p..p + 6), b"Shift+") {
            p += 6;
            modifiers = modifiers.union(Modifiers::SHIFT);
        };

        let key = ShortcutKey::from_str(const_subslice(s, p..s.len()));
        Shortcut::new(modifiers, key)
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
        write!(s, "{}", self.key.to_key()).unwrap();
        s
    }
}
