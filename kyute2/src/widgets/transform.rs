use kurbo::{Affine, Point, Vec2};

use crate::{
    core::{WeakWidget, WeakWidgetPtr},
    environment::Environment,
    BoxConstraints, Ctx, Event, Geometry, HitTestResult, LayoutCtx, PaintCtx, Widget, WidgetCtx, WidgetPod, WidgetPtr,
    WidgetPtrAny,
};

/// A container for a widget.
///
pub struct TransformNode {
    weak: WeakWidgetPtr<Self>,
    /// Parent-to-local transform.
    pub transform: Affine,
    pub content: WidgetPtrAny,
}

impl TransformNode {
    pub fn new(content: impl Into<WidgetPtrAny>) -> WidgetPtr<TransformNode> {
        WidgetPod::new_cyclic(|weak| TransformNode {
            weak,
            transform: Affine::IDENTITY,
            content: content.into(),
        })
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
}

impl WeakWidget for TransformNode {
    fn weak_self(&self) -> WeakWidgetPtr<Self> {
        self.weak.clone()
    }
}

impl Widget for TransformNode {
    fn mount(&mut self, cx: &mut Ctx) {
        self.content.mount(cx)
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
