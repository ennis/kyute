//! Widget tree manipulation and traversal.
use crate::{context::TreeCtx, environment::Environment, Element, WidgetId};
use bitflags::bitflags;
use std::any::TypeId;

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
        const NONE = 0;
        /// Any structural change (child added / removed).
        const STRUCTURE = (1 << 0);
        /// The size of the element has changed.
        const SIZE = (1 << 1);
        /// The positioning information of the element has changed (alignment).
        const POSITIONING = (1<<2);
        /// Geometry has changed (SIZE | POSITIONING)
        const GEOMETRY = Self::SIZE.bits() | Self::POSITIONING.bits();
        /// Element must be repainted.
        const PAINT = (1<<3);

        /// Child geometry may have changed.
        const CHILD_GEOMETRY = (1<<4);

        /// (Layout) constraints have changed.
        const LAYOUT_CONSTRAINTS = (1<<5);
        /// (Layout) child positions within the parent may have changed.
        const LAYOUT_CHILD_POSITIONS = (1<<7);

        /// The baseline of the element has changed.
        const BASELINE_CHANGED = (1<<8);

        // FIXME: the POSITIONING and SIZE flags are useless since if any of these changes we must call `layout`
        // on the child anyway to retrieve the latest size or alignment.
        // Technically, we could optimize the case where the child knows that only the positioning info has changed and not
        // its size, so that the parent can

        const ALL = 0xFFFF;
    }
}

/// New widget trait
pub trait Widget {
    type Element: Element;

    /// Returns this widget's ID, if it has one.
    fn id(&self) -> WidgetId;

    /// Creates the associated widget node.
    fn build(self, cx: &mut TreeCtx, env: &Environment) -> Self::Element;

    /// Updates an existing widget node.
    ///
    /// # Return value
    ///
    /// A set of change flags:
    /// - GEOMETRY: the geometry of the element might have changed
    fn update(self, cx: &mut TreeCtx, element: &mut Self::Element, env: &Environment) -> ChangeFlags;
}

/// Type-erased widget.
pub trait AnyWidget {
    /// Returns this widget's ID, if it has one.
    fn id(&self) -> WidgetId;

    /// Returns the produced element type ID.
    fn element_type_id(&self) -> TypeId;

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
    fn id(&self) -> WidgetId {
        Widget::id(self)
    }

    fn element_type_id(&self) -> TypeId {
        TypeId::of::<T>()
    }

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

    fn id(&self) -> WidgetId {
        AnyWidget::id(self)
    }

    fn build(self, cx: &mut TreeCtx, env: &Environment) -> Self::Element {
        AnyWidget::build(self, cx, env)
    }

    fn update(self, cx: &mut TreeCtx, element: &mut Self::Element, env: &Environment) -> ChangeFlags {
        AnyWidget::update(self, cx, element, env)
    }
}

/*/// Container for a widget.
pub struct WidgetNode<T> {
    content: T,
    id: WidgetId,
}

impl<T> WidgetNode<T> {
    /// Creates a new WidgetNode.
    #[composable]
    pub fn new(content: T) -> WidgetNode<T> {
        WidgetNode {
            content,
            id: WidgetId::here(),
        }
    }
}

// Would like to be T: ?Sized, but this would make Self::Element: ?Sized, and build() wouldn't work
impl<T: Widget> Widget for WidgetNode<T> {
    type Element = ElementNode<T::Element>;

    fn build(self, cx: &mut TreeCtx, env: &Environment) -> Self::Element {
        ElementNode::new(self.content.build(cx, env))
    }

    fn update(self, cx: &mut TreeCtx, element: &mut Self::Element, env: &Environment) -> ChangeFlags {
        element.update(cx, self, env)
    }
}
*/
