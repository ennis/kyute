//! Frame containers
use crate::{
    debug_util::DebugWriter, element::TransformNode, layout::place_into, widget::Axis, Alignment, ChangeFlags, Element,
    ElementId, Environment, Event, EventCtx, Geometry, HitTestResult, Insets, LayoutCtx, LayoutParams,
    LengthOrPercentage, PaintCtx, Point, Rect, RouteEventCtx, Size, TreeCtx, Vec2, Widget,
};
use std::any::Any;
use tracing::trace;

/// A container with a fixed width and height, into which an unique widget is placed.
pub struct FrameElement<T> {
    width: LengthOrPercentage,
    height: LengthOrPercentage,
    change_flags: ChangeFlags,
    /// Horizontal content alignment.
    x_align: Alignment,
    /// Vertical content alignment.
    y_align: Alignment,
    /// Content padding.
    padding_left: LengthOrPercentage,
    padding_right: LengthOrPercentage,
    padding_top: LengthOrPercentage,
    padding_bottom: LengthOrPercentage,
    scale_factor: f64,
    /// Computed size
    size: Size,
    /// Computed bounds
    bounding_rect: Rect,
    paint_bounding_rect: Rect,
    content: TransformNode<T>,
}

impl<T> FrameElement<T> {
    /// Updates this element's change flags given the changes reported by
    /// the content element.
    ///
    /// # Arguments
    /// * f the change flags reported by the content element
    ///
    /// # Return value
    /// * the changes to be reported to the element above this `FrameElement`.
    fn update_change_flags(&mut self, f: ChangeFlags) -> ChangeFlags {
        // propagate all flags, except GEOMETRY since the size of frames is fixed
        // and does not adapt to the content. Thus, child geometry changes do not affect
        // the geometry of this frame
        self.change_flags |= f.difference(ChangeFlags::GEOMETRY);
        if f.intersects(ChangeFlags::SIZE) {
            // if the size of the content has changed, we'll need to recompute its size and reposition it.
            self.change_flags |= ChangeFlags::CHILD_GEOMETRY | ChangeFlags::LAYOUT_CHILD_POSITIONS;
        }
        if f.intersects(ChangeFlags::POSITIONING) {
            // if the content reports that only the positioning elements have changed, then  the positioning has changed, not its size given the same constraints
            self.change_flags |= ChangeFlags::LAYOUT_CHILD_POSITIONS;
        }
        self.change_flags
    }
}

