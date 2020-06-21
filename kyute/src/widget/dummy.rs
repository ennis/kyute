use crate::event::Event;
use crate::layout::{BoxConstraints, Measurements};
use crate::{Rect, DummyVisual, Environment, LayoutCtx, Size, TypedWidget, Visual, Widget};

/// Dummy widget that does nothing.
pub struct DummyWidget;

impl TypedWidget for DummyWidget {
    type Visual = DummyVisual;

    fn layout(
        self,
        _context: &mut LayoutCtx,
        _previous_visual: Option<Box<DummyVisual>>,
        _constraints: &BoxConstraints,
        _env: Environment,
    ) -> (Box<DummyVisual>, Measurements) {
        (
            Box::new(DummyVisual),
            Measurements::new(Size::new(0.0, 0.0)),
        )
    }
}
