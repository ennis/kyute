use crate::util::Ptr;
use crate::view::{Action, ActionCtx, View};
use miniqt_sys::util::Deletable;
use miniqt_sys::*;
use veda::Collection;
use veda::CollectionChanges;
use veda::{Identifiable, IndexAddress, Revision};

/*pub struct Identified<S: Identifiable, A: Action>(S::Id, A);

// #26925 impl
impl<S: Identifiable, A: Action> fmt::Debug for Identified<S,A> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("Identified").field(&self.0).field(&self.1).finish()
    }
}

// #26925 impl
impl<S: Identifiable, A: Action> Clone for Identified<S,A> {
    fn clone(&self) -> Self {
        Identified(self.0.clone(), self.1.clone())
    }
}*/

struct ListNode<S: Identifiable, A: Action> {
    id: S::Id,
    // decorator for easier sorting on relayouts
    ix: usize,
    view: Box<dyn View<S, Action = A>>,
}

pub struct List<S: Identifiable, A: Action> {
    // (id, index (decorator for easier sorting on relayouts), view)
    nodes: Vec<ListNode<S, A>>,
    template: Box<dyn FnMut(S::Id) -> Box<dyn View<S, Action = A>>>,

    actx: Option<ActionCtx<A>>,
    widget: Option<Ptr<QWidget>>,
    layout: Option<Ptr<QVBoxLayout>>,
}

fn delete_child_widget<S: Identifiable, A: Action>(view: &Box<dyn View<S, Action = A>>) {
    view.widget_ptr()
        .map(|w| unsafe { Deletable::delete(w.as_ptr()) });
}

impl<S, C, A> View<C> for List<S, A>
where
    A: Action,
    S: Identifiable,
    C: Collection<Index = usize, Element = S>,
    C::Address: IndexAddress<Index = usize, Element = S>,
{
    type Action = A;

    fn update(&mut self, rev: Revision<C>) {
        assert!(
            self.widget.is_some() && self.layout.is_some() && self.actx.is_some(),
            "not mounted"
        );

        let data = rev.data();

        if rev.replaced() {
            eprintln!("List replace {:?}", rev.address());
            for ListNode { view, .. } in self.nodes.drain(..) {
                delete_child_widget(&view);
            }
            for (ix, d) in data.box_iter().enumerate() {
                eprintln!("List adding {:?}", d.id());
                let id = d.id();
                let mut view = (self.template)(id.clone());
                view.mount(self.actx.clone().unwrap());
                view.update(d.into());
                let node = ListNode { id, ix, view };
                self.nodes.push(node);
            }
            self.rebuild_layout()
        } else if let Some(changes) = rev.collection_changes() {
            eprintln!("List change {:?}", rev.address());
            match changes {
                CollectionChanges::Relayout => {
                    for ListNode { id, ix, .. } in self.nodes.iter_mut() {
                        *ix = data
                            .box_iter()
                            .position(|x| x.id() == *id)
                            .expect("element was removed");
                    }
                    self.nodes.sort_by_key(|ListNode { ix, .. }| *ix);
                    self.rebuild_layout();
                }
                &CollectionChanges::Update { start, end } => {
                    for ListNode { ix, view, .. } in &mut self.nodes[start..end] {
                        view.update(rev.focus_index(*ix).unwrap());
                    }
                }
                &CollectionChanges::Splice {
                    start,
                    remove,
                    insert,
                } => {
                    // comment this and replace `template` with `self.template` in the closure for a cool error message
                    let template = &mut self.template;
                    let actx = &mut self.actx;
                    for ListNode { view, .. } in self.nodes.splice(
                        start..(start + remove),
                        (start..start + insert).map(|ix| {
                            let d = data.get_at(ix).unwrap();
                            let id = d.id();
                            let mut view = (template)(id.clone());
                            view.mount(actx.clone().unwrap());
                            view.update(d.into());
                            ListNode { id, ix, view }
                        }),
                    ) {
                        // manually delete the qwidget, because we have ownership
                        // (rebuild_layout disowns the widgets of the child views).
                        delete_child_widget(&view);
                    }

                    self.rebuild_layout();
                }
            }
        } else {
            // hmmm?
        }
    }

    fn mount(&mut self, actx: ActionCtx<A>) {
        assert!(self.nodes.is_empty());
        assert!(self.widget.is_none());

        self.actx.replace(actx.clone());

        unsafe {
            let widget = Ptr::new(QWidget_new());
            let layout = Ptr::new(QVBoxLayout_new());

            QLayout_setContentsMargins(layout.upcast().as_ptr(), 0, 0, 0, 0);
            QWidget_setLayout(widget.as_ptr(), layout.upcast().as_ptr());

            self.widget.replace(widget);
            self.layout.replace(layout);
        }
    }

    fn widget_ptr(&self) -> Option<Ptr<QWidget>> {
        self.widget
    }
}

impl<S: Identifiable, A: Action> List<S, A> {
    pub fn new(
        template: impl FnMut(S::Id) -> Box<dyn View<S, Action = A>> + 'static,
    ) -> List<S, A> {
        List {
            nodes: Vec::new(),
            template: Box::new(template),
            actx: None,
            widget: None,
            layout: None,
        }
    }

    fn rebuild_layout(&mut self) {
        assert!(
            self.widget.is_some() && self.layout.is_some(),
            "not mounted"
        );

        let layout = self.layout.unwrap();

        // delete layout
        unsafe {
            QVBoxLayout_delete(layout.as_ptr());
            let layout = Ptr::new(QVBoxLayout_new());
            // re-add child widgets to the new layout
            for ListNode { view, .. } in self.nodes.iter_mut() {
                // TODO maybe just ignore children with no widget?
                let w = view.widget_ptr().expect("child has no widget");
                QBoxLayout_addWidget(layout.upcast().as_ptr(), w.as_ptr(), 0, Default::default());
            }
            QLayout_setContentsMargins(layout.upcast().as_ptr(), 0, 0, 0, 0);
            QWidget_setLayout(self.widget.unwrap().as_ptr(), layout.upcast().as_ptr());

            self.layout.replace(layout);
        }
    }
}
