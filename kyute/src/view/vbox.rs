use crate::event::Event;
use crate::model::{Data, Revision};
use crate::paint::RenderContext;
use crate::view::{EventCtx, View, ViewCollection};
use std::marker::PhantomData;

pub struct VBox<S: Data, V: ViewCollection<S>> {
    contents: V,
    _phantom: PhantomData<S>,
}

impl<S: Data, V: ViewCollection<S>> VBox<S, V> {
    pub fn new(contents: V) -> VBox<S, V> {
        VBox {
            contents,
            _phantom: PhantomData,
        }
    }

    pub fn contents(&self) -> &V {
        &self.contents
    }

    pub fn contents_mut(&mut self) -> &mut V {
        &mut self.contents
    }
}

impl<S: Data, V: ViewCollection<S>> View<S> for VBox<S, V> {
    type Action = V::Action;

    fn event(&mut self, e: &Event, ctx: &mut EventCtx<V::Action>) {
        self.contents.event(e, ctx)
    }

    fn update(&mut self, s: &Revision<S>) {
        self.contents.update(s)
    }

    fn paint(&mut self, state: &S, ctx: &mut RenderContext) -> bool {
        self.contents.paint(state, ctx)
    }
}
