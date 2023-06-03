//! Widget tree manipulation and traversal.
use crate::{composable, context::TreeCtx, elem_node::ElementNode, environment::Environment, Element, WidgetId};
use bitflags::bitflags;
use slotmap::SlotMap;
use tracing::warn;

/*struct TreeNode {
    /// Parent node.
    parent: Option<WidgetId>,
    /// Child widgets of this node (direct descendants).
    children: Vec<WidgetId>,
}

pub(crate) struct Tree {
    pub(crate) nodes: SlotMap<WidgetId, TreeNode>,
}

impl Tree {
    pub(crate) fn new() -> (Tree, WidgetId) {
        let mut nodes = SlotMap::with_key();
        let root = nodes.insert(TreeNode {
            parent: None,
            children: vec![],
        });
        (Tree { nodes }, root)
    }
}*/

////////////////////////////////////////////////////////////////////////////////////////////////////

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct ChangeFlags: u32 {
        /// Any structural change (child added / removed). Implies layout and paint.
        const STRUCTURE = 0b00000001;
        /// Layout has changed and needs to be recalculated. Implies paint.
        const LAYOUT = 0b00000010;
        /// Element must be repainted.
        const PAINT = 0b00000100;
    }
}

/// New widget trait
pub trait Widget {
    type Element: Element;

    /// Creates the associated widget node.
    fn build(self, cx: &mut TreeCtx, env: &Environment) -> Self::Element;

    /// Updates an existing widget node.
    fn update(self, cx: &mut TreeCtx, element: &mut Self::Element, env: &Environment) -> ChangeFlags;
}

/// Type-erased widget.
pub trait AnyWidget {
    /// Creates the associated widget node.
    fn build(self: Box<Self>, cx: &mut TreeCtx, env: &Environment) -> Box<dyn Element>;

    /// Updates an existing widget node.
    fn update(self: Box<Self>, cx: &mut TreeCtx, element: &mut Box<dyn Element>, env: &Environment) -> ChangeFlags;
}

impl<W, T> AnyWidget for W
where
    W: Widget<Element = T>,
    T: Element,
{
    fn build(self: Box<Self>, cx: &mut TreeCtx, env: &Environment) -> Box<dyn Element> {
        Box::new(Widget::build(*self, cx, env))
    }

    fn update(self: Box<Self>, cx: &mut TreeCtx, element: &mut Box<dyn Element>, env: &Environment) -> ChangeFlags {
        if let Some(element) = element.as_any_mut().downcast_mut::<T>() {
            Widget::update(*self, cx, element, env)
        } else {
            // not the same type, discard and rebuild
            *element = self.build(cx, env);
            ChangeFlags::STRUCTURE
        }
    }
}

impl Widget for Box<dyn AnyWidget> {
    type Element = Box<dyn Element>;

    fn build(self, cx: &mut TreeCtx, env: &Environment) -> Self::Element {
        AnyWidget::build(self, cx, env)
    }

    fn update(self, cx: &mut TreeCtx, element: &mut Self::Element, env: &Environment) -> ChangeFlags {
        AnyWidget::update(self, cx, element, env)
    }
}

/// Container for a widget that gives it an identifier.
pub struct WidgetNode<T> {
    id: WidgetId,
    content: T,
}

impl<T> WidgetNode<T> {
    /// Creates a new WidgetNode, assigning an ID derived from its position in the current call trace.
    #[composable]
    pub fn new(content: T) -> WidgetNode<T> {
        let id = WidgetId::here();
        WidgetNode { id, content }
    }

    pub fn id(&self) -> WidgetId {
        self.id
    }
}

// Would like to be T: ?Sized, but this would make Self::Element: ?Sized, and build() wouldn't work
impl<T: Widget> Widget for WidgetNode<T> {
    type Element = ElementNode<T::Element>;

    fn build(self, cx: &mut TreeCtx, env: &Environment) -> Self::Element {
        ElementNode::new(self.id, self.content.build(cx, env))
    }

    fn update(self, cx: &mut TreeCtx, element: &mut Self::Element, env: &Environment) -> ChangeFlags {
        self.content.update(cx, &mut element.content, env)
    }
}
