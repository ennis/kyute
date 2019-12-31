#![feature(raw)]
#![feature(stmt_expr_attributes)]
#![feature(const_cstr_unchecked)]
#![feature(const_fn)]

#[macro_use]
mod util;
#[macro_use]
mod signal;
mod application;
pub mod view;

pub use application::Application;
pub use miniqt_sys;

/*pub trait Widget: mopa::Any {
    //unsafe fn qwidget(&self) -> *mut QWidget;
}
mopafy!(Widget);*/

/*pub trait Container {
    fn get(&self, index: usize) -> Option<&dyn Any>;
    fn get_mut(&mut self, index: usize) -> Option<&mut dyn Any>;
}

pub trait Accepts<W>: Container {
    fn insert(&mut self, index: usize, new: W);
}

pub struct Context<'a, C: Container> {
    container: &'a mut C,
    index: usize,
}

impl<'a, C: Container> Context<'a, C>
{
    pub fn get<W>(&mut self) -> &mut W
        where
            W: Default + 'static,
            C: Accepts<W>
    {
        let i = self.index;
        let present = self.container.get(i).map(|x| x.downcast_ref::<W>()).is_some();
        if !present {
            // create it and insert it into the container at the current index
            self.container.insert(i, W::default());
        }
        self.index += 1;
        self.container.get_mut(i).map(|x| x.downcast_mut::<W>().unwrap()).unwrap()
    }
}*/

/*impl Container for VBox {
    fn get(&self, index: usize) -> Option<&dyn Any> {
        self.nodes.get(index).map(|x| x.deref())
    }

    fn get_mut(&mut self, index: usize) -> Option<&mut dyn Any> {
        self.nodes.get_mut(index).map(|x| x.deref_mut())
    }
}

impl<T: Any> Accepts<T> for VBox {
    fn insert(&mut self, index: usize, new: T) {
        self.nodes.insert(index, Box::new(new))
    }
}*/

/*impl Container for Root {
    fn get(&self, index: usize) -> Option<&dyn Any> {
        if index != 0 { None } else {
            self.node.as_ref().map(|x| x.deref())
        }
    }

    fn get_mut(&mut self, index: usize) -> Option<&mut dyn Any> {
        if index != 0 { None } else {
            self.node.as_mut().map(|x| x.deref_mut())
        }
    }
}

impl<T: Any> Accepts<T> for Root {
    fn insert(&mut self, index: usize, new: T) {
        assert_eq!(index, 0);
        self.node.replace(Box::new(new));
    }
}*/

/*
fn label<S, K>(cx: &Context<S>, lens: K) where
    K: Lens<C::State, String>
{
    // get label in context,
    // context has access to parent (VBox), will create the
    cx.get::<Label>(|cx, label| {
        cx.focus(lens, |text| {
            label.set_text(text);
        });
    });


    if cx.create() {
        eprintln!("label create");
    } else if cx.update() {
        eprintln!("label update");
    }
}*/
