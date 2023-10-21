use crate::{
    widget::Axis, ChangeFlags, Element, Environment, Event, EventCtx, Geometry, HitTestResult, LayoutCtx, LayoutParams,
    PaintCtx, RouteEventCtx, TreeCtx, Widget, WidgetId,
};
use kurbo::Point;
use std::any::Any;

/// A widget that does nothing.
#[derive(Copy, Clone, Default)]
pub struct Null;

////////////////////////////////////////////////////////////////////////////////////////////////////

impl Widget for Null {
    type Element = NullElement;

    fn id(&self) -> WidgetId {
        WidgetId::ANONYMOUS
    }

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
    fn id(&self) -> WidgetId {
        WidgetId::ANONYMOUS
    }

    fn layout(&mut self, _ctx: &mut LayoutCtx, _params: &LayoutParams) -> Geometry {
        Geometry::ZERO
    }

    fn event(&mut self, ctx: &mut EventCtx, event: &mut Event) -> ChangeFlags {
        ChangeFlags::NONE
    }

    fn route_event(&mut self, ctx: &mut RouteEventCtx, _event: &mut Event) -> ChangeFlags {
        ChangeFlags::NONE
    }

    fn natural_size(&mut self, axis: Axis, params: &LayoutParams) -> f64 {
        0.0
    }

    fn natural_baseline(&mut self, params: &LayoutParams) -> f64 {
        0.0
    }

    fn hit_test(&self, ctx: &mut HitTestResult, position: Point) -> bool {
        false
    }

    fn paint(&mut self, _ctx: &mut PaintCtx) {}

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
