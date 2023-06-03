use crate::{ChangeFlags, Element, Environment, Geometry, LayoutCtx, LayoutParams, TreeCtx, Widget, WidgetId};
use std::any::Any;

/// A widget that does nothing.
#[derive(Copy, Clone, Default)]
pub struct Null;

////////////////////////////////////////////////////////////////////////////////////////////////////

impl Widget for Null {
    type Element = NullElement;

    fn build(self, _ctx: &mut TreeCtx, _env: &Environment) -> Self::Element {
        NullElement
    }

    fn update(self, _ctx: &mut TreeCtx, _node: &mut Self::Element, _env: &Environment) -> ChangeFlags {
        // nothing to update
        ChangeFlags::empty()
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct NullElement;

impl Element for NullElement {
    fn id(&self) -> Option<WidgetId> {
        None
    }

    fn layout(&mut self, _ctx: &mut LayoutCtx, _params: &LayoutParams) -> Geometry {
        Geometry::ZERO
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
