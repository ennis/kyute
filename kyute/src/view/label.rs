use crate::event::Event;
use crate::model::{Data, Revision};
use crate::paint::RenderContext;
use crate::view::EventCtx;
use crate::view::View;
use std::marker::PhantomData;

pub struct Label<A> {
    text: String,
    _phantom: PhantomData<*const A>,
}

impl<A> Label<A> {
    pub fn new() -> Self {
        Label {
            text: "".into(),
            _phantom: PhantomData,
        }
    }
}

impl<S: Data, A> View<S> for Label<A> {
    type Action = A;

    fn event(&mut self, _e: &Event, _ctx: &mut EventCtx<A>) {
        unimplemented!()
    }

    fn update(&mut self, _rev: &Revision<S>) {}

    fn paint(&mut self, _state: &S, _ctx: &mut RenderContext) -> bool {
        unimplemented!()
    }
}
