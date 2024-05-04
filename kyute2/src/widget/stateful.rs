use crate::{
    state::State, widget::WidgetVisitor, BoxConstraints, ChangeFlags, ContextDataHandle, Event, Geometry,
    HitTestResult, LayoutCtx, PaintCtx, TreeCtx, Widget, WidgetId,
};
use kurbo::Point;

pub struct Stateful<T, W, F> {
    id: WidgetId,
    state: State<T>,
    inner: Option<W>,
    builder: F,
}

impl<T, W, F> Stateful<T, W, F>
where
    T: 'static,
    W: Widget,
    F: FnMut(&mut TreeCtx, StateHandle<T>) -> W,
{
    pub fn new(initial_data: T, builder: F) -> Stateful<T, W, F> {
        Stateful {
            id: WidgetId::next(),
            state: State::new(initial_data),
            inner: None,
            builder,
        }
    }

    /* pub fn set_state(&mut self, cx: &mut TreeCtx, data: T) {
        self.state.data = data;
    }

    /// Handle a state change event.
    fn handle_state_changed(&mut self, cx: &mut TreeCtx) {
        /*// update the dependents (which may include ourselves)
        let dependents = self.state.dependents.borrow().clone();
        for subtree in dependents.traverse() {
            cx.dispatch(self, subtree, &mut |cx, widget| {
                widget.update(cx);
            });
        }*/
    }*/
}

impl<T, W, F> Widget for Stateful<T, W, F>
where
    W: Widget,
    F: FnMut(&mut TreeCtx, StateHandle<T>) -> W,
    T: 'static,
{
    fn id(&self) -> WidgetId {
        self.id
    }

    fn visit_child(&mut self, cx: &mut TreeCtx, id: WidgetId, visitor: &mut WidgetVisitor) {
        if let Some(ref mut inner) = self.inner {
            cx.with_data(&mut self.state, |cx, _state_handle| {
                inner.visit_child(cx, id, visitor);
            });
        }
    }

    fn update(&mut self, cx: &mut TreeCtx) -> ChangeFlags {
        // rebuild the inner widget
        cx.with_data(&mut self.state, |cx, handle| {
            self.inner.replace((self.builder)(cx, StateHandle(handle)));
        });

        // assume everything changed
        ChangeFlags::ALL
    }

    fn event(&mut self, cx: &mut TreeCtx, event: &mut Event) -> ChangeFlags {
        // We don't do anything with events
        ChangeFlags::NONE
    }

    fn hit_test(&self, result: &mut HitTestResult, position: Point) -> bool {
        if let Some(ref inner) = self.inner {
            result.hit_test_child(inner, position)
        } else {
            false
        }
    }

    fn layout(&mut self, cx: &mut LayoutCtx, bc: &BoxConstraints) -> Geometry {
        if let Some(ref mut inner) = self.inner {
            inner.layout(cx, bc)
        } else {
            Default::default()
        }
    }

    fn paint(&mut self, cx: &mut PaintCtx) {
        if let Some(ref mut inner) = self.inner {
            inner.paint(cx);
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct StateHandle<T>(ContextDataHandle<State<T>>);

impl<T: 'static> StateHandle<T> {
    pub fn get<'a>(&self, cx: &'a TreeCtx) -> &'a T {
        let state = cx.data(self.0);
        state.set_dependency(cx);
        &state.data
    }

    pub fn set(&self, cx: &mut TreeCtx, value: T) {
        let state = cx.data_mut(self.0);
        state.data = value;
        // set dependency afterward because it needs a non-mutable borrow
        let state = cx.data(self.0);
        state.set_dependency(cx);
        state.request_update(cx);
    }
}
