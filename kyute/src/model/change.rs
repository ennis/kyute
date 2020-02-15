use crate::model::{Collection, Data, IndexAddress};
use std::ops::Range;

/// Summary of changes to a collection.
///
/// Represents a summary of the modifications applied to a collection between two predefined points
/// in time.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum CollectionChanges {
    Relayout,
    Splice {
        start: usize,
        remove: usize,
        insert: usize,
    },
    Update {
        // not Range<usize> because Range<T> is not copy for reasons
        start: usize,
        end: usize,
    },
}

/// Describes a change applied to some data.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Change {
    /// No changes
    None,
    /// Elements were inserted or removed into a collection.
    Collection(CollectionChanges),
    /// The entire element was replaced.
    Replace,
}

impl Change {
    pub fn replacement() -> Change {
        Change::Replace
    }

    pub fn empty() -> Change {
        Change::None
    }

    pub fn relayout() -> Change {
        Change::Collection(CollectionChanges::Relayout)
    }

    pub fn insertion(at: usize, len: usize) -> Change {
        Change::Collection(CollectionChanges::Splice {
            start: at,
            remove: 0,
            insert: len,
        })
    }

    pub fn deletion(at: usize, len: usize) -> Change {
        Change::Collection(CollectionChanges::Splice {
            start: at,
            remove: len,
            insert: 0,
        })
    }

    pub fn splicing(at: usize, remove: usize, insert: usize) -> Change {
        Change::Collection(CollectionChanges::Splice {
            start: at,
            remove,
            insert,
        })
    }

    pub fn update(range: Range<usize>) -> Change {
        Change::Collection(CollectionChanges::Update {
            start: range.start,
            end: range.end,
        })
    }
}

#[derive(Debug)]
pub struct Revision<'a, Root: Data> {
    pub data: &'a Root,
    pub addr: Option<Root::Address>,
    pub change: Change,
}

// #26925 impl
impl<'a, Root: Data> Clone for Revision<'a, Root> {
    fn clone(&self) -> Self {
        Revision {
            addr: self.addr.clone(),
            data: self.data,
            change: self.change,
        }
    }
}

//
impl<'a, Root: Data> Revision<'a, Root> {
    /*pub fn focus<K, R, F>(&self, lens: K, f: F) -> R where
        K: Lens<Source = Root>,
        F: FnOnce(&Revision<K::Target>) -> R
    {
        if let Change::Replace = self.change {
            lens.with(self.data, |data| {
                f(&Revision {
                    change: self.change,
                    addr: None,
                    data
                })
            })
        } else {
            self.addr.clone().and_then(|addr| {

                lens.unprefix(addr).map(|suffix| Revision {
                    addr: suffix,
                    change: self.change,
                    data: lens.get(self.data),
                })
            })
        }
    }*/

    pub fn replaced(&self) -> bool {
        self.change == Change::Replace
    }

    pub fn collection_changes(&self) -> Option<&CollectionChanges> {
        match &self.change {
            Change::Collection(changes) => Some(changes),
            _ => None,
        }
    }
}

impl<'a, Root: Data> From<&'a Root> for Revision<'a, Root> {
    fn from(data: &'a Root) -> Self {
        Revision {
            data,
            change: Change::Replace,
            addr: None,
        }
    }
}

/*
impl<'a, Root> Revision<'a, Root>
    where
        Root: Data + Collection,
        Root::Address: IndexAddress<Element = Root::Element, Index = Root::Index>,
{
    pub fn focus_index(&self, index: Root::Index) -> Option<Revision<'a, Root::Element>> {
        self.focus(<Root as Collection>::at_index(index))
    }
}
*/
