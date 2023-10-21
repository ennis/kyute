//!
use crate::{
    widget::Axis, Affine, ChangeFlags, Environment, Event, EventCtx, Geometry, HitTestResult, LayoutCtx, LayoutParams,
    PaintCtx, Point, Rect, RouteEventCtx, TreeCtx, Vec2, Widget, WidgetId,
};
use std::any::Any;
use tracing::warn;

pub trait Element: 'static {
    /// Returns this element's ID.
    fn id(&self) -> WidgetId;

    /// Measures this widget and layouts the children of this widget.
    fn layout(&mut self, ctx: &mut LayoutCtx, params: &LayoutParams) -> Geometry;

    /// Deliver an event to this element or one of its children.
    fn event(&mut self, ctx: &mut EventCtx, event: &mut Event) -> ChangeFlags;

    /// Routes an event through this element, to a child element.
    fn route_event(&mut self, ctx: &mut RouteEventCtx, event: &mut Event) -> ChangeFlags {
        if let Some(_next_target) = event.next_target() {
            warn!("Default `route_event` implementation called but event has a next target. Implement `route_event` to route the event to child elements.");
            ChangeFlags::NONE
        } else {
            self.event(&mut ctx.inner, event)
        }
    }

    /// Returns the _natural size_ of the element along the given axis.
    ///
    /// The _natural size_ of the element on an axis is the size it would take if the constraints
    /// on that axis were unbounded.
    ///
    /// It should be finite.
    // This is like druid's "compute_max_intrinsic", or flutter's getMaxIntrinsic{Width,Height}
    fn natural_size(&mut self, axis: Axis, params: &LayoutParams) -> f64;

    /// Returns the _natural baseline_ of the element.
    fn natural_baseline(&mut self, params: &LayoutParams) -> f64;

    fn hit_test(&self, ctx: &mut HitTestResult, position: Point) -> bool;

    /// Called to paint the widget.
    fn paint(&mut self, ctx: &mut PaintCtx);

    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl Element for Box<dyn Element> {
    fn id(&self) -> WidgetId {
        (&**self).id()
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, params: &LayoutParams) -> Geometry {
        (&mut **self).layout(ctx, params)
    }

    fn event(&mut self, ctx: &mut EventCtx, event: &mut Event) -> ChangeFlags {
        (&mut **self).event(ctx, event)
    }

    fn route_event(&mut self, ctx: &mut RouteEventCtx, event: &mut Event) -> ChangeFlags {
        (&mut **self).route_event(ctx, event)
    }

    fn natural_size(&mut self, axis: Axis, params: &LayoutParams) -> f64 {
        (&mut **self).natural_size(axis, params)
    }

    fn natural_baseline(&mut self, params: &LayoutParams) -> f64 {
        (&mut **self).natural_baseline(params)
    }

    fn hit_test(&self, ctx: &mut HitTestResult, position: Point) -> bool {
        (&**self).hit_test(ctx, position)
    }

    fn paint(&mut self, ctx: &mut PaintCtx) {
        (&mut **self).paint(ctx)
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    /*fn parent_data(&mut self) -> &mut dyn Any {
        (&mut **self).parent_data()
    }*/
}

////////////////////////////////////////////////////////////////////////////////////////////////////

/// A container for a widget.
pub struct TransformNode<T: ?Sized = dyn Element> {
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

    ///
    pub fn update<W>(&mut self, ctx: &mut TreeCtx, content_widget: W, env: &Environment) -> ChangeFlags
    where
        T: Element,
        W: Widget<Element = T>,
    {
        let change_flags = content_widget.update(ctx, &mut self.content, env);
        change_flags
    }
}

impl<T: Element> Element for TransformNode<T> {
    /// Returns the ID of the content element.
    fn id(&self) -> WidgetId {
        self.content.id()
    }

    /*/// Returns the bounding box of this element.
    pub fn bounding_rect(&self) -> Rect {
        self.transform
            .transform_rect_bbox(self.content.geometry().bounding_rect)
    }

    /// Returns the bounding box of this element.
    pub fn paint_bounding_rect(&self) -> Rect {
        self.transform
            .transform_rect_bbox(self.content.geometry().paint_bounding_rect)
    }*/

    /// Calls `layout` on the content element.
    fn layout(&mut self, ctx: &mut LayoutCtx, params: &LayoutParams) -> Geometry {
        self.content.layout(ctx, params)
    }

    fn event(&mut self, ctx: &mut EventCtx, event: &mut Event) -> ChangeFlags {
        // propagate down
        let change = self.content.event(ctx, event);
        /*if change.intersects(ChangeFlags::GEOMETRY) {
            self.geometry = None;
        }*/
        change
    }

    /// Propagates an event to the content element, applying the transform.
    fn route_event(&mut self, ctx: &mut RouteEventCtx, event: &mut Event) -> ChangeFlags {
        event.with_transform(&self.transform, |event| self.content.route_event(ctx, event))
    }

    fn natural_size(&mut self, axis: Axis, params: &LayoutParams) -> f64 {
        self.content.natural_size(axis, params)
    }

    fn natural_baseline(&mut self, params: &LayoutParams) -> f64 {
        self.content.natural_baseline(params)
    }

    /// Hit-tests the content element.
    fn hit_test(&self, ctx: &mut HitTestResult, position: Point) -> bool {
        let local_position = self.transform * position;
        self.content.hit_test(ctx, local_position)
    }

    fn paint(&mut self, ctx: &mut PaintCtx) {
        ctx.with_transform(&self.transform, |ctx| self.content.paint(ctx))
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
