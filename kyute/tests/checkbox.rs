#![feature(specialization)]
use druid_shell::WindowBuilder;
use kyute::dispatch::Dispatcher;
use kyute::model::State;
use kyute::view as v;
use kyute::view::{CheckboxState, ViewExt};
use kyute::window::ViewWindowHandler;

use std::rc::Rc;

#[test]
fn test_checkbox() {
    druid_shell::Application::init();
    let b = false;
    let state = Rc::new(State::new(b));

    // type inference does not work...
    // #41078

    #[derive(Copy, Clone, Debug)]
    enum Action {
        A(CheckboxState),
        B(CheckboxState),
    }

    let dsp = Dispatcher::new(move |action: Action| {
        eprintln!("action {:?}", action);
    });

    let mut w = WindowBuilder::new();
    w.set_title("test");
    w.set_handler(Box::new(ViewWindowHandler::new(
        state.clone(),
        dsp.clone(),
        v::VBox::new((
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
        )),
    )));

    let win = w.build().expect("could not open main window");
    win.show();

    let mut run_loop = druid_shell::RunLoop::new();
    run_loop.run();
}

#[test]
fn test_checkbox_with_binding() {
    let b = false;
    let db = State::new(b);

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
