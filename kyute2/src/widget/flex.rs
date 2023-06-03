use crate::{
    composable, elem_node::ElementNode, vec_diff::Patch, Alignment, AnyWidget, ChangeFlags, Element, Environment,
    Geometry, LayoutCtx, LayoutParams, TreeCtx, Widget, WidgetId, WidgetNode,
};
use std::any::Any;

fn reconcile_elements(
    cx: &mut TreeCtx,
    widgets: Vec<WidgetNode<Box<dyn AnyWidget>>>,
    elements: &mut Vec<ElementNode<Box<dyn Element>>>,
    env: &Environment,
) -> ChangeFlags {
    let mut pos = 0;
    let mut change_flags = ChangeFlags::empty();
    for widget in widgets {
        let id = widget.id();
        let found = elements[pos..].iter().position(|elem| elem.id() == Some(id));
        if let Some(found) = found {
            // rotate element in place
            elements[pos..].rotate_left(found);
            // and update it
            change_flags |= widget.update(cx, &mut elements[pos], env);
        } else {
            // insert new item
            elements.insert(pos, widget.build(cx, env));
            change_flags |= ChangeFlags::STRUCTURE;
        }

        pos += 1;
    }

    if pos < elements.len() {
        // there are elements to be removed
        elements.truncate(pos);
        change_flags |= ChangeFlags::STRUCTURE;
    }

    change_flags
}

pub struct VBox {
    content: Vec<WidgetNode<Box<dyn AnyWidget>>>,
}

impl VBox {
    pub fn new() -> VBox {
        VBox { content: vec![] }
    }

    #[composable]
    pub fn push(&mut self, widget: impl Widget + 'static) {
        self.content.push(WidgetNode::new(Box::new(widget)))
    }
}

impl Widget for VBox {
    type Element = VBoxElement;

    fn build(self, cx: &mut TreeCtx, env: &Environment) -> Self::Element {
        let content: Vec<_> = self.content.into_iter().map(|widget| widget.build(cx, env)).collect();
        VBoxElement { content }
    }

    fn update(self, cx: &mut TreeCtx, node: &mut Self::Element, env: &Environment) -> ChangeFlags {
        reconcile_elements(cx, self.content, &mut node.content, env)
    }
}

pub struct VBoxElement {
    content: Vec<ElementNode<Box<dyn Element>>>,
}

impl Element for VBoxElement {
    fn id(&self) -> Option<WidgetId> {
        None
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, params: &LayoutParams) -> Geometry {
        Geometry::ZERO
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
