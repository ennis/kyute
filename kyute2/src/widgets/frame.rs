//! Frame containers
use std::cell::Cell;

use kurbo::{Affine, Vec2};

use crate::{
    drawing::{Decoration, RoundedRectBorder, ShapeBorder, ShapeDecoration},
    environment::Environment,
    layout::place_into,
    Alignment, BoxConstraints, ChangeFlags, Ctx, Event, Geometry, HitTestResult, Insets, LayoutCtx, LengthOrPercentage,
    PaintCtx, Point, Rect, Size, Widget, WidgetCtx, WidgetPod, WidgetPtrAny,
};

/// A container with a fixed width and height, into which a unique widget is placed.
pub struct Frame<B> {
    width: LengthOrPercentage,
    height: LengthOrPercentage,
    change_flags: Cell<ChangeFlags>,
    /// Horizontal content alignment.
    x_align: Alignment,
    /// Vertical content alignment.
    y_align: Alignment,
    /// Content padding.
    padding_left: LengthOrPercentage,
    padding_right: LengthOrPercentage,
    padding_top: LengthOrPercentage,
    padding_bottom: LengthOrPercentage,
    decoration: ShapeDecoration<B>,
    /// Computed size
    size: Size,
    offset: Vec2,
    /// Computed bounds
    bounding_rect: Rect,
    paint_bounding_rect: Rect,
    content: WidgetPtrAny,
}

impl Frame<RoundedRectBorder> {
    pub fn new(
        width: LengthOrPercentage,
        height: LengthOrPercentage,
        content: impl Widget,
    ) -> Frame<RoundedRectBorder> {
        Frame {
            width,
            height,
            change_flags: Cell::new(ChangeFlags::all()),
            x_align: Default::default(),
            y_align: Default::default(),
            padding_left: Default::default(),
            padding_right: Default::default(),
            padding_top: Default::default(),
            padding_bottom: Default::default(),
            size: Default::default(),
            offset: Default::default(),
            bounding_rect: Default::default(),
            paint_bounding_rect: Default::default(),
            decoration: ShapeDecoration::new(),
            content: WidgetPod::new(content),
        }
    }
}

/*
impl<T, B> Frame<T, B> {
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
        self.change_flags |= f.difference(ChangeFlags::GEOMETRY | ChangeFlags::APP_LOGIC);
        if f.intersects(ChangeFlags::GEOMETRY) {
            // if the size of the content has changed, we'll need to recompute its size and reposition it.
            self.change_flags |= ChangeFlags::CHILD_GEOMETRY | ChangeFlags::LAYOUT_CHILD_POSITIONS;
        }
        self.change_flags
    }
}*/

