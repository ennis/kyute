use kurbo::{Affine, Point, Vec2};

use crate::{
    environment::Environment, BoxConstraints, Ctx, Event, Geometry, HitTestResult, LayoutCtx, PaintCtx, Widget,
};

/// A container for a widget.
///
pub struct TransformNode<W> {
    /// Parent-to-local transform.
    pub transform: Affine,
    pub content: W,
}

impl<W> TransformNode<W> {
    pub fn new(content: W) -> TransformNode<W> {
        TransformNode {
            transform: Affine::IDENTITY,
            content: content.into(),
        }
    }
}

impl<W> TransformNode<W> {
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
}

impl<W: Widget> Widget for TransformNode<W> {
    fn mount(&mut self, cx: &mut Ctx) {
        self.content.mount(cx)
    }

    fn update(&mut self, cx: &mut Ctx) {
        self.content.update(cx)
    }

    fn environment(&self) -> Environment {
        self.content.environment()
    }

    fn event(&mut self, cx: &mut Ctx, event: &mut Event) {
        event.with_transform(&self.transform, |event| {
            self.content.event(cx, event);
        });
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
