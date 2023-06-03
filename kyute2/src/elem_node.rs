//!
use crate::{
    context::{EventCtx, LayoutCtx},
    Environment, Geometry, LayoutParams, TreeCtx, Widget, WidgetId,
};
use kurbo::{Affine, Point};
use std::any::Any;

pub trait Element: 'static {
    /// Returns this element's ID.
    fn id(&self) -> Option<WidgetId>;

    /// Measures this widget and layouts the children of this widget.
    fn layout(&mut self, ctx: &mut LayoutCtx, params: &LayoutParams) -> Geometry;

    /// Deliver an event to this element or one of its children.
    fn event(&mut self, ctx: &mut EventCtx) {
        // do nothing
    }

    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl Element for Box<dyn Element> {
    fn id(&self) -> Option<WidgetId> {
        (&**self).id()
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, params: &LayoutParams) -> Geometry {
        (&mut **self).layout(ctx, params)
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

/// A container for a widget.
pub struct ElementNode<T: ?Sized = dyn Element> {
    /// Unique ID of the widget, if it has one.
    id: Option<WidgetId>,
    /// Parent-to-local transform.
    transform: Affine,
    /// Inner element
    pub content: T,
}

impl<T: Sized> ElementNode<T> {
    pub fn new(id: WidgetId, content: T) -> ElementNode<T> {
        ElementNode {
            id: Some(id),
            transform: Affine::IDENTITY,
            content,
        }
    }
}

impl<T: ?Sized> ElementNode<T> {
    /// Sets the position of the contained element relative to the parent.
    ///
    /// Shorthand for `set_transform(Affine::translate(pos))`
    pub fn set_position(&mut self, pos: Point) {
        self.transform = Affine::translate(pos.to_vec2());
    }

    /// Sets the transform applied to the content element.
    pub fn set_transform(&mut self, tr: Affine) {
        self.transform = tr;
    }
}

impl<T: ?Sized + Element> Element for ElementNode<T> {
    fn id(&self) -> Option<WidgetId> {
        self.id.or_else(|| self.content.id())
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, params: &LayoutParams) -> Geometry {
        todo!()
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        todo!()
    }
}
