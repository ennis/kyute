use crate::{
    composable, elem_node::TransformNode, AnyWidget, ChangeFlags, Element, Environment, Event, EventCtx, Geometry,
    HitTestResult, LayoutCtx, LayoutParams, PaintCtx, RouteEventCtx, TreeCtx, Widget, WidgetId,
};
use kurbo::Point;
use std::any::Any;

fn reconcile_elements(
    cx: &mut TreeCtx,
    widgets: Vec<Box<dyn AnyWidget>>,
    elements: &mut Vec<TransformNode<Box<dyn Element>>>,
    env: &Environment,
) -> ChangeFlags {
    let mut pos = 0;
    let mut change_flags = ChangeFlags::empty();
    for widget in widgets {
        let id = Widget::id(&widget);
        // find element matching ID and type
        let element_type_id = widget.element_type_id();
        let found = elements[pos..]
            .iter_mut()
            .position(|elem| elem.id() == id && Any::type_id(elem.content.as_any_mut()) == element_type_id);
        if let Some(found) = found {
            // rotate element in place
            elements[pos..].rotate_left(found);
            // and update it
            change_flags |= elements[pos].update(cx, widget, env);
            pos += 1;
        } else {
            // insert new item
            elements.insert(pos, TransformNode::new(widget.build(cx, env)));

            if id != WidgetId::ANONYMOUS {
                cx.child_added(id);
            }
            change_flags |= ChangeFlags::STRUCTURE;
        }
    }

    if pos < elements.len() {
        // there are elements to be removed
        for elem in &elements[pos..] {
            let id = elem.id();
            if id != WidgetId::ANONYMOUS {
                cx.child_removed(id);
            }
        }
        elements.truncate(pos);
        change_flags |= ChangeFlags::STRUCTURE;
    }

    change_flags
}

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
        reconcile_elements(cx, self.content, &mut node.content, env)
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