impl<T: Element + 'static> Element for FrameElement<T> {
    fn id(&self) -> ElementId {
        self.content.id()
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, params: &LayoutParams) -> Geometry {
        // First, determine the size of this frame.
        // If any lengths are specified as percentages, resolve them:
        // consider percentage lengths as relative to the maximum available space.
        // TODO emit warning if available space is infinite
        let width = params.constrain_width(self.width.resolve(params.max.width));
        let height = params.constrain_height(self.height.resolve(params.max.height));
        let padding_left = self.padding_left.resolve(params.max.width);
        let padding_right = self.padding_right.resolve(params.max.width);
        let padding_top = self.padding_top.resolve(params.max.height);
        let padding_bottom = self.padding_top.resolve(params.max.height);

        // Computed size of the frame: just apply constraints from the parent element.
        let size = Size::new(width, height);

        // Call layout on the content. This is only necessary if:
        // - the computed size of the frame has changed (because the constraints passed to the child change in turn)
        // - the scale factor has changed (this invalidates all layouts)
        // - the current change flags says the child geometry or position are dirty
        if self.size != size
            || self.scale_factor != params.scale_factor
            || self
                .change_flags
                .intersects(ChangeFlags::CHILD_GEOMETRY | ChangeFlags::LAYOUT_CHILD_POSITIONS)
        {
            let sub = LayoutParams { max: size, ..*params };
            let content_geom = self.content.layout(ctx, &sub);

            if self.change_flags.contains(ChangeFlags::LAYOUT_CHILD_POSITIONS) {
                let offset = place_into(
                    content_geom.size,
                    content_geom.baseline,
                    size,
                    None,
                    self.x_align,
                    self.y_align,
                    &Insets::new(padding_left, padding_right, padding_top, padding_bottom),
                );
                self.content.set_offset(offset);
            }

            // update our bounding rectangles
            self.bounding_rect = self.content.transform.transform_rect_bbox(content_geom.bounding_rect);
            self.paint_bounding_rect = self
                .content
                .transform
                .transform_rect_bbox(content_geom.paint_bounding_rect);
        }

        self.scale_factor = params.scale_factor;
        self.size = size;
        self.change_flags = ChangeFlags::empty();
        // TODO propagate baseline
        Geometry {
            size,
            baseline: None,
            bounding_rect: self.bounding_rect,
            paint_bounding_rect: self.paint_bounding_rect,
        }
    }

    fn event(&mut self, ctx: &mut EventCtx, event: &mut Event) -> ChangeFlags {
        let flags = self.content.event(ctx, event);
        self.update_change_flags(flags)
    }

    /*fn route_event(&mut self, ctx: &mut RouteEventCtx, event: &mut Event) -> ChangeFlags {
        // we inherit the ID of the content so forward it
        let flags = self.content.route_event(ctx, event);
        self.update_change_flags(flags)
    }*/

    fn natural_size(&mut self, axis: Axis, params: &LayoutParams) -> f64 {
        let size = match axis {
            Axis::Horizontal => params.constrain_width(self.width.resolve(params.max.width)),
            Axis::Vertical => params.constrain_height(self.height.resolve(params.max.height)),
        };
        if !size.is_finite() {
            self.content.natural_size(axis, params)
        } else {
            size
        }
    }

    fn natural_baseline(&mut self, params: &LayoutParams) -> f64 {
        // TODO: welp, we'd need to take alignment and padding into account here
        self.content.natural_baseline(params)
    }

    fn hit_test(&self, ctx: &mut HitTestResult, position: Point) -> bool {
        if self.bounding_rect.contains(position) {
            self.content.hit_test(ctx, position);
            if self.size.to_rect().contains(position) {
                ctx.add(self.id());
                return true;
            }
        }
        false
    }

    fn paint(&mut self, ctx: &mut PaintCtx) {
        self.content.paint(ctx)
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn debug(&self, visitor: &mut DebugWriter) {
        visitor.type_name("FrameElement");
        visitor.property("width", self.width);
        visitor.property("height", self.height);
        visitor.property("x_align", self.x_align);
        visitor.property("y_align", self.y_align);
        visitor.property("padding_left", self.padding_left);
        visitor.property("padding_right", self.padding_right);
        visitor.property("padding_top", self.padding_top);
        visitor.property("padding_bottom", self.padding_bottom);
        visitor.property("scale_factor", self.scale_factor);
        visitor.property("size", self.size);
        visitor.property("bounding_rect", self.bounding_rect);
        visitor.property("paint_bounding_rect", self.paint_bounding_rect);
        visitor.child("content", &self.content);
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct Frame<T> {
    width: LengthOrPercentage,
    height: LengthOrPercentage,
    x_align: Alignment,
    y_align: Alignment,
    padding_left: LengthOrPercentage,
    padding_right: LengthOrPercentage,
    padding_top: LengthOrPercentage,
    padding_bottom: LengthOrPercentage,
    content: T,
}

impl<T> Frame<T> {
    pub fn new(width: LengthOrPercentage, height: LengthOrPercentage, content: T) -> Frame<T> {
        Frame {
            width,
            height,
            x_align: Default::default(),
            y_align: Default::default(),
            padding_left: Default::default(),
            padding_right: Default::default(),
            padding_top: Default::default(),
            padding_bottom: Default::default(),
            content,
        }
    }
}

impl<T: Widget> Widget for Frame<T> {
    type Element = FrameElement<T::Element>;

    fn build(self, cx: &mut TreeCtx, element_id: ElementId) -> Self::Element {
        let content = cx.build(self.content);
        trace!("build Frame");
        FrameElement {
            content: TransformNode::new(content),
            width: self.width,
            height: self.height,
            change_flags: ChangeFlags::ALL,
            x_align: Default::default(),
            y_align: Default::default(),
            padding_left: self.padding_left,
            padding_right: self.padding_right,
            padding_top: self.padding_top,
            padding_bottom: self.padding_bottom,
            scale_factor: 0.0,
            size: Default::default(),
            bounding_rect: Default::default(),
            paint_bounding_rect: Default::default(),
        }
    }

    fn update(self, cx: &mut TreeCtx, element: &mut Self::Element) -> ChangeFlags {
        // update width/height
        if self.width != element.width || self.height != element.height {
            element.width = self.width;
            element.height = self.height;
            // if the specified frame size changes, then our geometry changes (of course),
            // the position of the content may change, and its size as well
            // (since the layout constraints passed to the child change).
            element.change_flags |=
                ChangeFlags::SIZE | ChangeFlags::LAYOUT_CHILD_POSITIONS | ChangeFlags::CHILD_GEOMETRY;
        }
        if self.padding_top != element.padding_top
            || self.padding_bottom != element.padding_bottom
            || self.padding_right != element.padding_right
            || self.padding_left != element.padding_left
            || self.x_align != element.x_align
            || self.y_align != element.y_align
        {
            element.padding_top = self.padding_top;
            element.padding_right = self.padding_right;
            element.padding_left = self.padding_left;
            element.padding_bottom = self.padding_bottom;
            element.x_align = self.x_align;
            element.y_align = self.y_align;
            element.change_flags |= ChangeFlags::LAYOUT_CHILD_POSITIONS;
        }

        // update contents
        let flags = element.content.update(cx, self.content);
        element.update_change_flags(flags)
    }
}
