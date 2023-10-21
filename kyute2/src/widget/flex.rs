use crate::{
    composable, elem_node::TransformNode, widget::Axis, AnyWidget, ChangeFlags, Element, Environment, Event, EventCtx,
    Geometry, HitTestResult, LayoutCtx, LayoutParams, PaintCtx, RouteEventCtx, TreeCtx, Widget, WidgetId,
};
use kurbo::Point;
use std::any::Any;

pub struct VBox {
    id: WidgetId,
    content: Vec<Box<dyn AnyWidget>>,
}

impl VBox {
    #[composable]
    pub fn new() -> VBox {
        VBox {
            id: WidgetId::here(),
            content: vec![],
        }
    }

    pub fn push(&mut self, widget: impl Widget + 'static) {
        self.content.push(Box::new(widget))
    }
}

impl Widget for VBox {
    type Element = VBoxElement;

    fn id(&self) -> WidgetId {
        self.id
    }

    fn build(self, cx: &mut TreeCtx, env: &Environment) -> Self::Element {
        let content: Vec<_> = self
            .content
            .into_iter()
            .map(|widget| TransformNode::new(widget.build(cx, env)))
            .collect();
        VBoxElement { id: self.id, content }
    }

    fn update(self, cx: &mut TreeCtx, node: &mut Self::Element, env: &Environment) -> ChangeFlags {
        todo!()
        //reconcile_elements(cx, self.content, &mut node.content, env)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct VBoxElement {
    id: WidgetId,
    content: Vec<TransformNode<Box<dyn Element>>>,
}

impl Element for VBoxElement {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, params: &LayoutParams) -> Geometry {
        Geometry::ZERO
    }

    fn event(&mut self, ctx: &mut EventCtx, event: &mut Event) -> ChangeFlags {
        todo!()
    }

    fn route_event(&mut self, ctx: &mut RouteEventCtx, event: &mut Event) -> ChangeFlags {
        if let Some(next_target) = event.next_target() {
            let child = self
                .content
                .iter_mut()
                .find(|e| e.id() == next_target)
                .expect("invalid child specified");
            child.route_event(ctx, event)
        } else {
            self.event(&mut ctx.inner, event)
        }
    }

    fn natural_size(&mut self, axis: Axis, params: &LayoutParams) -> f64 {
        todo!()
    }

    fn natural_baseline(&mut self, params: &LayoutParams) -> f64 {
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
