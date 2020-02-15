use crate::model::{Data, Revision};
use crate::util::Ptr;
use crate::view::{ActionCtx, View, ViewCollection};
use miniqt_sys::*;
use std::marker::PhantomData;

pub struct VBox<S: Data, V: ViewCollection<S>> {
    widget: Option<Ptr<QWidget>>,
    layout: Option<Ptr<QVBoxLayout>>,
    contents: V,
    _phantom: PhantomData<S>,
}

impl<S: Data, V: ViewCollection<S>> VBox<S, V> {
    pub fn new(contents: V) -> VBox<S, V> {
        VBox {
            widget: None,
            layout: None,
            contents,
            _phantom: PhantomData,
        }
    }

    pub fn contents(&self) -> &V {
        &self.contents
    }

    pub fn contents_mut(&mut self) -> &mut V {
        // TODO watch for changes (insertions/deletions)
        &mut self.contents
    }
}

impl<S: Data, V: ViewCollection<S>> View<S> for VBox<S, V> {
    type Action = V::Action;

    fn update(&mut self, s: &Revision<S>) {
        self.contents.update(s)
    }

    fn mount(&mut self, actx: ActionCtx<Self::Action>) {
        unsafe {
            let widget = Ptr::new(QWidget_new());
            let layout = Ptr::new(QVBoxLayout_new());
            QLayout_setContentsMargins(layout.upcast().as_ptr(), 0, 0, 0, 0);
            QWidget_setLayout(widget.as_ptr(), layout.upcast().as_ptr());

            self.contents.mount(actx);
            let widgets = self.contents.widgets();

            for w in widgets.iter() {
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
