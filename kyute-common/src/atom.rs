use crate::Data;
use std::{fmt, ops::Deref};
use string_cache::DefaultAtom;

/// Interned strings. Typically used for names and string identifiers.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Default, serde::Serialize, serde::Deserialize)]
pub struct Atom(DefaultAtom);

impl Deref for Atom {
    type Target = DefaultAtom;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Data for Atom {
    fn same(&self, other: &Self) -> bool {
        &self.0 == &other.0
    }
}

impl<T> From<T> for Atom
where
    DefaultAtom: From<T>,
{
    fn from(value: T) -> Self {
        Atom(DefaultAtom::from(value))
    }
}

impl fmt::Display for Atom {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// Helper function to adjust a name so that it doesn't clash with existing names.
pub fn make_unique_atom<'a>(base_name: impl Into<Atom>, existing: impl Iterator<Item = Atom> + Clone) -> Atom {
    let mut counter = 0;
    let base_name = base_name.into();
    let mut disambiguated_name = base_name.clone();

    'check: loop {
        let existing = existing.clone();
        // check for property with the same name
        for name in existing {
            if name == disambiguated_name {
                disambiguated_name = Atom::from(format!("{}_{}", base_name, counter));
                counter += 1;
                // restart check
                continue 'check;
            }
        }
        break;
    }

    disambiguated_name
}
