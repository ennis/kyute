use crate::{
    event::Event,
    layout::{BoxConstraints, Measurements},
    DummyVisual, Environment, LayoutCtx, Rect, Size, TypedWidget, Visual, Widget,
};

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
