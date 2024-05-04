use std::ops::Deref;

/// An entry in a `FlatTree`.
#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq, Debug)]
pub enum Entry<I> {
    /// Inner node.
    Node(I),
    /// Leaf node.
    Leaf(I),
    Enter,
    Exit,
}

/// Represents a set of paths in a tree structure.
///
/// TODO: de-genericize this if we only use it for WidgetIds
#[derive(Clone)]
pub struct PathSet<I> {
    entries: Vec<Entry<I>>,
}

impl<I> Default for PathSet<I> {
    fn default() -> Self {
        Self::new()
    }
}

impl<I> PathSet<I> {
    /// Creates a new empty tree.
    pub fn new() -> Self {
        Self { entries: vec![] }
    }
}

impl<I> PathSet<I>
where
    I: Copy + Eq + Ord + Default,
{
    /// Creates a new tree from the specified path.
    pub fn from_path(path: &[I]) -> Self {
        let mut tree = Self::new();
        tree.insert(path);
        tree
    }

    pub fn from_path_bubbling(path: &[I]) -> Self {
        let mut tree = Self::new();
        for i in 0..path.len() {
            tree.insert(&path[0..=i]);
        }
        tree
    }

    /// Returns the entry at the specified path, creating it if it does not exist.
    pub fn insert(&mut self, path: &[I]) {
        /*if path.is_empty() {
            return;
        }*/
        self.insert_inner(0, path)
    }

    /// Returns an iterator over each subtree of the tree.
    pub fn traverse(&self) -> PathTraversalIter<I> {
        PathTraversalIter(&self.entries)
    }

    pub fn merge_with(&mut self, other: PathSubset<I>) {
        // TODO make something more efficient
        for path in other.iter() {
            self.insert(&path);
        }
    }

    /// Borrows this path set as a slice.
    pub fn as_slice(&self) -> PathSubset<I> {
        PathSubset(&self.entries)
    }

    fn insert_inner(&mut self, subtree: usize, path: &[I]) {
        let Some((head, rest)) = path.split_first() else {
            panic!("ids must not be empty");
        };
        let head = *head;
        let mut i = subtree;
        use Entry::*;

        while i < self.entries.len() {
            match self.entries[i] {
                Node(id) | Leaf(id) => {
                    if id == head {
                        if rest.is_empty() {
                            self.entries[i] = Leaf(id);
                        } else {
                            if !matches!(self.entries.get(i + 1), Some(Enter)) {
                                self.entries.splice((i + 1)..(i + 1), [Enter, Exit]);
                            }
                            self.insert_inner(i + 2, rest);
                        }
                        return;
                    } else if id > head {
                        break;
                    }
                }
                Enter => {
                    // skip
                    let mut depth = 1;
                    while depth > 0 && i < self.entries.len() {
                        i += 1;
                        match self.entries[i] {
                            Enter => depth += 1,
                            Exit => depth -= 1,
                            _ => {}
                        }
                    }
                }
                Exit => {
                    break;
                }
            }
            i += 1;
        }

        // if not inserted, insert at last position
        if rest.is_empty() {
            self.entries.insert(i, Leaf(head));
        } else {
            self.entries.splice(i..i, [Node(head), Enter, Exit]);
            self.insert_inner(i + 2, rest);
        }
    }

    /*fn remove(&mut self, ids: &[u32]) {
        let mut i = 0;
        let mut depth = 0;
        let mut found = false;
        while i < self.entries.len() {
            match self.entries[i] {
                Entry::Node(id) | Entry::Leaf(id) => {
                    if depth == ids.len() && id == ids[depth] {
                        found = true;
                        break;
                    }
                    if depth < ids.len() && id == ids[depth] {
                        depth += 1;
                    } else {
                        depth = 0;
                    }
                }
                Entry::Enter => {
                    depth += 1;
                }
                Entry::Exit => {
                    depth -= 1;
                }
            }
            i += 1;
        }

        if found {
            let mut j = i;
            while j < self.entries.len() {
                match self.entries[j] {
                    Entry::Enter => {
                        let mut depth = 1;
                        while depth > 0 && j < self.entries.len() {
                            j += 1;
                            match self.entries[j] {
                                Entry::Enter => depth += 1,
                                Entry::Exit => depth -= 1,
                                _ => {}
                            }
                        }
                    }
                    Entry::Exit => {
                        break;
                    }
                    _ => {}
                }
                j += 1;
            }
            self.entries.drain(i..=j);
        }
    }*/
}

/// A subset of paths in a `PathSet`, sharing a common root element (i.e. a subtree within a `PathSet`).
pub struct PathSubset<'a, I>(&'a [Entry<I>]);

