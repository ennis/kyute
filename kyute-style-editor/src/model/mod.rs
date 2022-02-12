mod atom;

use crate::model::atom::{make_unique_name, Atom};
use kyute::{
    imbl::{HashMap, HashSet, Vector},
    Color, Data,
};
use std::sync::Arc;

#[derive(Clone, Data)]
pub struct StyleSheet {
    items: HashMap<Atom, Arc<Item>>,
}

impl StyleSheet {
    pub fn new() -> StyleSheet {
        StyleSheet {
            items: Default::default(),
        }
    }

    pub fn set_color(&mut self, item: &Item, new_color: Color) {}

    pub fn items(&self) -> &HashMap<Atom, Arc<Item>> {
        &self.items
    }

    /// Unlinks the value from its source.
    pub fn unlink(&mut self, item: &Item) {
        if let Some(source) = item.source.clone() {
            self.items.get_mut(&item.name).unwrap().source = None;
            self.items
                .get_mut(&source.name)
                .unwrap()
                .dependents
                .remove(&source.name);
        }
    }

    /// Links the value of two items.
    pub fn link(&mut self, source: &Item, dest: &Item) {
        if dest.source.is_some() {
            self.unlink(dest);
        }
        let source = self.items.get_mut(&source.name).unwrap();
        source.dependents.insert(dest.name.clone());
        let source = source.clone();
        self.items.get_mut(&dest.name).unwrap().source = Some(source);
    }

    /// Sets the value of an item.
    pub fn set_item(&mut self, item: &Item, kind: ItemKind, cascade: bool) {
        // unlink source first if this is an explicit assignment
        if !cascade & item.source.is_some() {
            self.unlink(item);
        }

        // update item value
        let new_item = self.items.get_mut(&item.name).unwrap();
        match (kind.clone(), &new_item.kind) {
            (ItemKind::Color(new), ItemKind::Color(ref mut old)) => {
                *old = new;
            }
            _ => panic!("mismatched item types"),
        }

        // update dependents recursively
        let dependents = item.dependents.clone();
        for dep in dependents {
            let dep = self.items.get(&dep).unwrap().clone();
            self.set_item(&dep, kind.clone(), true);
        }
    }

    /// Generates a unique child name from the specified stem.
    fn make_unique_item_name(&self, stem: impl Into<Atom>) -> Atom {
        let mut counter = 0;
        let stem = stem.into();
        let mut unique_name = stem.clone();

        'check: loop {
            // check for property with the same name
            for item in self.items.values() {
                if item.name == unique_name {
                    unique_name = Atom::from(format!("{}_{}", stem, counter));
                    counter += 1;
                    // restart check
                    continue 'check;
                }
            }
            break;
        }

        unique_name
    }

    /// Creates a new item.
    pub fn create_item(&mut self, name: Atom, kind: ItemKind) -> Arc<Item> {
        let name = self.make_unique_item_name(name.clone());
        let item = Arc::new(Item {
            name: name.clone(),
            source: None,
            dependents: Default::default(),
            kind,
        });
        self.items.insert(name.clone(), item.clone());
        item
    }

    /// Removes an item.
    pub fn remove_item(&mut self, item: &Item) {
        for dep in item.dependents.clone() {
            let dep_item = self.items.get(&dep).unwrap().clone();
            self.unlink(&dep_item);
        }
        self.items.remove(&item.name);
    }
}

#[derive(Clone, Data)]
pub struct Item {
    name: Atom,
    source: Option<Arc<Item>>,
    dependents: HashSet<Atom>,
    kind: ItemKind,
}

impl Item {
    pub fn kind(&self) -> &ItemKind {
        &self.kind
    }

    pub fn name(&self) -> &Atom {
        &self.name
    }
}

#[derive(Clone, Data)]
pub enum ItemKind {
    Color(ItemColor),
}

#[derive(Clone, Data)]
pub struct ItemColor {
    pub color: Color,
}
