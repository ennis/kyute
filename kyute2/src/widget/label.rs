use crate::{ChangeFlags, Element, Environment, Geometry, LayoutCtx, LayoutParams, TreeCtx, Widget, WidgetId};
use std::any::Any;

/// Simple text label.
#[derive(Clone, Debug, Default)]
pub struct Label(Option<String>);

impl Widget for Label {
    type Element = LabelElement;

    fn build(self, cx: &mut TreeCtx, _env: &Environment) -> Self::Element {
        LabelElement {
            text: self.0.unwrap_or_default(),
        }
    }

    fn update(self, cx: &mut TreeCtx, node: &mut Self::Element, _env: &Environment) -> ChangeFlags {
        if let Some(text) = self.0 {
            if node.text != text {
                node.text = text;
                cx.relayout();
                return ChangeFlags::LAYOUT;
            }
        }
        ChangeFlags::empty()
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct LabelElement {
    text: String,
}

impl Element for LabelElement {
    fn id(&self) -> Option<WidgetId> {
        None
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, params: &LayoutParams) -> Geometry {
        // TODO
        Geometry::ZERO
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
