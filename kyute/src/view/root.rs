use crate::application::{ensure_qt_initialized, ProcessEventFlags};
use crate::util::CBox;
use crate::view::{Action, ActionRoot, View};
use miniqt_sys::*;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use veda::{Data, Revision, Watcher};

pub struct Root<V: View> {
    view: RefCell<V>,
    root_widget: CBox<QWidget>,
    actx: Rc<ActionRoot<V::Action>>,
    exited: Cell<bool>,
}

impl<V: View> Root<V>
{
    pub fn new(mut view: V) -> Rc<Root<V>> {
        ensure_qt_initialized();

        let actx = ActionRoot::new();
        view.mount(actx.clone());

        let root_widget = unsafe { CBox::from_ptr(view.widget_ptr().expect("no widget")) };

        let r = Root {
            view: RefCell::new(view),
            root_widget,
            actx,
            exited: Cell::new(false),
        };

        Rc::new(r)
    }

    pub fn exited(&self) -> bool {
        self.exited.get()
    }

    pub fn run(&self) -> Vec<V::Action> {
        unsafe {
            QWidget_show(self.root_widget.as_raw_ptr());
            let event_loop = QEventLoop_new();

            let actions = loop {
                QEventLoop_processEvents(
                    event_loop,
                    ProcessEventFlags::WAIT_FOR_MORE_EVENTS.bits() as u32,
                );
                let actions = self.actx.collect_actions();

                // If the root widget is not visible anymore, assume that the window has been closed
                // and that we should exit.
                if !QWidget_isVisible(self.root_widget.as_raw_ptr()) {
                    self.exited.set(true);
                    break actions;
                }

                if !actions.is_empty() {
                    break actions;
                }
            };

            QEventLoop_delete(event_loop);
            actions
        }
    }
}

/*impl<S: Data> View<S> for Root<S> {
    fn update(&mut self, rev: Revision<S>) {
        self.view.as_mut().map(|v| v.update(rev.clone()));
    }
}*/

/*
impl<V: View> Watcher<> for Root<S, A> {
    fn on_change(&self, revision: Revision<S>) {
        self.view.borrow_mut().update(revision);
    }
}*/
