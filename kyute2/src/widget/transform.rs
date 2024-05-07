use crate::{
    environment::Environment, BoxConstraints, ChangeFlags, Event, Geometry, HitTestResult, LayoutCtx, PaintCtx,
    TreeCtx, Widget, WidgetId,
};
use kurbo::{Affine, Point, Vec2};
use std::cell::Cell;

/// A container for a widget.
///
/// TODO: make a version with only an offset instead of a full-blown transform
pub struct TransformNode<T: ?Sized = dyn Widget> {
    /// Parent-to-local transform.
    pub transform: Affine,
    pub content: T,
}

impl<T: Sized> TransformNode<T> {
    pub fn new(content: T) -> TransformNode<T> {
        TransformNode {
            transform: Affine::IDENTITY,
            content,
        }
    }
}

impl<T: ?Sized> TransformNode<T> {
    /// Sets the position of the contained element relative to the parent.
    ///
    /// Shorthand for `set_transform(Affine::translate(offset))`
    pub fn set_offset(&mut self, offset: Vec2) {
        self.transform = Affine::translate(offset);
    }

    /// Sets the transform applied to the content element.
    pub fn set_transform(&mut self, tr: Affine) {
        self.transform = tr;
    }

    /// Returns the transform applied to the content.
    pub fn transform(&self) -> Affine {
        self.transform
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
    fn update(&mut self, cx: &mut TreeCtx) {
        self.content.update(cx)
    }

    fn environment(&self) -> Environment {
        self.content.environment()
    }

    fn event(&mut self, cx: &mut TreeCtx, event: &mut Event) {
        event.with_transform(&self.transform, |event| self.content.event(cx, event))
    }

    fn hit_test(&mut self, result: &mut HitTestResult, position: Point) -> bool {
        result.test_with_transform(&self.transform, position, |result, position| {
            self.content.hit_test(result, position)
        })
    }

    fn layout(&mut self, cx: &mut LayoutCtx, bc: &BoxConstraints) -> Geometry {
        self.content.layout(cx, bc)
    }

    fn paint(&mut self, cx: &mut PaintCtx) {
        cx.with_transform(&self.transform, |cx| self.content.paint(cx));
    }
}