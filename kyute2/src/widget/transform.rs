use crate::{
    BoxConstraints, ChangeFlags, Event, Geometry, HitTestResult, LayoutCtx, PaintCtx, TreeCtx, Widget, WidgetId,
};
use kurbo::{Affine, Point, Vec2};
use std::cell::Cell;

/// A container for a widget.
///
/// TODO: make a version with only an offset instead of a full-blown transform
pub struct TransformNode<T: ?Sized = dyn Widget> {
    /// Parent-to-local transform.
    pub transform: Cell<Affine>,
    pub content: T,
}

impl<T: Sized> TransformNode<T> {
    pub fn new(content: T) -> TransformNode<T> {
        TransformNode {
            transform: Cell::new(Affine::IDENTITY),
            content,
        }
    }
}

impl<T: ?Sized> TransformNode<T> {
    /// Sets the position of the contained element relative to the parent.
    ///
    /// Shorthand for `set_transform(Affine::translate(offset))`
    pub fn set_offset(&self, offset: Vec2) {
        self.transform.set(Affine::translate(offset));
    }

    /// Sets the transform applied to the content element.
    pub fn set_transform(&self, tr: Affine) {
        self.transform.set(tr);
    }

    /// Returns the transform applied to the content.
    pub fn transform(&self) -> Affine {
        self.transform.get()
    }

    /*///
    pub fn update<W>(&mut self, ctx: &mut TreeCtx, content_widget: W) -> ChangeFlags
    where
        T: Element,
        W: Widget<Element = T>,
    {
        let change_flags = content_widget.update(ctx, &mut self.content);
        change_flags
    }*/
}

impl<T: Widget> Widget for TransformNode<T> {
    fn update(&self, cx: &mut TreeCtx) {
        self.content.update(cx)
    }

    fn event(&self, cx: &mut TreeCtx, event: &mut Event) {
        event.with_transform(&self.transform.get(), |event| self.content.event(cx, event))
    }

    fn hit_test(&self, result: &mut HitTestResult, position: Point) -> bool {
        result.test_with_transform(&self.transform.get(), position, |result, position| {
            self.content.hit_test(result, position)
        })
    }

    fn layout(&self, cx: &mut LayoutCtx, bc: &BoxConstraints) -> Geometry {
        self.content.layout(cx, bc)
    }

    fn paint(&self, cx: &mut PaintCtx) {
        cx.with_transform(&self.transform.get(), |cx| self.content.paint(cx));
    }
}
