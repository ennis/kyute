use crate::{AnyWidget, ChangeFlags, Element, TreeCtx, Widget};
use std::cmp::Ordering;

/*/// An element of a vector diff.
pub enum DiffElem<T> {
    /// Add element at the specified position.
    Insert(usize, T),
    /// Remove the element at the specified position.
    Remove(usize),
    /// Modifies the element at the specified position;
    Replace(usize, T),
}

impl<T> DiffElem<T> {
    /// Returns the position of the change.
    pub fn index(&self) -> usize {
        match *self {
            DiffElem::Insert(pos, _) | DiffElem::Remove(pos) | DiffElem::Replace(pos, _) => pos,
        }
    }
}

/// List of changes (insertions/removals) to apply on a vector (or any sequence container indexed by usize).
#[derive(Clone, Debug)]
pub struct VecDiff<T>(Vec<DiffElem<T>>);

impl<T> VecDiff<T> {
    /// Creates a new, empty `VecDiff` (no changes).
    pub fn new() -> VecDiff<T> {
        VecDiff(vec![])
    }

    /// Adds a change that inserts an element at the given position.
    pub fn insert(&mut self, at: usize, elem: T) {
        self.0.push(DiffElem::Insert(at, elem))
    }

    /// Adds a change that removes an element at the given position.
    pub fn remove(&mut self, at: usize) {
        self.0.push(DiffElem::Remove(at))
    }

    /// Consumes and applies this diff to the given vector.
    pub fn apply<U>(mut self, on: &mut Vec<U>, insert: impl FnMut(T) -> U, modify: impl FnMut(&mut U, T)) {
        // go through changes in order
        self.0.sort_by_key(DiffElem::<T>::index);
        let mut fixup = 0;
        for entry in self.0 {
            match entry {
                DiffElem::Insert(pos, val) => {
                    on.insert(pos + fixup, insert(val));
                    fixup += 1;
                }
                DiffElem::Remove(pos) => {
                    on.remove(pos + fixup);
                    fixup -= 1;
                }
                DiffElem::Replace(pos, val) => {
                    modify(&mut on[pos + fixup], val);
                }
            }
        }
    }
}*/

////////////////////////////////////////////////////////////////////////////////////////////////////
/*
enum PatchItem<W> {
    /// Moves the cursor to the beginning of the sequence.
    Start,
    /// Advances the cursor.
    Advance(usize),
    /// Inserts a new widget at the current cursor position.
    Insert(W),
    /// Updates the element at the current position if the ID matches, otherwise inserts a new element constructed from the widget.
    Update(W),
    /// Removes the widget at the cursor position.
    Remove,
    /// Skips to the end.
    SkipToEnd,
    /// Terminates the patch and trims extra elements after the cursor.
    End,
}

// TODO: this should be in the same module as `Widget`
pub struct Patch<W> {
    items: Vec<PatchItem<W>>,
}

impl<W> Patch<W> {
    pub fn new() -> Patch<W> {
        Patch {
            items: vec![PatchItem::End],
        }
    }

    pub fn apply<T>(self, cx: &mut TreeCtx, elements: &mut Vec<T>, env: &Environment) -> ChangeFlags
    where
        T: Element,
        W: Widget<Element = T>,
    {
        let mut cursor = 0;
        let mut change_flags = ChangeFlags::empty();
        for patch in self.items {
            match patch {
                PatchItem::Start => {
                    cursor = 0;
                }
                PatchItem::Advance(adv) => {
                    cursor = (cursor + adv).min(elements.len());
                }
                PatchItem::Insert(widget) => {
                    elements.insert(cursor, widget.build(cx, env));
                    change_flags |= ChangeFlags::STRUCTURE;
                    cursor += 1;
                }
                PatchItem::Remove => {
                    elements.remove(cursor);
                    change_flags |= ChangeFlags::STRUCTURE;
                }
                PatchItem::SkipToEnd => {
                    cursor = elements.len();
                }
                PatchItem::Update(widget) => {
                    change_flags |= widget.update(cx, &mut elements[cursor], env);
                    cursor += 1;
                }
                PatchItem::End => {}
            }
        }
        elements.truncate(cursor);
        change_flags
    }
}
*/
