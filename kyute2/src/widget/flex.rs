use crate::{
    element::TransformNode, widget::Axis, AnyWidget, BoxConstraints, ChangeFlags, Element, ElementId, Event, EventCtx,
    Geometry, HitTestResult, LayoutCtx, PaintCtx, TreeCtx, Widget,
};
use kurbo::Point;
use std::any::Any;

pub struct VBox {
    content: Vec<Box<dyn AnyWidget>>,
}

impl VBox {
    pub fn new() -> VBox {
        VBox { content: vec![] }
    }

    pub fn push(&mut self, widget: impl Widget + 'static) {
        self.content.push(Box::new(widget))
    }
}

impl Widget for VBox {
    type Element = VBoxElement;

    fn build(self, cx: &mut TreeCtx, id: ElementId) -> Self::Element {
        let content: Vec<_> = self
            .content
            .into_iter()
            .map(|widget| TransformNode::new(cx.build(widget)))
            .collect();
        VBoxElement { id, content }
    }

    fn update(self, cx: &mut TreeCtx, node: &mut Self::Element) -> ChangeFlags {
        todo!()
        //reconcile_elements(cx, self.content, &mut node.content, env)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct VBoxElement {
    id: ElementId,
    content: Vec<TransformNode<Box<dyn Element>>>,
}

impl Element for VBoxElement {
    fn id(&self) -> ElementId {
        self.id
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, params: &BoxConstraints) -> Geometry {
        Geometry::ZERO
    }

    fn event(&mut self, ctx: &mut EventCtx, event: &mut Event) -> ChangeFlags {
        if let Some(next_target) = event.next_target() {
            let child = self
                .content
                .iter_mut()
                .find(|e| e.id() == next_target)
                .expect("invalid child specified");
            child.event(ctx, event)
        } else {
            // Nothing
            ChangeFlags::NONE
        }
    }

    fn natural_width(&mut self, height: f64) -> f64 {
        todo!()
    }

    fn natural_height(&mut self, width: f64) -> f64 {
        todo!()
    }

    fn natural_baseline(&mut self, params: &BoxConstraints) -> f64 {
        todo!()
    }

    fn hit_test(&self, ctx: &mut HitTestResult, position: Point) -> bool {
        todo!()
    }

    fn paint(&mut self, ctx: &mut PaintCtx) {
        todo!()
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
