use crate::{
    Alignment, ChangeFlags, Element, Environment, Geometry, LayoutCtx, LayoutParams, TreeCtx, Widget, WidgetId,
};
use std::any::Any;

////////////////////////////////////////////////////////////////////////////////////////////////////

/// Horizontal aligmnent modifier.
pub struct HorizontalAlignment<W>(pub Alignment, pub W);

impl<W: Widget> Widget for HorizontalAlignment<W> {
    type Element = HorizontalAlignmentElement<W::Element>;

    fn build(self, cx: &mut TreeCtx, env: &Environment) -> Self::Element {
        let inner = self.1.build(cx, env);
        HorizontalAlignmentElement(self.0, inner)
    }

    fn update(self, cx: &mut TreeCtx, node: &mut Self::Element, env: &Environment) -> ChangeFlags {
        if node.0 != self.0 {
            cx.relayout()
        }
        self.1.update(cx, &mut node.1, env)
    }
}

pub struct HorizontalAlignmentElement<T>(pub Alignment, pub T);

impl<T: Element> Element for HorizontalAlignmentElement<T> {
    fn id(&self) -> Option<WidgetId> {
        self.1.id()
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, params: &LayoutParams) -> Geometry {
        todo!()
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
