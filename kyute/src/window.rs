//! Window handles.
//! The window does not disappear when it goes out of scope, you need to call "close" for that.

use crate::dispatch::DispatcherHandle;
use crate::model::{Data, Revision, State, Watcher};
use crate::paint::RenderContext;
use crate::view::View;
use druid_shell::{Application, WinCtx};
use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

struct StateWrapper<V> {
    view: RefCell<V>,
}

impl<V> StateWrapper<V> {
    fn new(view: V) -> StateWrapper<V> {
        StateWrapper {
            view: RefCell::new(view),
        }
    }
}

impl<S: Data, V: View<S>> Watcher<S> for StateWrapper<V> {
    fn on_change(&self, revision: &Revision<S>) {
        self.view.borrow_mut().update(revision);
    }
}

pub struct ViewWindowHandler<S: Data, V, A> {
    wrapper: Rc<StateWrapper<V>>,
    state: Rc<State<S>>,
    dispatcher: DispatcherHandle<A>,
    handle: Option<druid_shell::WindowHandle>,
}

impl<S: Data, V, A> ViewWindowHandler<S, V, A> {
    pub fn new(
        state: Rc<State<S>>,
        dispatcher: DispatcherHandle<A>,
        v: V,
    ) -> ViewWindowHandler<S, V, A> {
        ViewWindowHandler {
            wrapper: Rc::new(StateWrapper::new(v)),
            dispatcher,
            handle: None,
            state,
        }
    }
}

impl<S: Data + 'static, V: View<S> + 'static, A: 'static> druid_shell::WinHandler
    for ViewWindowHandler<S, V, A>
{
    fn connect(&mut self, handle: &druid_shell::WindowHandle) {
        self.handle.replace(handle.clone());
    }

    fn paint(&mut self, piet: &mut RenderContext, _ctx: &mut dyn WinCtx) -> bool {
        let s = &self.state;
        let view = &mut *self.wrapper.view.borrow_mut();
        s.with(|s| view.paint(s, piet))
    }

    fn destroy(&mut self, _ctx: &mut dyn WinCtx) {
        Application::quit()
    }

    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}
