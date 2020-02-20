#[macro_use]
mod property;

mod binding;
mod button;
mod checkbox;
mod dispatch;
mod label;
mod lensed;
mod map;
mod root;
mod tuple;
mod vbox;

use std::fmt::Debug;

use crate::model::{Data, Revision};

pub use button::Button;
pub use button::ButtonAction;
pub use checkbox::Checkbox;
pub use checkbox::CheckboxState;
pub use label::Label;
pub use lensed::Lensed;
pub use map::Map;
pub use property::Property;
pub use property::SimpleProperty;
//pub use root::Root;
use crate::event::Event;
use crate::paint::RenderContext;
pub use vbox::VBox;

pub trait ActionSink<A> {
    fn emit(&mut self, a: A);
}

struct ActionCollector<A> {
    actions: Vec<A>,
}

impl<A> ActionSink<A> for ActionCollector<A> {
    fn emit(&mut self, a: A) {
        self.actions.push(a);
    }
}

pub struct EventCtx<'a, A> {
    actions: &'a mut dyn ActionSink<A>,
}

impl<'a, A> EventCtx<'a, A> {
    pub fn new(actions: &'a mut dyn ActionSink<A>) -> EventCtx<'a, A> {
        EventCtx { actions }
    }

    pub fn emit(&mut self, a: A) {
        self.actions.emit(a);
    }

    pub fn action_sink(&mut self) -> &mut dyn ActionSink<A> {
        self.actions
    }
}

pub trait View<S: Data> {
    type Action;

    fn event(&mut self, e: &Event, a: &mut EventCtx<Self::Action>);
    fn update(&mut self, s: &Revision<S>);
    fn paint(&mut self, state: &S, ctx: &mut RenderContext) -> bool;
}

pub trait ViewCollection<S: Data> {
    type Action;

    fn event(&mut self, e: &Event, a: &mut EventCtx<Self::Action>);
    fn update(&mut self, s: &Revision<S>);
    fn paint(&mut self, state: &S, ctx: &mut RenderContext) -> bool;
}

pub trait ViewExt<S: Data>: View<S> {
    fn map<A, F>(self, closure: F) -> Map<Self, F>
    where
        Self: Sized,
        F: Fn(Self::Action) -> A,
    {
        Map::new(self, closure)
    }
}

impl<S: Data, V: View<S>> ViewExt<S> for V {}
