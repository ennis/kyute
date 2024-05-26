use kurbo::{Affine, Point, Vec2};

use crate::{
    environment::Environment, BoxConstraints, Ctx, Event, Geometry, HitTestResult, LayoutCtx, PaintCtx, Widget,
    WidgetCtx, WidgetPod, WidgetPtrAny,
};

/// A container for a widget.
///
pub struct TransformNode {
    /// Parent-to-local transform.
    pub transform: Affine,
    pub content: WidgetPtrAny,
}

impl TransformNode {
    pub fn new(content: impl Widget) -> TransformNode {
        TransformNode {
            transform: Affine::IDENTITY,
            content: WidgetPod::new(content),
        }
    }
}

impl TransformNode {
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

impl Widget for TransformNode {
    fn mount(&mut self, cx: &mut WidgetCtx<Self>) {
        self.content.dyn_mount(cx)
    }

    fn hit_test(&mut self, result: &mut HitTestResult, position: Point) -> bool {
        result.test_with_transform(&self.transform, position, |result, position| {
            self.content.dyn_hit_test(result, position)
        })
    }

    fn layout(&mut self, cx: &mut LayoutCtx, bc: &BoxConstraints) -> Geometry {
        self.content.layout(cx, bc)
    }

    fn paint(&mut self, cx: &mut PaintCtx) {
        cx.with_transform(&self.transform, |cx| self.content.paint(cx));
    }
}
