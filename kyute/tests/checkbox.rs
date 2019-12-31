#![feature(specialization)]
use kyute::miniqt_sys::*;
use std::marker::PhantomData;
use std::rc::Rc;
use veda::lens::LensIndexExt;
use veda::{Data, Database, Lens, IdentityLens};

use kyute::view as kyv;

#[test]
fn test_checkbox() {
    let b = false;
    let mut db = Database::new(b);

    let root = kyv::Root::new(kyv::Checkbox::new("test"));
    db.add_watcher(root.clone());

    while !root.exited() {
        root.run();
    }
}

#[test]
fn test_checkbox_with_binding()
{
    let b = false;
    let mut db = Database::new(b);

    let root = kyv::Root::new(kyv::Binding::new(kyv::Checkbox::new("test"), |checkbox, checked| {
        if *checked.data() {
            checkbox.set_label("checked");
        } else {
            checkbox.set_label("unchecked");
        }
    }));

    db.add_watcher(root.clone());

    while !root.exited() {
        for a in root.run() {
            match a {
                kyv::CheckboxState::Checked | kyv::CheckboxState::PartiallyChecked => db.replace(IdentityLens::new(), true),
                kyv::CheckboxState::Unchecked => db.replace(IdentityLens::new(), false),
            }
        }
    }
}