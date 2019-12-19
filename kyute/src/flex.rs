use crate::util::{Inherits, MaybeOwned};
use crate::{ensure_qt_initialized, Widget};
use miniqt_sys::*;
use std::ptr;

impl_inherits!(QLayout: QObject);
impl_inherits!(QBoxLayout: QLayout);
impl_inherits!(QVBoxLayout: QBoxLayout);
impl_inherits!(QHBoxLayout: QBoxLayout);
impl_inherits!(QVBoxLayout: QLayout);
impl_inherits!(QHBoxLayout: QLayout);

//--------------------------------------------------------------------------------------------------

pub struct Column {
    widget: MaybeOwned<QWidget>,
    layout: *mut QVBoxLayout,
}

/*
impl<'a, T, Action> Widget for Column<'a, T, Action> {
    type State = T;
    type Action = Vec<Action>;

    fn event(&mut self, ctx: &mut EventCtx, state: &mut T, event: &Event) -> Option<Vec<Action>> {
        if let Event::Show = event {
            unsafe { QWidget_show(self.widget.as_raw()) }
        } else {
            for c in self.children.iter_mut() {
                c.event(ctx, state, event);
            }
        }
        None
    }

    fn update(&mut self, ctx: &mut UpdateCtx, state: &T) {
        for c in self.children.iter_mut() {
            c.update(ctx, state)
        }
    }

    fn paint(&mut self, ctx: &mut PaintCtx, state: &T) {
        for c in self.children.iter_mut() {
            c.paint(ctx, state)
        }
    }

    fn mount(&mut self, ctx: &mut UpdateCtx) {
        for c in self.children.iter_mut() {
            c.mount(ctx)
        }
    }

    fn qwidget(&mut self) -> Option<*mut QWidget> {
        self.widget.as_raw::<QWidget>().into()
    }
}

impl<'a, T, A> Column<'a, T, A> {
    pub fn new() -> Column<'a, T, A> {
        ensure_qt_initialized();
        // TODO safety
        unsafe {
            let widget = QWidget_new();
            let layout = QVBoxLayout_new();
            QWidget_setLayout(widget, Inherits::<QLayout>::upcast(layout));

            Column {
                widget: MaybeOwned::owned(widget),
                layout,
                children: Vec::new(),
            }
        }
    }

    pub fn add(&mut self, mut child: impl Widget<State = T, Action = A> + 'a) {
        let pod = WidgetPod::new(child);
        let qwidget = pod.qwidget();
        self.children.push(pod);
        // TODO safety
        unsafe {
            QBoxLayout_addWidget(Inherits::upcast(self.layout), qwidget, 0, Default::default());
        }
    }
}
*/