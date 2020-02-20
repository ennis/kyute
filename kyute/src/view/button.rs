use crate::event::Event;
use crate::model::{Data, Lens, Revision};
use crate::paint::RenderContext;
use crate::view::{EventCtx, View};
use std::marker::PhantomData;

pub struct Button<S: Data, Label: Lens<S, String>> {
    label: Label,
    _phantom: PhantomData<S>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ButtonAction {
    Clicked,
    Released,
}

impl<S: Data, Label: Lens<S, String>> Button<S, Label> {
    pub fn new(label: Label) -> Button<S, Label> {
        Button {
            label,
            _phantom: PhantomData,
        }
    }
}

impl<S: Data, Label: Lens<S, String>> View<S> for Button<S, Label> {
    type Action = ButtonAction;

    fn event(&mut self, _e: &Event, _a: &mut EventCtx<Self::Action>) {
        unimplemented!()
    }

    fn update(&mut self, _state: &Revision<S>) {
        unimplemented!()
    }

    fn paint(&mut self, _state: &S, _ctx: &mut RenderContext) -> bool {
        false
    }
}
