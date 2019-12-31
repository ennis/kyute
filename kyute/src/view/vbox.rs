use crate::util::Ptr;
use crate::view::{Action, ActionCtx, View};
use miniqt_sys::*;
use veda::{Data, Collection, Identifiable};
use veda::Revision;
use std::ops::RangeBounds;

pub struct VBox<S: Data, A: Action> {
    widget: Option<Ptr<QWidget>>,
    layout: Option<Ptr<QVBoxLayout>>,
    views: Vec<Box<dyn View<S, Action = A>>>,
}

// View has no update() function anymore
// View is not parameterized on the input state anymore
//
/*
pub trait ViewCollection

pub struct VBox2<A: Action, V: ViewCollection<Action=A>> {
    widget: Option<Ptr<QWidget>>,
    layout: Option<Ptr<QVBoxLayout>>,
    contents: V,        // simply want to extract a bunch of QWidgets from it
}
*/


/*
pub trait CollectionView<T>
{
    fn splice<R,I>(&mut self, range: R, replace_with: I)
        where R: RangeBounds<usize>,
              I: IntoIterator<Item = T>;

    fn get(&self) -> Option<&T>;
    fn get_mut(&mut self) -> Option<&mut T>;

    fn sort_by<C>(&mut self, reference: &C) where
        C: Collection<Index=usize>,
        T: Identifiable,
        C::Element: Identifiable<Id=T::Id>;
}

pub trait CollectionChanges {
    fn splice(&mut self, start: usize, remove: usize, insert: usize)
}*/


impl<S: Data, A: Action> VBox<S, A> {
    pub fn new(views: Vec<Box<dyn View<S, Action = A>>>) -> VBox<S, A> {
        VBox {
            views,
            widget: None,
            layout: None,
        }
    }

    /*pub fn label<'a>(&'a mut self) -> impl Property<String> + 'a
    {
        ValueProperty::new( // can borrow self
            /*get*/ || { QLabel_getText() },
            /*set*/ |s| { QLabel_setText(s) }
        );

        vbox.label().set(...);
    }

    pub fn views(&mut self) -> impl CollectionProperty<Box<dyn View<S, Action=A>>> {

        // issue: can't borrow the whole self in the closures
        VecBackedCollectionProperty::new(
            &mut self.views,
            |views, start, end| {      // splice
                rebuild_layout(layout, views);
            },
            |views| {       // sorted
                rebuild_layout(layout, views);
            },
            |update, start, end| {}  // updated
        );
        }*/


        // given a Revision<C>
        // -> transform into a vec of Box<View> + insertion/deletion

        // CollectionView<T>:
        // - splice (insert/remove)
        // - sorting
        // - update (Revision<C>, fn S -> T where C: Collection<S>)
        // + hooks

        // problem:
        // - widgets expect their data to be sorted in some way, which might be different from the
        //   sorting of the source collection

        // define a mapping between a source and a target collection
        // - problem: the target may want to be sorted according to some criterion of the source collection
        //      i.e. not enough data to sort in the target independently of the source
        // - sort_with(&C


}

impl<S: Data, A: Action> View<S> for VBox<S, A> {
    type Action = A;
    fn update(&mut self, rev: Revision<S>) {
        eprintln!("VBox update {:?}", rev.address());

        assert!(self.widget.is_some(), "not mounted");

        for v in self.views.iter_mut() {
            v.update(rev.clone())
        }
    }

    fn mount(&mut self, actx: ActionCtx<A>) {
        unsafe {
            let widget = Ptr::new(QWidget_new());
            let layout = Ptr::new(QVBoxLayout_new());
            QLayout_setContentsMargins(layout.upcast().as_ptr(), 0, 0, 0, 0);
            QWidget_setLayout(widget.as_ptr(), layout.upcast().as_ptr());

            for v in self.views.iter_mut() {
                v.mount(actx.clone());
                let w = v.widget_ptr().expect("no widget");
                QBoxLayout_addWidget(layout.upcast().as_ptr(), w.as_ptr(), 0, Default::default());
            }

            self.widget.replace(widget);
            self.layout.replace(layout);
        }
    }

    fn widget_ptr(&self) -> Option<Ptr<QWidget>> {
        self.widget
    }
}
