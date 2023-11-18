use std::any::Any;

use kurbo::Point;

use crate::{
    debug_util::DebugWriter, BoxConstraints, ChangeFlags, Element, ElementId, Event, EventCtx, Geometry, HitTestResult,
    LayoutCtx, PaintCtx, TreeCtx, Widget,
};

/// A widget that does nothing.
#[derive(Copy, Clone, Default)]
pub struct Null;

////////////////////////////////////////////////////////////////////////////////////////////////////

impl Widget for Null {
    type Element = NullElement;

    fn build(self, _cx: &mut TreeCtx, _element_id: ElementId) -> Self::Element {
        NullElement
    }

    fn update(self, _ctx: &mut TreeCtx, _node: &mut Self::Element) -> ChangeFlags {
        // nothing to update
        ChangeFlags::empty()
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct NullElement;

impl Element for NullElement {
    fn id(&self) -> ElementId {
        ElementId::ANONYMOUS
    }

    fn layout(&mut self, _ctx: &mut LayoutCtx, _params: &BoxConstraints) -> Geometry {
        Geometry::ZERO
    }

    fn event(&mut self, _ctx: &mut EventCtx, _event: &mut Event) -> ChangeFlags {
        ChangeFlags::NONE
    }

    fn natural_width(&mut self, _height: f64) -> f64 {
        0.0
    }

    fn natural_height(&mut self, _width: f64) -> f64 {
        0.0
    }

    fn natural_baseline(&mut self, _params: &BoxConstraints) -> f64 {
        0.0
    }

    fn hit_test(&self, _ctx: &mut HitTestResult, _position: Point) -> bool {
        false
    }

    fn paint(&mut self, _ctx: &mut PaintCtx) {}

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn debug(&self, w: &mut DebugWriter) {
        w.type_name("NullElement")
    }
}
