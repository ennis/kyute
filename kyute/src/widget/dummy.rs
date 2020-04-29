use crate::event::Event;
use crate::layout::{BoxConstraints, Layout};
use crate::renderer::Theme;
use crate::visual::{DummyVisual, NodeData, PaintCtx, Visual};
use crate::visual::{NodeArena, NodeCursor};
use crate::widget::{LayoutCtx, Widget};
use crate::Bounds;
use generational_indextree::NodeId;
use std::any::Any;

/// Dummy widget that does nothing.
pub struct DummyWidget;

impl<A: 'static> Widget<A> for DummyWidget {
    fn layout(
        self,
        _ctx: &mut LayoutCtx<A>,
        nodes: &mut NodeArena,
        cursor: &mut NodeCursor,
        _constraints: &BoxConstraints,
        _theme: &Theme,
    ) -> NodeId {
        cursor.get_or_insert_default::<DummyVisual>(nodes)
    }
}