impl<'a, I> Default for PathSubset<'a, I> {
    fn default() -> Self {
        PathSubset(&[])
    }
}

impl<'a, I> Clone for PathSubset<'a, I> {
    fn clone(&self) -> Self {
        PathSubset(self.0)
    }
}

impl<'a, I> Copy for PathSubset<'a, I> {}

impl<'a, I> PathSubset<'a, I>
where
    I: Copy + Default,
{
    /// Returns an iterator over each subtree.
    pub fn traverse(&self) -> PathTraversalIter<I> {
        PathTraversalIter(self.0)
    }

    pub fn iter(&self) -> PathSetIter<I> {
        PathSetIter {
            rest: self.0,
            path: vec![Default::default()],
        }
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /*
    /// Returns the root element of the subset.
    pub fn root(&self) -> I {
        match self.0.first() {
            Some(Entry::Node(id)) | Some(Entry::Leaf(id)) => Some(*id),
            _ => panic!("invalid path subset"),
        }
    }

    /// Returns whether the root element is a leaf.
    pub fn is_leaf(&self) -> bool {
        matches!(self.0.first(), Some(Entry::Leaf(_)))
    }*/
}

/// An iterator over the subtrees of a path set.
pub struct PathTraversalIter<'a, I>(&'a [Entry<I>]);

impl<'a, I> Iterator for PathTraversalIter<'a, I>
where
    I: Copy,
{
    type Item = (I, bool, PathSubset<'a, I>);

    fn next(&mut self) -> Option<Self::Item> {
        let Some((cur, mut rest)) = self.0.split_first() else {
            return None;
        };

        let (id, leaf) = match cur {
            Entry::Node(k) => (k, false),
            Entry::Leaf(k) => (k, true),
            _ => {
                // happens if next entry is `Exit`
                return None;
            }
        };

        let next_paths = match rest {
            [Entry::Enter, ..] => {
                let mut depth = 1;
                let mut i = 1;
                while depth > 0 && i < rest.len() {
                    match rest[i] {
                        Entry::Enter => depth += 1,
                        Entry::Exit => depth -= 1,
                        _ => {}
                    }
                    i += 1;
                }
                let (first, after) = self.0.split_at(i);
                rest = after;
                PathSubset(&first[1..])
            }
            _ => PathSubset(&[]),
        };

        self.0 = rest;
        Some((*id, leaf, next_paths))
    }
}

pub struct PathSetIter<'a, I> {
    rest: &'a [Entry<I>],
    path: Vec<I>,
}

impl<'a, I> Iterator for PathSetIter<'a, I>
where
    I: Copy + Default,
{
    type Item = Vec<I>;

    fn next(&mut self) -> Option<Self::Item> {
        while !self.rest.is_empty() {
            let Some((cur, rest)) = self.rest.split_first() else {
                return None;
            };
            self.rest = rest;
            match cur {
                Entry::Leaf(id) => {
                    let len = self.path.len();
                    self.path[len - 1] = *id;
                    let (_, rest) = self.rest.split_at(1);
                    self.rest = rest;
                    return Some(self.path.clone());
                }
                Entry::Node(id) => {
                    let len = self.path.len();
                    self.path[len - 1] = *id;
                }
                Entry::Enter => {
                    self.path.push(Default::default());
                }
                Entry::Exit => {
                    self.path.pop();
                }
            }

            let (_, rest) = self.rest.split_at(1);
            self.rest = rest;
        }
        return None;
    }
}

#[cfg(test)]
mod tests {
    use super::{Entry, PathSet};

    fn node(id: u32) -> Entry<u32> {
        Entry::Node(id)
    }
    fn leaf(id: u32) -> Entry<u32> {
        Entry::Leaf(id)
    }

    fn enter() -> Entry<u32> {
        Entry::Enter
    }
    fn exit() -> Entry<u32> {
        Entry::Exit
    }

    #[test]
    fn test_id_tree() {
        let mut tree = PathSet::new();
        tree.insert(&[1, 2, 3]);
        tree.insert(&[1, 2, 4]);
        tree.insert(&[1, 5]);
        tree.insert(&[1, 5, 1]);
        tree.insert(&[6]);

        eprintln!("{:?}", tree.entries);

        assert_eq!(
            tree.entries,
            vec![
                node(1),
                enter(),
                node(2),
                enter(),
                leaf(3),
                leaf(4),
                exit(),
                leaf(5),
                enter(),
                leaf(1),
                exit(),
                exit(),
                leaf(6),
            ]
        );
    }
}
