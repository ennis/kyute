/*
pub struct Root<S: Data, V: View<S>, H: Fn(V::Action, &State<S>)> {
    state: State<S>,
    view: RefCell<V>,
    handler: RefCell<H>,
}

impl<S: Data, V: View<S>> Root<S, V>
{
    pub fn new(state: S, view: V, handler: F) -> Rc<Root<S, V>>
    {
        let r = Rc::new(Root {
            state,
            view: RefCell::new(view),
            exited: Cell::new(false),
        });

        // this does not create a reference loop because State holds only weak refs
        r.state.add_watcher(r.clone());
        r
    }

    pub fn exited(&self) -> bool {
        self.exited.get()
    }
}

//
// Window<S>

impl<S: Data, V: View<S>> Watcher<S> for Root<S, V> {
    fn on_change(&self, revision: &Revision<S>) {
        self.view.borrow_mut().update(revision);
    }
}
*/
