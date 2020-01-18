use crate::util::Ptr;
use crate::view::{Action, ActionCtx, View, ViewCollection};
use miniqt_sys::*;
use veda::{Data, Collection, Identifiable};
use veda::Revision;
use std::ops::RangeBounds;

pub struct VBox<V: ViewCollection>
{
    widget: Option<Ptr<QWidget>>,
    layout: Option<Ptr<QVBoxLayout>>,
    contents: V,
}

impl<V: ViewCollection> VBox<V>
{
    pub fn new(contents: V) -> VBox<V> {
        VBox {
            widget: None,
            layout: None,
            contents,
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

impl<V: ViewCollection> View for VBox<V>
{
    type Action = V::Action;

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
