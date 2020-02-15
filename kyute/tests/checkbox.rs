#![feature(specialization)]
use kyute::miniqt_sys::*;
use kyute::model::LensIndexExt;
use kyute::model::{Data, IdentityLens, Lens, State};
use std::marker::PhantomData;
use std::rc::Rc;

use kyute::view as v;
use kyute::view::{CheckboxState, ViewExt};

#[test]
fn test_checkbox() {
    let b = false;
    let mut db = State::new(b);

    // type inference does not work...
    // #41078

    #[derive(Copy,Clone,Debug)]
    enum Action {
        A(CheckboxState),
        B(CheckboxState),
    }

    let root = v::Root::new(v::VBox::new((
        v::Checkbox::new(
            |state: &_| "Checkbox A".to_string(),
            |state: &_| CheckboxState::Checked,
        )
        .map(Action::A),
        v::Checkbox::new(
            |state: &_| "Checkbox B".to_string(),
            |state: &_| CheckboxState::Checked,
        )
        .map(Action::B),
    )));

    db.add_watcher(root.clone());

    while !root.exited() {
        for a in root.run() {
            eprintln!("action {:?}", a);
        }
    }
}

#[test]
fn test_checkbox_with_binding() {
    let b = false;
    let mut db = State::new(b);

    /*let root = kyv::Root::new(kyv::Binding::new(
        kyv::Checkbox::new("test"),
        |checkbox, checked| {
            if *checked.data() {
                checkbox.set_label("checked");
            } else {
                checkbox.set_label("unchecked");
            }
        },
    ));

    db.add_watcher(root.clone());

    while !root.exited() {
        for a in root.run() {
            match a {
                kyv::CheckboxState::Checked | kyv::CheckboxState::PartiallyChecked => {
                    db.replace(IdentityLens::new(), true)
                }
                kyv::CheckboxState::Unchecked => db.replace(IdentityLens::new(), false),
            }
        }
    }*/
}
