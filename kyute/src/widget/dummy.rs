use crate::event::Event;
use crate::layout::{BoxConstraints, Layout};
use crate::renderer::Theme;
use crate::visual::reconciliation::NodePlace;
use crate::visual::{DummyVisual, Node, PaintCtx, Visual};
use crate::widget::{LayoutCtx, Widget};
use crate::Bounds;
use std::any::Any;

/// Dummy widget that does nothing.
pub struct DummyWidget;

impl<A: 'static> Widget<A> for DummyWidget {
    fn layout<'a>(
        self,
        _ctx: &mut LayoutCtx<A>,
        place: &'a mut dyn NodePlace,
        _constraints: &BoxConstraints,
        _theme: &Theme,
    ) -> &'a mut Node {
        place.get_or_insert_default::<DummyVisual>()
    }
}
