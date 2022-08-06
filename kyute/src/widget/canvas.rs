use crate::{widget::prelude::*, Length, Transform};
use kyute::style::WidgetState;
use std::sync::Arc;

pub enum PositioningMode {
    /// Position relative to center.
    Center,
    /// Position relative to leading edge (
    Leading,
    Trailing,
    SizeRelative,
}

#[derive(Clone)]
struct CanvasItem {
    offset_x: Length,
    offset_y: Length,
    widget: Arc<WidgetPod>,
    anchor: Alignment,
}

#[derive(Clone)]
pub struct Canvas {
    id: WidgetId,
    transform: Transform,
    left: Length,
    right: Length,
    top: Length,
    bottom: Length,
    items: Vec<CanvasItem>,
}

impl Canvas {
    #[composable]
    pub fn new() -> Canvas {
        Canvas {
            id: WidgetId::here(),
            transform: Transform::identity(),
            left: Length::Dip(f64::NEG_INFINITY),
            right: Length::Dip(f64::INFINITY),
            top: Length::Dip(f64::NEG_INFINITY),
            bottom: Length::Dip(f64::INFINITY),
            items: vec![],
        }
    }

    pub fn bounds(
        mut self,
        left: impl Into<Length>,
        top: impl Into<Length>,
        right: impl Into<Length>,
        bottom: impl Into<Length>,
    ) -> Self {
        self.set_bounds(left, top, right, bottom);
        self
    }

    pub fn set_bounds(
        &mut self,
        left: impl Into<Length>,
        top: impl Into<Length>,
        right: impl Into<Length>,
        bottom: impl Into<Length>,
    ) {
        self.left = left.into();
        self.top = top.into();
        self.right = right.into();
        self.bottom = bottom.into();
    }

    pub fn set_transform(&mut self, transform: Transform) {
        self.transform = transform;
    }

    pub fn item(
        mut self,
        offset_x: impl Into<Length>,
        offset_y: impl Into<Length>,
        widget: impl Widget + 'static,
    ) -> Canvas {
        self.add_item(offset_x, offset_y, widget);
        self
    }

    pub fn add_item(
        &mut self,
        offset_x: impl Into<Length>,
        offset_y: impl Into<Length>,
        widget: impl Widget + 'static,
    ) {
        self.items.push(CanvasItem {
            anchor: Alignment::CENTER,
            offset_x: offset_x.into(),
            offset_y: offset_y.into(),
            widget: Arc::new(WidgetPod::new(widget)),
        });
    }
}

impl Widget for Canvas {
    fn widget_id(&self) -> Option<WidgetId> {
        Some(self.id)
    }

    fn layout(&self, ctx: &mut LayoutCtx, constraints: &LayoutParams, env: &Environment) -> BoxLayout {
        // a canvas always takes the maximum available space
        let width = constraints.finite_max_width().unwrap_or(0.0);
        let height = constraints.finite_max_height().unwrap_or(0.0);

        let left = self.left.compute(constraints);
        let top = self.top.compute(constraints);
        let right = self.right.compute(constraints);
        let bottom = self.bottom.compute(constraints);

        // place the items in the canvas
        // padding is ignored
        for item in self.items.iter() {
            let child_layout_constraints = LayoutParams {
                widget_state: WidgetState::default(),
                parent_font_size: constraints.parent_font_size,
                scale_factor: constraints.scale_factor,
                min: Size::zero(),
                max: Size::new(f64::INFINITY, f64::INFINITY),
            };
            let layout = item.widget.layout(ctx, &child_layout_constraints, env);
            let mut offset = Offset::new(item.offset_x.compute(constraints), item.offset_y.compute(constraints));

            // prevent item from going out of bounds
            offset.x = offset.x.clamp(left, right - layout.measurements.width());
            offset.y = offset.y.clamp(top, bottom - layout.measurements.height());

            let transform = offset.to_transform().then(&self.transform);
            item.widget.set_transform(transform);
        }

        let size = Size::new(width, height);
        BoxLayout::new(size)
    }

    fn event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        for item in self.items.iter() {
            item.widget.route_event(ctx, event, env)
        }
    }

    fn paint(&self, ctx: &mut PaintCtx) {
        for item in self.items.iter() {
            item.widget.paint(ctx)
        }
    }
}

/// A widget that applies an arbitrary transform to its content.
pub struct Viewport<Content> {
    content: WidgetPod<Content>,
    transform: Transform,
    constrain_width: bool,
    constrain_height: bool,
}

impl<Content: Widget + 'static> Viewport<Content> {
    #[composable]
    pub fn new(content: Content) -> Viewport<Content> {
        Viewport {
            transform: Transform::identity(),
            content: WidgetPod::new(content),
            constrain_width: false,
            constrain_height: false,
        }
    }

    /// Sets the transform of the content.
    pub fn transform(mut self, transform: Transform) -> Self {
        self.set_transform(transform);
        self
    }

    /// Sets the transform of the content.
    pub fn set_transform(&mut self, transform: Transform) {
        self.transform = transform;
    }

    /// Constrains the content width to this viewport's width.
    ///
    /// By default, the content width is unconstrained.
    /// Calling this method constrains the maximum content width to be less than this viewport's width.
    pub fn constrain_width(mut self) -> Self {
        self.constrain_width = true;
        self
    }

    /// Constrains the content height to this viewport's height.
    ///
    /// By default, the content height is unconstrained.
    /// Calling this method constrains the maximum content height to be less than this viewport's height.
    pub fn constrain_height(mut self) -> Self {
        self.constrain_height = true;
        self
    }

    pub fn content(&self) -> &Content {
        self.content.inner()
    }
}

impl<Content: Widget + 'static> Widget for Viewport<Content> {
    fn widget_id(&self) -> Option<WidgetId> {
        self.content.inner().widget_id()
    }

    fn layout(&self, ctx: &mut LayoutCtx, constraints: &LayoutParams, env: &Environment) -> BoxLayout {
        let mut subconstraints = *constraints;
        if !self.constrain_width {
            subconstraints.min.width = 0.0;
            subconstraints.max.width = f64::INFINITY;
        }
        if !self.constrain_height {
            subconstraints.min.height = 0.0;
            subconstraints.max.height = f64::INFINITY;
        }

        // unconstrained
        self.content.layout(ctx, &subconstraints, env);

        if !ctx.speculative {
            self.content.set_transform(self.transform);
        }

        // always take the maximum available space
        let width = constraints.finite_max_width().unwrap_or(0.0);
        let height = constraints.finite_max_height().unwrap_or(0.0);
        let size = Size::new(width, height);

        // FIXME TODO we discarded any padding / alignment doesn't make much sense as well
        BoxLayout {
            x_align: Default::default(),
            y_align: Default::default(),
            padding_left: 0.0,
            padding_top: 0.0,
            padding_right: 0.0,
            padding_bottom: 0.0,
            measurements: Measurements::from(size),
        }
    }

    fn event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        self.content.route_event(ctx, event, env)
    }

    fn paint(&self, ctx: &mut PaintCtx) {
        self.content.paint(ctx)
    }
}
