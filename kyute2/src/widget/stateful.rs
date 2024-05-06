use crate::{
    state::State, BoxConstraints, ChangeFlags, ContextDataHandle, Event, Geometry, HitTestResult, LayoutCtx, PaintCtx,
    TreeCtx, Widget, WidgetPod, WidgetPtr,
};
use kurbo::Point;
use std::cell::RefCell;

pub struct Stateful<T, F> {
    state: State<T>,
    inner: RefCell<Option<WidgetPtr>>,
    builder: F,
}

impl<T, F, W> Stateful<T, F>
where
    T: 'static,
    W: Widget + 'static,
    F: FnMut(&mut TreeCtx, State<T>) -> W,
{
    pub fn new(initial_data: T, builder: F) -> Stateful<T, F> {
        Stateful {
            state: State::new(initial_data),
            inner: RefCell::new(None),
            builder,
        }
    }
}

//

impl<T, W, F> Widget for Stateful<T, F>
where
    W: Widget + 'static,
    F: Fn(&mut TreeCtx, State<T>) -> W,
    T: 'static,
{
    fn update(&self, cx: &mut TreeCtx) {
        self.inner.replace_with(|inner| {
            let widget: WidgetPtr = WidgetPod::new((self.builder)(cx, self.state.clone()));
            widget.update(cx);
            Some(widget)
        });
    }

    fn event(&self, cx: &mut TreeCtx, event: &mut Event) {}

    fn hit_test(&self, result: &mut HitTestResult, position: Point) -> bool {
        if let Some(ref inner) = &*self.inner.borrow() {
            inner.hit_test(result, position)
        } else {
            false
        }
    }

    fn layout(&self, cx: &mut LayoutCtx, bc: &BoxConstraints) -> Geometry {
        if let Some(ref inner) = &*self.inner.borrow() {
            inner.layout(cx, bc)
        } else {
            Default::default()
        }
    }

    fn paint(&self, cx: &mut PaintCtx) {
        if let Some(ref inner) = &*self.inner.borrow() {
            inner.paint(cx);
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

/*
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
*/
