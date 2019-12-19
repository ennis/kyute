#![feature(raw)]

#[macro_use]
pub mod util;
#[macro_use]
pub mod signal;
pub mod flex;
pub mod application;

pub use miniqt_sys;
pub use self::application::ensure_qt_initialized;

use veda::db::{Database, Data};
use veda::lens::Lens;
use std::collections::HashMap;
use crate::util::Inherits;
use mopa::mopafy;
use miniqt_sys::*;

pub trait View<M: Data> {
    fn sync(&mut self, db: &Database<M>);
}

pub trait Widget: mopa::Any {
    unsafe fn qwidget(&self) -> *mut QWidget;
}

mopafy!(Widget);


struct Flex {
    widget: *mut QWidget,
    layout: *mut QVBoxLayout,
    children: HashMap<veda::lens::ComponentIndex, Box<dyn Widget>>,
}

impl Widget for Flex {
    unsafe fn qwidget(&self) -> *mut QWidget {
        self.widget
    }
}

impl Flex {
    pub fn add<T: Widget>(&mut self, index: veda::lens::ComponentIndex, widget: impl Widget) {
        unsafe {
            QBoxLayout_addWidget(
                Inherits::upcast(self.layout),
                widget.qwidget(),
                0, Default::default());
        }
        self.children.insert(index, Box::new(widget));
    }

    pub fn remove(&mut self, index: veda::lens::ComponentIndex) {
        unimplemented!()
    }

    pub fn child(&self, index: veda::lens::ComponentIndex) -> Option<&dyn Widget> {
        unimplemented!()
    }

    pub fn child_mut(&mut self, index: veda::lens::ComponentIndex) -> Option<&mut dyn Widget> {
        unimplemented!()
    }
}

/*
fn handler(db: &Data, change: &Change) {
    // knows the structure

    // enter vbox

    {
        let vbox = root;
        if let Some(rest) = change.path.starts_with(Data::field) {
            // enter [0]
            let child_0 =

        }
    }


}
*/

// + lens to access child by index
/*struct QtView<M: Model, V: Model> {
    // issue: need to remove handler when the object pointed by the partial path is removed

    handlers: HashMap<PartialPath<M>, Box<dyn Fn(&M, &mut V)>>
}*/
