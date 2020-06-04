use crate::event::Event;
use crate::layout::{BoxConstraints, Measurements};
use crate::{Bounds, Widget, LayoutCtx, Visual, DummyVisual, Size, TypedWidget, Environment};
use generational_indextree::NodeId;
use std::any::Any;
use std::any::TypeId;

/// Dummy widget that does nothing.
pub struct DummyWidget;

impl<A: 'static> TypedWidget<A> for DummyWidget {
    type Visual = DummyVisual;

    fn layout(
        self,
        _context: &mut LayoutCtx<A>,
        _previous_visual: Option<Box<DummyVisual>>,
        _constraints: &BoxConstraints,
        _env: Environment,
    ) -> (Box<DummyVisual>, Measurements)
    {
        (Box::new(DummyVisual), Measurements::new(Size::new(0.0,0.0)))
    }
}