impl<B: ShapeBorder + 'static> Widget for Frame<B> {
    fn mount(&mut self, cx: &mut WidgetCtx<Self>) {
        self.content.dyn_mount(cx)
    }

    fn hit_test(&mut self, ctx: &mut HitTestResult, position: Point) -> bool {
        ctx.test_with_offset(self.offset, position, |result, position| {
            self.content.dyn_hit_test(result, position)
        }) || self.bounding_rect.contains(position)
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, params: &BoxConstraints) -> Geometry {
        // First, determine the size of this frame.
        // If any lengths are specified as percentages, resolve them:
        // consider percentage lengths as relative to the maximum available space.
        // TODO emit warning if available space is infinite
        let width = params.constrain_width(self.width.resolve(params.max.width));
        let height = params.constrain_height(self.height.resolve(params.max.height));

        // compute padding: resolve user padding + padding added by the decoration
        let deco_insets = self.decoration.insets();
        let padding_left = self.padding_left.resolve(params.max.width) + deco_insets.x0;
        let padding_right = self.padding_right.resolve(params.max.width) - deco_insets.x1;
        let padding_top = self.padding_top.resolve(params.max.height) + deco_insets.y0;
        let padding_bottom = self.padding_bottom.resolve(params.max.height) - deco_insets.y1;

        // Computed size of the frame: just apply constraints from the parent element.
        let size = Size::new(width, height);

        // Call layout on the content. This is only necessary if:
        // - the computed size of the frame has changed (because the constraints passed to the child change in turn)
        // - the scale factor has changed (this invalidates all layouts)
        // - the current change flags says the child geometry or position are dirty
        //if self.size != size
        //    || self
        //        .change_flags
        //        .intersects(ChangeFlags::CHILD_GEOMETRY | ChangeFlags::LAYOUT_CHILD_POSITIONS)
        //{
        let content_geom = self.content.layout(ctx, &BoxConstraints { max: size, ..*params });

        if self.change_flags.get().contains(ChangeFlags::LAYOUT_CHILD_POSITIONS) {
            let offset = place_into(
                content_geom.size,
                content_geom.baseline,
                size,
                None,
                self.x_align,
                self.y_align,
                &Insets::new(padding_left, padding_top, padding_right, padding_bottom),
            );
            let transform = Affine::translate(offset);
            self.bounding_rect = transform.transform_rect_bbox(content_geom.bounding_rect);
            self.paint_bounding_rect = transform.transform_rect_bbox(content_geom.paint_bounding_rect);
            self.offset = offset;
        }

        self.size = size;

        Geometry {
            size,
            baseline: None,
            bounding_rect: self.bounding_rect,
            paint_bounding_rect: self.paint_bounding_rect,
        }
    }

    fn paint(&mut self, ctx: &mut PaintCtx) {
        ctx.with_canvas(|canvas| {
            self.decoration.paint(canvas, self.size.to_rect());
        });
        ctx.with_offset(self.offset, |ctx| {
            self.content.paint(ctx);
        });
    }
}

/*
////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct Frame<T, B> {
    /// Maximum width of the frame.
    pub width: LengthOrPercentage,
    /// Maximum height of the frame.
    pub height: LengthOrPercentage,
    /// Horizontal content alignment.
    pub x_align: Alignment,
    /// Vertical content alignment.
    pub y_align: Alignment,
    /// Padding on the left side of the content.
    pub padding_left: LengthOrPercentage,
    /// Padding on the right side of the content.
    pub padding_right: LengthOrPercentage,
    /// Padding on the top side of the content.
    pub padding_top: LengthOrPercentage,
    /// Padding on the bottom side of the content.
    pub padding_bottom: LengthOrPercentage,
    /// Decoration of the frame.
    ///
    /// May add additional padding.
    pub decoration: ShapeDecoration<B>,
    /// The content of the frame.
    pub content: T,
}

impl<T> Frame<T, RoundedRectBorder> {
    pub fn new(width: LengthOrPercentage, height: LengthOrPercentage, content: T) -> Frame<T, RoundedRectBorder> {
        Frame {
            width,
            height,
            x_align: Default::default(),
            y_align: Default::default(),
            padding_left: Default::default(),
            padding_right: Default::default(),
            padding_top: Default::default(),
            padding_bottom: Default::default(),
            decoration: ShapeDecoration::new(),
            content,
        }
    }
}

impl<T, B> Frame<T, B> {
    /// Sets the decoration of this frame.
    pub fn decoration<C>(self, decoration: ShapeDecoration<C>) -> Frame<T, C> {
        Frame {
            width: self.width,
            height: self.height,
            x_align: self.x_align,
            y_align: self.y_align,
            padding_left: self.padding_left,
            padding_right: self.padding_right,
            padding_top: self.padding_top,
            padding_bottom: self.padding_bottom,
            decoration,
            content: self.content,
        }
    }
}

impl<T: Widget, B: ShapeBorder + 'static> Widget for Frame<T, B> {
    type Element = FrameElement<T::Element, B>;

    fn build(self, cx: &mut TreeCtx, _element_id: ElementId) -> Self::Element {
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
            size: Default::default(),
            bounding_rect: Default::default(),
            paint_bounding_rect: Default::default(),
            decoration: self.decoration,
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
                ChangeFlags::GEOMETRY | ChangeFlags::LAYOUT_CHILD_POSITIONS | ChangeFlags::CHILD_GEOMETRY;
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

        // TODO check for decoration change
        element.decoration = self.decoration;

        // update contents
        let flags = element.content.update(cx, self.content);
        element.update_change_flags(flags);
        flags
    }
}
*/
